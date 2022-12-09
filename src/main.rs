#![deny(warnings)]

#[cfg(feature = "systemd")]
mod systemd;
#[cfg(feature = "systemd")]
extern crate systemd as systemd_ext;

use minmon::{config, Error, Result};

// TODO process action: implement timeout
// TODO retry action on next trigger if failed previously
// TODO add "alarm_duration" and maybe "good_duration" on recover/action
// TODO journal logging with extra fields (check/alarm/action name, ..)
// TODO hierarchical logging (or just placeholders?)
// TODO include alarm/action "last status" in report to see if action execution works correctly
// TODO consistent debug logging
// TODO (example) configs in README
// TODO implement report
// TODO tests!

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

async fn main_wrapper() -> Result<()> {
    let config_file_path = get_config_file_path()?;
    let config = config::Config::try_from(config_file_path.as_path())
        .map_err(|x| Error(format!("Failed to parse config file: {}", x)))?;
    init_logging(&config)?;
    #[cfg(feature = "systemd")]
    {
        systemd::init();
    }
    let checks = minmon::from_config(&config)?;
    for mut check in checks {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check.interval());
            loop {
                interval.tick().await;
                if let Err(error) = check.trigger().await {
                    log::error!("Check '{}' failed: {}", check.name(), error);
                }
            }
        });
    }

    // TODO make sure all relevant signals are handles correctly
    //let sigint = tokio::signal::unix::SignalKind::terminate().flatten_stream();

    let mut stream =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    stream.recv().await;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(error) = main_wrapper().await {
        log::error!("Exiting due to error: {}", error);
        // Also print to stderr here because logging might not be initialized if the config file cannot
        // be parsed.
        eprintln!("Exiting due to error: {}", error);
        std::process::exit(1);
    }
}
