use crate::action;
use crate::alarm;
use crate::alarm::{Alarm, AlarmBase, DataSink};
use crate::config;
use crate::filter;
use crate::filter::FilterFactory;
use crate::measurement;
use crate::ActionMap;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

#[cfg(feature = "docker")]
mod docker_container_status;
mod filesystem_usage;
mod memory_usage;
mod network_throughput;
mod pressure_average;
mod process_exit_status;
mod systemd_unit_status;
#[cfg(feature = "sensors")]
mod temperature;

#[async_trait]
pub trait Check: Send + Sync {
    async fn trigger(&mut self);
    fn interval(&self) -> std::time::Duration;
    fn name(&self) -> &str;
}

#[async_trait]
pub trait DataSource: Send + Sync {
    type Item: Send + Sync + measurement::Measurement;

    async fn get_data(
        &mut self,
        placeholders: &mut PlaceholderMap,
    ) -> Result<Vec<Result<Option<Self::Item>>>>;
    fn format_data(&self, data: &Self::Item) -> String;
    fn ids(&self) -> &[String];
}

pub struct CheckBase<T, U>
where
    T: DataSource,
    U: Alarm,
{
    interval: std::time::Duration,
    name: String,
    timeout: std::time::Duration,
    placeholders: PlaceholderMap,
    filter: Option<Vec<Box<dyn filter::Filter<T::Item>>>>,
    data_source: T,
    alarms: Vec<Vec<U>>,
}

impl<T, U> CheckBase<T, U>
where
    T: DataSource,
    U: Alarm<Item = T::Item>,
{
    fn new(
        interval: std::time::Duration,
        name: String,
        timeout: Option<std::time::Duration>,
        placeholders: PlaceholderMap,
        filter: Option<Vec<Box<dyn filter::Filter<T::Item>>>>,
        data_source: T,
        alarms: Vec<Vec<U>>,
    ) -> Result<Self> {
        if interval.is_zero() {
            Err(Error(String::from("'interval' cannot be 0.")))
        } else if name.is_empty() {
            Err(Error(String::from("'name' cannot be empty.")))
        } else if matches!(timeout, Some(timeout) if timeout.is_zero()) {
            Err(Error(String::from("'timeout' cannot be 0.")))
        } else if matches!(timeout, Some(timeout) if timeout > interval) {
            Err(Error(String::from(
                "'timeout' cannot be greater than 'interval'.",
            )))
        } else {
            let timeout = timeout.unwrap_or_else(|| {
                interval.min(std::time::Duration::from_secs(
                    config::default::check_timeout().into(),
                ))
            });
            Ok(Self {
                interval,
                name,
                timeout,
                placeholders,
                filter,
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
    async fn trigger(&mut self) {
        let mut placeholders = crate::global_placeholders();
        crate::merge_placeholders(&mut placeholders, &self.placeholders);
        placeholders.insert(String::from("check_name"), self.name.clone());
        let res =
            tokio::time::timeout(self.timeout, self.data_source.get_data(&mut placeholders)).await;
        let ids = self.data_source.ids();
        let data_vec = match res {
            Ok(inner) => inner,
            Err(_) => Err(Error(format!(
                "Timed out after {} seconds.",
                self.timeout.as_secs()
            ))),
        };
        let mut data_vec = data_vec.unwrap_or_else(|x| {
            let mut res = Vec::new();
            for _ in 0..ids.len() {
                res.push(Err(x.clone()))
            }
            res
        });
        if let Some(filter) = &mut self.filter {
            data_vec = data_vec
                .into_iter()
                .zip(filter.iter_mut())
                .map(|(data, filter)| match data {
                    Ok(Some(data)) => Ok(Some(filter.filter(data))),
                    Ok(None) => Ok(None),
                    Err(x) => {
                        filter.error();
                        Err(x)
                    }
                })
                .collect();
        }
        for ((i, data), alarms) in data_vec.iter().enumerate().zip(self.alarms.iter_mut()) {
            match data {
                Ok(data) => match data {
                    Some(data) => log::debug!(
                        "Check '{}' got {} for id '{}'.",
                        self.name,
                        self.data_source.format_data(data),
                        ids[i]
                    ),
                    None => log::debug!("Check '{}' for id '{}' is warming up.", self.name, ids[i]),
                },
                Err(err) => log::warn!(
                    "Check '{}' got no data for id '{}': {}",
                    self.name,
                    ids[i],
                    err
                ),
            }
            for alarm in alarms.iter_mut() {
                let mut placeholders = placeholders.clone();
                let result = match data {
                    Ok(data) => match data {
                        Some(data) => alarm.put_data(data, placeholders).await,
                        None => Ok(()),
                    },
                    Err(err) => {
                        placeholders.insert(String::from("check_error"), err.to_string());
                        alarm.put_error(err, placeholders).await
                    }
                };
                if let Err(err) = result {
                    log::error!("{} had an error: {}", alarm.log_id(), err);
                }
            }
        }
    }

    fn interval(&self) -> std::time::Duration {
        self.interval
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
}

fn factory<'a, T, U>(check_config: &'a config::Check, actions: &ActionMap) -> Result<Box<dyn Check>>
where
    T: DataSource + TryFrom<&'a config::Check, Error = Error> + 'static,
    U: DataSink<Item = T::Item> + TryFrom<&'a config::Alarm, Error = Error> + 'static,
    T::Item: filter::FilterFactory,
{
    let data_source = T::try_from(check_config)?;
    let mut all_alarms: Vec<Vec<AlarmBase<U>>> = Vec::new();
    for (i, id) in data_source.ids().iter().enumerate() {
        let mut alarms: Vec<AlarmBase<U>> = Vec::new();
        let mut used_names = std::collections::HashSet::new();
        for alarm_config in check_config.alarms.iter() {
            if !used_names.insert(alarm_config.name.clone()) {
                return Err(Error(format!(
                    "Found duplicate alarm name '{}' for check '{}'.",
                    alarm_config.name, check_config.name
                )));
            }
            let alarm_log_id = format!(
                "Alarm '{}', id '{}' from check '{}'",
                alarm_config.name, id, check_config.name
            );
            if alarm_config.disable {
                log::info!("{} is disabled.", alarm_log_id);
                continue;
            }
            if i == 0 {
                log::info!(
                    "Alarm '{}' from check '{}' will be triggered after {} bad cycles and recover after {} good cycles.",
                    alarm_config.name,
                    check_config.name,
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
                alarm_log_id.clone(),
            )?;
            let alarm = alarm::AlarmBase::new(
                alarm_config.name.clone(),
                id.clone(),
                action::get_action(&alarm_config.action, actions)?,
                alarm_config.placeholders.clone(),
                check_config
                    .filter
                    .as_ref()
                    .map(T::Item::filter_factory)
                    .transpose()?,
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
                match &alarm_config.error_recover_action {
                    Some(action) => Some(action::get_action(action, actions)?),
                    None => None,
                },
                alarm_config.error_recover_placeholders.clone(),
                alarm_config.invert,
                alarm_state_machine,
                data_sink,
                alarm_log_id,
            )?;
            alarms.push(alarm);
        }
        all_alarms.push(alarms);
    }
    let filter = check_config
        .filter
        .as_ref()
        .map(|x| {
            (0..data_source.ids().len())
                .map(|_| T::Item::filter_factory(x))
                .collect()
        })
        .transpose()?;
    Ok(Box::new(CheckBase::new(
        std::time::Duration::from_secs(check_config.interval.into()),
        check_config.name.clone(),
        check_config
            .timeout
            .map(|x| std::time::Duration::from_secs(x.into())),
        check_config.placeholders.clone(),
        filter,
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
        #[cfg(feature = "docker")]
        config::CheckType::DockerContainerStatus(_) => factory::<
            docker_container_status::DockerContainerStatus,
            alarm::BinaryState,
        >(check_config, actions),
        config::CheckType::FilesystemUsage(_) => {
            factory::<filesystem_usage::FilesystemUsage, alarm::Level>(check_config, actions)
        }
        config::CheckType::MemoryUsage(_) => {
            factory::<memory_usage::MemoryUsage, alarm::Level>(check_config, actions)
        }
        config::CheckType::NetworkThroughput(_) => {
            factory::<network_throughput::NetworkThroughput, alarm::DataSize>(check_config, actions)
        }
        config::CheckType::PressureAverage(_) => {
            factory::<pressure_average::PressureAverage, alarm::Level>(check_config, actions)
        }
        config::CheckType::ProcessExitStatus(_) => factory::<
            process_exit_status::ProcessExitStatus,
            alarm::StatusCode,
        >(check_config, actions),
        config::CheckType::SystemdUnitStatus(_) => factory::<
            systemd_unit_status::SystemdUnitStatus,
            alarm::BinaryState,
        >(check_config, actions),
        #[cfg(feature = "sensors")]
        config::CheckType::Temperature(_) => {
            factory::<temperature::Temperature, alarm::Temperature>(check_config, actions)
        }
    }
    .map_err(|x| {
        Error(format!(
            "Failed to create check '{}' from config: {}",
            check_config.name, x
        ))
    })
}
