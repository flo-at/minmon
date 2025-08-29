use crate::{Error, Result};

const SD_STATE_READY: &str = "READY=1";
const SD_STATE_WATCHDOG: &str = "WATCHDOG=1";

pub async fn init() {
    if !libsystemd::daemon::booted() {
        log::debug!("Could not detect systemd. Skipping notification and watchdog initialization.");
        return;
    }
    spawn_watchdog_task();
    if let Err(err) = sd_notify(&[SD_STATE_READY]).await {
        log::error!("Failed to notify systemd: {err}");
    }
}

pub fn init_journal() -> Result<()> {
    systemd_journal_logger::JournalLog::new()
        .map_err(|x| Error(format!("Could not create new journal logger: {x}")))?
        .install()
        .map_err(|x| Error(format!("Could not install journal logger: {x}")))
}

async fn sd_notify(state: &[&str]) -> Result<bool> {
    let env_sock = match std::env::var("NOTIFY_SOCKET").ok() {
        None => return Ok(false),
        Some(v) => v,
    };

    let socket = tokio::net::UnixDatagram::unbound()
        .map_err(|x| Error(format!("Failed to open Unix datagram socket: {x}")))?;

    let msg = state
        .iter()
        .fold(String::new(), |res, s| res + &format!("{s}\n"))
        .into_bytes();

    let sent_len = socket
        .send_to(&msg, env_sock)
        .await
        .map_err(|x| Error(format!("Failed to send notify datagram: {x}")))?;

    if sent_len != msg.len() {
        return Err(Error(format!(
            "Incomplete notify message: sent {} out of {}",
            sent_len,
            msg.len()
        )));
    }

    Ok(true)
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
                    if let Err(err) = sd_notify(&[SD_STATE_WATCHDOG]).await {
                        log::error!("Failed to reset systemd watchdog: {err}");
                    }
                }
            });
            log::debug!(
                "Systemd watchdog will be reset every {} milliseconds.",
                reset_interval.as_millis()
            );
        }
    } else {
        log::debug!("Systemd watchdog is disabled.");
    }
}
