const GENERIC_ERROR: &str = "Could not connect to systemd.";

pub fn init() {
    spawn_watchdog_task();
    notify_ready();
}

pub fn notify_ready() {
    log::debug!("Letting systemd know we're ready..");
    systemd::daemon::notify(false, [(systemd::daemon::STATE_READY, "1")].iter())
        .expect(GENERIC_ERROR);
}

pub fn spawn_watchdog_task() {
    let timeout_ms = systemd::daemon::watchdog_enabled(false).expect(GENERIC_ERROR);
    if timeout_ms > 0 {
        let reset_interval_ms = timeout_ms / 2; // as recommended by systemd
        log::debug!("Systemd watchdog timeout is {} milliseconds.", timeout_ms);
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_millis(reset_interval_ms));

            loop {
                interval.tick().await;
                log::debug!("Systemd watchdog tick.."); // TODO
                systemd::daemon::notify(false, [(systemd::daemon::STATE_WATCHDOG, "1")].iter())
                    .expect(GENERIC_ERROR);
            }
        });
        log::info!(
            "Systemd watchdog will be reset every {} milliseconds.",
            reset_interval_ms
        );
    } else {
        log::debug!("Systemd watchdog is disabled.");
    }
}
