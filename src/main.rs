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
        config::LogTarget::Stdout | config::LogTarget::Stderr => {
            let target = if config.log.target == config::LogTarget::Stdout {
                env_logger::Target::Stdout
            } else {
                env_logger::Target::Stderr
            };
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
                .target(target)
                .format_timestamp_secs()
                .init();
        }
        #[cfg(feature = "systemd")]
        config::LogTarget::Journal => {
            systemd::init_journal()?;
            log::set_max_level(log::LevelFilter::from(config.log.level));
        }
    }
    Ok(())
}

async fn random_interval(max: std::time::Duration) {
    let delay = rand::random::<f32>() * max.as_secs_f32() + 0.001;
    let mut delay = tokio::time::interval(std::time::Duration::from_secs_f32(delay));
    delay.tick().await; // the first tick completes immediately
    delay.tick().await;
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
            random_interval(check.interval()).await;
            let mut interval = tokio::time::interval(check.interval());
            loop {
                interval.tick().await;
                check.trigger().await;
            }
        });
    }

    if let Some(mut report) = report {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(report.interval());
            loop {
                interval.tick().await;
                report.trigger().await;
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
