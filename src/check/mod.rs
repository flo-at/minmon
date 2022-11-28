use crate::alarm;
use crate::alarm::Alarm;
use crate::config;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

mod filesystem_usage;
mod memory_usage;

use filesystem_usage::FilesystemUsage;
use memory_usage::MemoryUsage;

use crate::ActionMap;

#[async_trait]
pub trait Check: Send + Sync {
    async fn trigger(&mut self) -> Result<()>;
    fn interval(&self) -> std::time::Duration;
    fn report(&self) -> String;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait DataSource: Send + Sync {
    type Item: Send + Sync;

    async fn get_data(&self) -> Result<Vec<Self::Item>>;
    fn measurement_ids(&self) -> &[String];
}

pub struct CheckBase<T, U>
where
    T: DataSource,
    U: Alarm,
{
    interval: u32,
    name: String,
    placeholders: PlaceholderMap,
    data_source: T,
    alarms: Vec<Vec<U>>,
}

impl<T, U> CheckBase<T, U>
where
    T: DataSource,
    U: Alarm<Item = T::Item>,
{
    fn new(
        interval: u32,
        name: String,
        placeholders: PlaceholderMap,
        data_source: T,
        alarms: Vec<Vec<U>>,
    ) -> Self {
        Self {
            interval,
            name,
            placeholders,
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
    async fn trigger(&mut self) -> Result<()> {
        let data_vec = self
            .data_source
            .get_data()
            .await
            .map_err(|x| Error(format!("Failed to get data: {}", x)))?;
        for (data, alarms) in data_vec.iter().zip(self.alarms.iter_mut()) {
            for alarm in alarms.iter_mut() {
                alarm.put_data(data, self.placeholders.clone()).await?;
            }
        }
        Ok(())
    }

    fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.interval.into())
    }

    fn report(&self) -> String {
        format!("Check '{}' reporting in!", self.name) // TODO
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

fn factory<'a, T, U>(check_config: &'a config::Check, actions: &ActionMap) -> Result<Box<dyn Check>>
where
    T: DataSource + TryFrom<&'a config::Check, Error = Error> + 'static, // TODO warum 'static?
    U: Alarm<Item = T::Item> + 'static,                                  // TODO warum 'static?
{
    let data_source = T::try_from(check_config)?;
    let mut all_alarms: Vec<Vec<U>> = Vec::new();
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
            let level_alarm = U::new(measurement_id, alarm_config, actions); // TODO ?
            alarms.push(level_alarm);
        }
        all_alarms.push(alarms);
    }
    Ok(Box::new(CheckBase::new(
        check_config.interval,
        check_config.name.clone(),
        check_config.placeholders.clone(),
        data_source,
        all_alarms,
    )))
}

pub fn from_check_config(
    check_config: &config::Check,
    actions: &ActionMap,
) -> Result<Box<dyn Check>> {
    match &check_config.type_ {
        // NOTE Add mapping here when implementing new data source / alarms.
        config::CheckType::FilesystemUsage(_) => {
            factory::<FilesystemUsage, alarm::Level>(check_config, actions)
        }
        config::CheckType::MemoryUsage => {
            factory::<MemoryUsage, alarm::Level>(check_config, actions)
        }
    }
    .map_err(|x| {
        Error(format!(
            "Failed to create check '{}' from config: {}",
            check_config.name, x
        ))
    })
}
