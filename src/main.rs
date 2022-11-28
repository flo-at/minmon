mod action;
#[allow(unused_variables)] // TODO temporary
#[allow(dead_code)] // TODO temporary!
mod alarm;
mod check;
mod config;
#[cfg(feature = "systemd")]
mod systemd;
#[cfg(feature = "systemd")]
extern crate systemd as systemd_ext;

use std::collections::HashMap;

type ActionMap = HashMap<String, std::sync::Arc<dyn action::Action>>;
pub type PlaceholderMap = std::collections::HashMap<String, String>;

#[derive(Debug)]
pub struct Error(String);
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// TODO additional placeholders from config file (check, alarm, action)
// TODO implement report
// NOTE FilesystemUsage uses "available blocks" (not "free blocks") i.e. blocks available to
//      unpriv. users

fn get_config_file_path() -> Result<std::path::PathBuf> {
    if let Some(path_str) = std::env::args().nth(1) {
        Ok(std::path::PathBuf::from(path_str))
    } else {
        Err(Error(String::from("Config file path not specified.")))
    }
}

fn init_logging(config: &config::Config) -> Result<()> {
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
            systemd_ext::journal::JournalLog::init()
                .map_err(|x| Error(format!("Could not initialize journal logger: {}", x)))?;
        }
    }
    Ok(())
}

fn init_actions(config: &config::Config) -> Result<ActionMap> {
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
        res.insert(
            action_config.name.clone(),
            match &action_config.type_ {
                config::ActionType::WebHook(_) => {
                    std::sync::Arc::new(action::WebHook::try_from(action_config)?)
                }
                config::ActionType::Log(_) => {
                    std::sync::Arc::new(action::Log::try_from(action_config)?)
                }
            },
        );
        log::info!(
            "Action {}::'{}' initialized.",
            action_config.type_,
            action_config.name
        );
    }
    Ok(res)
}

fn init_checks(config: &config::Config, actions: &ActionMap) -> Result<Vec<Box<dyn check::Check>>> {
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
        let check = check::from_check_config(check_config, actions)?;
        log::info!(
            "Check {} will be triggered every {} seconds.",
            check.name(),
            check.interval().as_secs()
        );
        res.push(check);
    }
    Ok(res)
}

async fn main_wrapper() -> Result<()> {
    let config_file_path = get_config_file_path()?;
    let config = config::Config::try_from(config_file_path.as_path())
        .map_err(|x| Error(format!("Failed to parse config file: {}", x)))?;
    init_logging(&config)?;
    #[cfg(feature = "systemd")]
    {
        systemd::init();
    }
    let actions = init_actions(&config)?;
    let checks = init_checks(&config, &actions)?;
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
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(error) = main_wrapper().await {
        // Print to stderr here because logging might not be initialized if the config file cannot
        // be parsed.
        eprintln!("Exiting due to error: {}", error);
        std::process::exit(1);
    }
}
