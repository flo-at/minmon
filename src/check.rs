use crate::alarm;
use crate::alarm::DataSink;
use crate::config;

use std::collections::HashMap;

use crate::ActionMap;

pub trait MeasurementIds {
    fn measurement_ids(&self) -> &[String];
}

pub trait Report {
    fn trigger(&mut self) -> std::time::Duration;
    fn report(&self) -> String;
}

pub trait DataSource {
    type Item;
    fn get_data(&self) -> Vec<(String, Self::Item)>;
}

pub struct FilesystemUsage {
    mountpoints: Vec<String>, // TODO possible to store a reference?
}

impl From<&config::CheckFilesystemUsage> for FilesystemUsage {
    fn from(filesystem_usage: &config::CheckFilesystemUsage) -> Self {
        Self {
            mountpoints: filesystem_usage.mountpoints.clone(),
        }
    }
}

impl DataSource for FilesystemUsage {
    type Item = u8;

    fn get_data(&self) -> Vec<(String, Self::Item)> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            let stat = nix::sys::statvfs::statvfs(&mountpoint[..]).unwrap();
            let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
            res.push((mountpoint.clone(), usage as u8));
        }
        res
    }
}

impl MeasurementIds for FilesystemUsage {
    fn measurement_ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}

pub struct Check<T, U>
where
    T: DataSource,
    U: DataSink<Item = T::Item>,
{
    interval: u32,
    name: String,
    check_impl: T,
    alarms: HashMap<String, Vec<U>>,
}

impl<T, U> Check<T, U>
where
    T: DataSource,
    U: DataSink<Item = T::Item>,
{
    pub fn trigger(&mut self) {
        let data_vec = self.check_impl.get_data();
        for (id, data) in data_vec.iter() {
            let alarms = self.alarms.get_mut(id).unwrap();
            for alarm in alarms.iter_mut() {
                alarm.put_data(id, data);
            }
        }
    }

    fn new(interval: u32, name: String, check_impl: T, alarms: HashMap<String, Vec<U>>) -> Self {
        Self {
            interval,
            name,
            check_impl,
            alarms,
        }
    }
}

impl<T, U> Report for Check<T, U>
where
    T: DataSource,
    U: DataSink<Item = T::Item>,
{
    fn trigger(&mut self) -> std::time::Duration {
        self.trigger();
        std::time::Duration::from_secs(self.interval.into())
    }

    fn report(&self) -> String {
        format!("Check '{}' reporting in!", self.name) // TODO
    }
}

pub fn from_check_config(check_config: &config::Check, actions: &ActionMap) -> Box<dyn Report> {
    match &check_config.type_ {
        config::CheckType::FilesystemUsage(filesystem_usage_config) => {
            let level_check_impl = FilesystemUsage::from(filesystem_usage_config);
            let mut level_alarm_map: HashMap<String, Vec<alarm::Level>> = HashMap::new();
            for measurement_id in level_check_impl.measurement_ids().iter() {
                let mut level_alarms: Vec<alarm::Level> = Vec::new();
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
                    let level_alarm = alarm::Level::from(alarm_config);
                    level_alarms.push(level_alarm);
                }
                level_alarm_map.insert(measurement_id.clone(), level_alarms);
            }
            Box::new(Check::new(
                check_config.interval,
                check_config.name.clone(),
                level_check_impl,
                level_alarm_map,
            ))
        }
    }
}
