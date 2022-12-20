use crate::{Error, Result};

const UPTIME_PATH: &str = "/proc/uptime";

fn read_system_uptime() -> Result<std::time::Duration> {
    let buffer = std::fs::read_to_string(UPTIME_PATH)
        .map_err(|x| Error(format!("Error reading from {}: {}", UPTIME_PATH, x)))?;
    let line = buffer
        .lines()
        .next()
        .ok_or_else(|| Error(format!("Could not read from {}.", UPTIME_PATH)))?;
    let uptime: f64 = crate::get_number(
        &format!("Could not read uptime from {}", UPTIME_PATH),
        line,
        0,
    )?;
    Ok(std::time::Duration::from_secs_f64(uptime))
}

static mut START_TIME: Option<std::time::Instant> = None;
static mut START_SYSTEM_UPTIME: Option<std::time::Duration> = None;

static INIT: std::sync::Once = std::sync::Once::new();

pub fn system() -> std::time::Duration {
    unsafe { START_SYSTEM_UPTIME }.unwrap() + process()
}

pub fn process() -> std::time::Duration {
    std::time::Instant::now().duration_since(unsafe { START_TIME }.unwrap())
}

pub fn init() -> Result<()> {
    let mut res = Ok(());
    INIT.call_once(|| unsafe {
        START_TIME = Some(std::time::Instant::now());
        let system_uptime = read_system_uptime();
        match system_uptime {
            Ok(system_uptime) => START_SYSTEM_UPTIME = Some(system_uptime),
            Err(err) => res = Err(err),
        }
    });
    res
}
