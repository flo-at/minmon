mod action;
#[allow(unused_variables)] // TODO temporary
#[allow(dead_code)] // TODO temporary!
mod alarm;
mod check;
mod config;
mod placeholder;
#[cfg(feature = "systemd")]
mod systemd;
#[cfg(feature = "systemd")]
extern crate systemd as systemd_ext;

use std::collections::HashMap;

type ActionMap = HashMap<String, std::sync::Arc<dyn action::Trigger>>;

#[derive(Debug)]
pub struct Error(String);
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// TODO implement report
// TODO check/handle unwraps!
// TODO initial checks (e.g. file system exists), matching check/alarm config types
// NOTE FilesystemUsage uses "available blocks" (not "free blocks") i.e. blocks available to
//      unpriv. users

fn get_config_file_path() -> Result<std::path::PathBuf> {
    if let Some(path_str) = std::env::args().nth(1) {
        Ok(std::path::PathBuf::from(path_str))
    } else {
        Err(Error(String::from("Config file path not specified.")))
    }
}

fn init_logging(config: &config::Config) {
    match config.log.target {
        config::LogTarget::Stdout => {
            let mut builder = env_logger::Builder::from_default_env();
            builder
                .filter_level(log::LevelFilter::from(config.log.level))
                .format(|buf, record| {
                    use std::io::Write;
                    writeln!(
                        buf,
                        "{} [{}] {}",
                        buf.timestamp(),
                        record.level(),
                        record.args()
                    )
                })
                .format_timestamp_secs()
                .init();
        }
        #[cfg(feature = "systemd")]
        config::LogTarget::Journal => {
            systemd_ext::journal::JournalLog::init().unwrap();
        }
    }
}

fn init_actions(config: &config::Config) -> ActionMap {
    log::info!("Initializing {} actions(s)..", config.actions.len());
    let mut res = ActionMap::new();
    for action_config in config.actions.iter() {
        if action_config.disable {
            log::info!(
                "Action {}::'{}' is disabled.",
                action_config.type_,
                action_config.name
            );
            continue;
        }
        match &action_config.type_ {
            config::ActionType::WebHook(_) => {
                res.insert(
                    action_config.name.clone(),
                    std::sync::Arc::new(action::WebHook::from(action_config)),
                );
            }
        }
        log::info!(
            "Action {}::'{}' initialized.",
            action_config.type_,
            action_config.name
        );
    }
    res
}

fn init_checks(config: &config::Config, actions: &ActionMap) -> Vec<Box<dyn check::Check>> {
    log::info!("Initializing {} check(s)..", config.checks.len());
    let mut res: Vec<Box<dyn check::Check>> = Vec::new();
    for check_config in config.checks.iter() {
        if check_config.disable {
            log::info!(
                "Check {}::'{}' is disabled.",
                check_config.type_,
                check_config.name
            );
            continue;
        }
        let check = check::from_check_config(check_config, actions).unwrap(); // TODO
        log::info!(
            "Check {} will be triggered every {} seconds.",
            check.name(),
            check.interval().as_secs()
        );
        res.push(check);
    }
    res
}

#[tokio::main(flavor = "current_thread")]
//async fn main() -> Result<(), Box<dyn std::error::Error>> {
async fn main() {
    let config_file_path = get_config_file_path().expect("TODO handle this / pass on!");
    let config = config::Config::try_from(config_file_path.as_path()).unwrap();
    init_logging(&config);
    #[cfg(feature = "systemd")]
    {
        systemd::init();
    }
    let actions = init_actions(&config);
    let checks = init_checks(&config, &actions);
    for mut check in checks {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check.interval());
            loop {
                interval.tick().await;
                if let Err(error) = check.trigger().await {
                    log::error!(
                        "Error while check '{}' was triggered: {}",
                        check.name(),
                        error
                    );
                }
            }
        });
    }

    //let sigint = tokio::signal::unix::SignalKind::terminate().flatten_stream();

    let mut stream =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    stream.recv().await;
}
