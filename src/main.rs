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

// TODO check/handle unwraps!
// TODO initial checks (e.g. file system exists), matching check/alarm config types
// NOTE FilesystemUsage uses "available blocks" (not "free blocks") i.e. blocks available to
//      unpriv. users

fn get_config_file_path() -> Result<std::path::PathBuf, &'static str> {
    if let Some(path_str) = std::env::args().nth(1) {
        Ok(std::path::PathBuf::from(path_str))
    } else {
        Err("Config file path not specified.")
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

fn init_actions(config: &config::Config) -> HashMap<String, Box<dyn action::Trigger>> {
    log::info!("Initializing {} actions(s)..", config.actions.len());
    let mut res: HashMap<String, Box<dyn action::Trigger>> = HashMap::new();
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
                    Box::new(action::WebHook::from(action_config)),
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

fn init_checks(config: &config::Config) {
    log::info!("Initializing {} check(s)..", config.checks.len());
    for check_config in config.checks.iter() {
        if check_config.disable {
            log::info!(
                "Check {}::'{}' is disabled.",
                check_config.type_,
                check_config.name
            );
            continue;
        }
        log::info!(
            "Check {}::'{}' will be triggered every {} seconds.",
            check_config.type_,
            check_config.name,
            check_config.interval
        );
        let mut level_checks: Vec<(
            Box<dyn check::LevelSource>,
            HashMap<String, Box<dyn alarm::LevelSink>>,
        )> = Vec::new();
        match &check_config.type_ {
            config::CheckType::FilesystemUsage(filesystem_usage_config) => {
                use check::MeasurementIds;
                let level_check = Box::new(check::FilesystemUsage::from(filesystem_usage_config));
                let mut level_alarms: HashMap<String, Box<dyn alarm::LevelSink>> = HashMap::new();
                for alarm_config in check_config.alarms.iter() {
                    if alarm_config.disable {
                        log::info!("Alarm '{}' is disabled.", alarm_config.name);
                        continue;
                    }
                    log::info!(
                        "Alarm '{}' will be triggered after {} cycles.",
                        alarm_config.name,
                        alarm_config.cycles
                    );
                    for measurement_id in level_check.measurement_ids().iter() {
                        let level_alarm = Box::new(alarm::Level::from(alarm_config));
                        level_alarms.insert(measurement_id.clone(), level_alarm);
                    }
                }
                level_checks.push((level_check, level_alarms));
            }
        };
        // TODO use tasks
        for _ in 1..20 {
            for (level_check, level_alarms) in level_checks.iter_mut() {
                for (id, level) in level_check.get_levels() {
                    level_alarms.get_mut(&id).unwrap().level(level);
                }
            }
        }
    }
}

#[tokio::main]
//async fn main() -> Result<(), Box<dyn std::error::Error>> {
async fn main() {
    let config_file_path = get_config_file_path().expect("TODO handle this / pass on!");
    let config = config::Config::try_from(config_file_path.as_path()).unwrap();
    init_logging(&config);
    #[cfg(feature = "systemd")]
    {
        systemd::init();
    }
    init_actions(&config);
    init_checks(&config);
}
