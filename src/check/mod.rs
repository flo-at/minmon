use crate::action;
use crate::alarm;
use crate::alarm::{Alarm, AlarmBase, DataSink};
use crate::config;
use crate::ActionMap;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

mod filesystem_usage;
mod memory_usage;

#[async_trait]
pub trait Check: Send + Sync {
    async fn trigger(&mut self) -> Result<()>;
    fn interval(&self) -> std::time::Duration;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait DataSource: Send + Sync {
    type Item: Send + Sync;

    async fn get_data(&self) -> Result<Vec<Result<Self::Item>>>;
    fn ids(&self) -> &[String];
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
    ) -> Result<Self> {
        if interval == 0 {
            Err(Error(String::from("'interval' cannot be 0.")))
        } else if name.is_empty() {
            Err(Error(String::from("'name' cannot be empty.")))
        } else {
            Ok(Self {
                interval,
                name,
                placeholders,
                data_source,
                alarms,
            })
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
        let mut placeholders = self.placeholders.clone();
        placeholders.insert(String::from("check_name"), self.name.clone());
        let data_vec = self
            .data_source
            .get_data()
            .await
            .map_err(|x| Error(format!("Failed to get data: {}", x)))?;
        for (data, alarms) in data_vec.iter().zip(self.alarms.iter_mut()) {
            for alarm in alarms.iter_mut() {
                let mut placeholders = placeholders.clone();
                let result = match data {
                    Ok(data) => alarm.put_data(data, placeholders).await,
                    Err(err) => {
                        placeholders.insert(String::from("check_error"), format!("{}", err));
                        alarm.put_error(err, placeholders).await
                    }
                };
                if let Err(err) = result {
                    log::error!(
                        "Error in alarm '{}' from check '{}': {}",
                        alarm.name(),
                        self.name,
                        err
                    );
                }
            }
        }
        Ok(())
    }

    fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.interval.into())
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

fn factory<'a, T, U>(check_config: &'a config::Check, actions: &ActionMap) -> Result<Box<dyn Check>>
where
    T: DataSource + TryFrom<&'a config::Check, Error = Error> + 'static,
    U: DataSink<Item = T::Item> + TryFrom<&'a config::Alarm, Error = Error> + 'static,
{
    let data_source = T::try_from(check_config)?;
    let mut all_alarms: Vec<Vec<AlarmBase<U>>> = Vec::new();
    for (i, id) in data_source.ids().iter().enumerate() {
        let mut alarms: Vec<AlarmBase<U>> = Vec::new();
        for alarm_config in check_config.alarms.iter() {
            if alarm_config.disable {
                log::info!("Alarm '{}' is disabled.", alarm_config.name);
                continue;
            }
            if i == 0 {
                log::info!(
                "Alarm '{}' will be triggered after {} bad cycles and recover after {} good cycles.",
                alarm_config.name,
                alarm_config.cycles,
                alarm_config.recover_cycles
            );
            }
            let data_sink = U::try_from(alarm_config)?;
            let alarm_state_machine = alarm::StateMachine::new(
                alarm_config.cycles,
                alarm_config.repeat_cycles,
                alarm_config.recover_cycles,
                alarm_config.error_repeat_cycles,
            )?;
            let alarm = alarm::AlarmBase::new(
                alarm_config.name.clone(),
                id.clone(),
                action::get_action(&alarm_config.action, actions)?,
                alarm_config.placeholders.clone(),
                match &alarm_config.recover_action {
                    Some(action) => Some(action::get_action(action, actions)?),
                    None => None,
                },
                alarm_config.recover_placeholders.clone(),
                match &alarm_config.error_action {
                    Some(action) => Some(action::get_action(action, actions)?),
                    None => None,
                },
                alarm_config.error_placeholders.clone(),
                alarm_config.invert,
                alarm_state_machine,
                data_sink,
            )?;
            alarms.push(alarm);
        }
        all_alarms.push(alarms);
    }
    Ok(Box::new(CheckBase::new(
        check_config.interval,
        check_config.name.clone(),
        check_config.placeholders.clone(),
        data_source,
        all_alarms,
    )?))
}

pub fn from_check_config(
    check_config: &config::Check,
    actions: &ActionMap,
) -> Result<Box<dyn Check>> {
    match &check_config.type_ {
        // NOTE Add mapping here when implementing new data source / alarms.
        config::CheckType::FilesystemUsage(_) => {
            factory::<filesystem_usage::FilesystemUsage, alarm::Level>(check_config, actions)
        }
        config::CheckType::MemoryUsage(_) => {
            factory::<memory_usage::MemoryUsage, alarm::Level>(check_config, actions)
        }
    }
    .map_err(|x| {
        Error(format!(
            "Failed to create check '{}' from config: {}",
            check_config.name, x
        ))
    })
}
