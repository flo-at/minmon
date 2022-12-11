#![deny(warnings)]

#[cfg(feature = "systemd")]
mod systemd;

use minmon::{config, Error, Result};

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
            systemd::init_journal()?;
        }
    }
    Ok(())
}

async fn main_wrapper() -> Result<()> {
    minmon::uptime::init()?;

    let config_file_path = get_config_file_path()?;
    let config = config::Config::try_from(config_file_path.as_path())
        .map_err(|x| Error(format!("Failed to parse config file: {}", x)))?;

    init_logging(&config)?;

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    log::info!("Starting MinMon v{}..", VERSION);

    #[cfg(feature = "systemd")]
    {
        systemd::init();
    }

    let (report, checks) = minmon::from_config(&config)?;
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

    if let Some(mut report) = report {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(report.interval());
            loop {
                interval.tick().await;
                if let Err(error) = report.trigger().await {
                    log::error!("Report failed: {}", error);
                }
            }
        });
    }

    use tokio::signal::unix::{signal, SignalKind};
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigint.recv() => log::info!("Received signal SIGINT. Shutting down."),
        _ = sigterm.recv() => log::info!("Received signal SIGTERM. Shutting down."),
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(error) = main_wrapper().await {
        log::error!("Exiting due to error: {}", error);
        // Also print to stderr here because logging might not be initialized if the config file
        // cannot be parsed.
        eprintln!("Exiting due to error: {}", error);
        std::process::exit(1);
    }
}
