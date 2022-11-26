use crate::alarm;
use crate::alarm::Alarm;
use crate::config;
use async_trait::async_trait;

mod filesystem_usage;

pub use filesystem_usage::FilesystemUsage;

use std::collections::HashMap;

use crate::ActionMap;

#[async_trait]
pub trait Check: Send + Sync {
    async fn trigger(&mut self);
    fn interval(&self) -> std::time::Duration;
    fn report(&self) -> String; // TODO implement
    fn name(&self) -> &str;
}

pub trait DataSource: Send + Sync {
    type Item: Send + Sync;

    fn validate(&self) -> bool; // TODO call validate early on
    fn get_data(&self) -> Vec<(String, Self::Item)>;
    fn measurement_ids(&self) -> &[String];
}

pub struct CheckBase<T, U>
where
    T: DataSource,
    U: Alarm,
{
    interval: u32,
    name: String,
    data_source: T,
    alarms: HashMap<String, Vec<U>>, // TODO Vec<Vec<U>> should also work
}

impl<T, U> CheckBase<T, U>
where
    T: DataSource,
    U: Alarm<Item = T::Item>,
{
    fn new(interval: u32, name: String, data_source: T, alarms: HashMap<String, Vec<U>>) -> Self {
        Self {
            interval,
            name,
            data_source,
            alarms,
        }
    }
}

#[async_trait]
impl<T, U> Check for CheckBase<T, U>
where
    T: DataSource,
    U: Alarm<Item = T::Item>,
{
    async fn trigger(&mut self) {
        let data_vec = self.data_source.get_data();
        for (id, data) in data_vec.iter() {
            let alarms = self.alarms.get_mut(id).unwrap();
            for alarm in alarms.iter_mut() {
                alarm.put_data(id, data).await;
            }
        }
    }

    fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.interval.into())
    }

    fn report(&self) -> String {
        format!("Check '{}' reporting in!", self.name) // TODO
    }

    fn name(&self) -> &str {
        &self.name[..]
    }
}

fn factory<'a, T, U>(check_config: &'a config::Check, actions: &ActionMap) -> Box<dyn Check>
where
    T: DataSource + From<&'a config::Check> + 'static, // TODO warum 'static?
    U: Alarm<Item = T::Item> + 'static,                // TODO warum 'static?
{
    let data_source = T::from(check_config);
    let mut alarm_map: HashMap<String, Vec<U>> = HashMap::new();
    for measurement_id in data_source.measurement_ids().iter() {
        let mut alarms: Vec<U> = Vec::new();
        for alarm_config in check_config.alarms.iter() {
            if alarm_config.disable {
                log::info!("Alarm '{}' is disabled.", alarm_config.name);
                continue;
            }
            log::info!(
                "Alarm '{}' will be triggered after {} bad cycles and recover after {} good cycles.",
                alarm_config.name,
                alarm_config.cycles,
                alarm_config.recover_cycles
            );
            let level_alarm = U::new(alarm_config, actions);
            alarms.push(level_alarm);
        }
        alarm_map.insert(measurement_id.clone(), alarms);
    }
    Box::new(CheckBase::new(
        check_config.interval,
        check_config.name.clone(),
        data_source,
        alarm_map,
    ))
}

pub fn from_check_config(check_config: &config::Check, actions: &ActionMap) -> Box<dyn Check> {
    match &check_config.type_ {
        config::CheckType::FilesystemUsage(_) => {
            factory::<FilesystemUsage, alarm::Level>(check_config, actions)
        } // TODO add mapping here when implementing new data source / alarms
    }
}
