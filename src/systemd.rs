use crate::{Error, Result};

const GENERIC_ERROR: &str = "Could not connect to systemd.";

pub fn init() {
    spawn_watchdog_task();
    notify_ready();
}

pub fn init_journal() -> Result<()> {
    systemd_journal_logger::init()
        .map_err(|x| Error(format!("Could not initialize journal logger: {}", x)))
}

fn notify_ready() {
    libsystemd::daemon::notify(false, &[libsystemd::daemon::NotifyState::Ready])
        .expect(GENERIC_ERROR);
}

fn spawn_watchdog_task() {
    if let Some(timeout) = libsystemd::daemon::watchdog_enabled(false) {
        if timeout.is_zero() {
            log::debug!("Systemd watchdog is disabled.");
        } else {
            let reset_interval = timeout / 2; // as recommended by systemd
            log::debug!(
                "Systemd watchdog timeout is {} milliseconds.",
                timeout.as_millis()
            );
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(reset_interval);
                loop {
                    interval.tick().await;
                    if let Err(err) = libsystemd::daemon::notify(
                        false,
                        &[libsystemd::daemon::NotifyState::Watchdog],
                    ) {
                        log::error!("Failed to reset systemd watchdog: {}", err);
                    }
                }
            });
            log::info!(
                "Systemd watchdog will be reset every {} milliseconds.",
                reset_interval.as_millis()
            );
        }
    } else {
        log::debug!("Systemd watchdog is disabled.");
    }
}
