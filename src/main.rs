#[allow(unused_variables)] // TODO temporary
#[allow(dead_code)] // TODO temporary!
mod config;
#[cfg(feature = "systemd")]
mod systemd;
#[cfg(feature = "systemd")]
extern crate systemd as systemd_ext;

use std::collections::HashMap;

// TODO check/handle unwraps!
// TODO initial checks (e.g. file system exists), mathing check/alarm config types
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
        let mut level_checks: Vec<(Box<dyn LevelSource>, HashMap<String, Box<dyn LevelSink>>)> =
            Vec::new();
        match &check_config.type_ {
            config::CheckType::FilesystemUsage(filesystem_usage_config) => {
                let level_check = Box::new(FilesystemUsage::from(filesystem_usage_config));
                let mut level_alarms: HashMap<String, Box<dyn LevelSink>> = HashMap::new();
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
                        let mut level_alarm = Box::new(AlarmLevel::from(alarm_config));
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

struct Alarm {
    name: String,
    action: String,
    cycles: u32,
    repeat_cycles: u32,
    recover_action: String,
    recover_cycles: u32,
    // --
    bad_cycles: u32,
    good_cycles: u32,
    good: bool,
}

impl Alarm {
    fn bad(&mut self) -> bool {
        self.good_cycles = 0;
        self.bad_cycles += 1;
        if self.bad_cycles >= self.cycles {
            let good_old = self.good;
            self.good = false;
            return good_old
                || (self.repeat_cycles > 0
                    && (self.bad_cycles - self.cycles) % self.repeat_cycles == 0);
        }
        false
    }

    fn good(&mut self) -> bool {
        self.bad_cycles = 0;
        self.good_cycles += 1;
        if self.good_cycles == self.recover_cycles {
            let good_old = self.good;
            self.good = true;
            return !good_old;
        }
        false
    }
}

impl From<&config::Alarm> for Alarm {
    fn from(alarm: &config::Alarm) -> Self {
        Self {
            name: alarm.name.clone(),
            action: alarm.action.clone(),
            cycles: alarm.cycles,
            repeat_cycles: alarm.repeat_cycles,
            recover_action: alarm.recover_action.clone(),
            recover_cycles: alarm.recover_cycles,
            bad_cycles: 0,
            good_cycles: 0,
            good: true,
        }
    }
}

struct AlarmLevel {
    alarm: Alarm,
    level: u8,
}

impl From<&config::Alarm> for AlarmLevel {
    fn from(alarm: &config::Alarm) -> Self {
        let config::AlarmType::Level(level_config) = &alarm.type_;
        Self {
            alarm: Alarm::from(alarm),
            level: level_config.level,
        }
    }
}

impl LevelSink for AlarmLevel {
    fn level(&mut self, level: u8) {
        log::debug!("Got level {} for alarm '{}'", level, self.alarm.name);
        if level >= self.level {
            if self.alarm.bad() {
                log::debug!("BAD action triggered!");
            }
        } else {
            if self.alarm.good() {
                log::debug!("GOOD action triggered!");
            }
        }
    }
}

// for alarms that expect a "level" (u8 in [0..100])
trait LevelSink {
    fn level(&mut self, level: u8);
}

// for checks that return "levels" (u8 in [0..100])
trait LevelSource {
    fn get_levels(&self) -> Vec<(String, u8)>;
}

trait MeasurementIds {
    fn measurement_ids(&self) -> &[String];
}

struct FilesystemUsage {
    mountpoints: Vec<String>, // TODO possible to store a reference?
}

impl From<&config::CheckFilesystemUsage> for FilesystemUsage {
    fn from(filesystem_usage: &config::CheckFilesystemUsage) -> Self {
        Self {
            mountpoints: filesystem_usage.mountpoints.clone(),
        }
    }
}

impl MeasurementIds for FilesystemUsage {
    fn measurement_ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}

impl LevelSource for FilesystemUsage {
    fn get_levels(&self) -> Vec<(String, u8)> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            let stat = nix::sys::statvfs::statvfs(&mountpoint[..]).unwrap();
            let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
            res.push((mountpoint.clone(), usage as u8));
        }
        res
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
    init_checks(&config);
}
