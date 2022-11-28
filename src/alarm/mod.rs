use crate::action;
use crate::config;
use crate::ActionMap;
use crate::{PlaceholderMap, Result};
use async_trait::async_trait;

mod level;

pub use level::Level;

#[async_trait]
pub trait Alarm: Send + Sync {
    type Item: Send + Sync;

    fn new(measurement_id: &str, alarm: &config::Alarm, actions: &ActionMap) -> Self;
    async fn put_data(&mut self, data: &Self::Item, mut placeholders: PlaceholderMap)
        -> Result<()>;
}

pub struct AlarmBase {
    name: String,
    id: String,
    action: Option<std::sync::Arc<dyn action::Action>>,
    cycles: u32,
    repeat_cycles: u32,
    recover_action: Option<std::sync::Arc<dyn action::Action>>,
    recover_cycles: u32,
    placeholders: PlaceholderMap,
    // --
    bad_cycles: u32,
    good_cycles: u32,
    good: bool,
    // TODO UUID, timestamp
}

impl AlarmBase {
    async fn bad(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        self.good_cycles = 0;
        self.bad_cycles += 1;
        if self.bad_cycles >= self.cycles {
            let good_old = self.good;
            self.good = false;
            if good_old
                || (self.repeat_cycles > 0
                    && (self.bad_cycles - self.cycles) % self.repeat_cycles == 0)
            {
                self.trigger(placeholders).await?;
            }
        }
        Ok(())
    }

    async fn good(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        self.bad_cycles = 0;
        self.good_cycles += 1;
        if self.good_cycles == self.recover_cycles {
            let good_old = self.good;
            self.good = true;
            if !good_old {
                self.trigger_recover(placeholders).await?;
            }
        }
        Ok(())
    }

    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        placeholders = self.add_placeholders(placeholders)?;
        match &self.action {
            Some(action) => {
                log::debug!("Action '{}' triggered.", self.name);
                action.trigger(placeholders).await
            }
            None => {
                log::debug!("Action '{}' was triggered but is disabled.", self.name);
                Ok(())
            }
        }
    }

    async fn trigger_recover(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        placeholders = self.add_placeholders(placeholders)?;
        match &self.recover_action {
            Some(action) => action.trigger(placeholders).await,
            None => Ok(()),
        }
    }

    fn add_placeholders(&self, mut placeholders: PlaceholderMap) -> Result<PlaceholderMap> {
        placeholders.insert(String::from("alarm_name"), self.name.clone());
        for (key, value) in self.placeholders.iter() {
            placeholders.insert(key.clone(), value.clone());
        }
        Ok(placeholders)
    }

    pub fn new(measurement_id: &str, alarm: &config::Alarm, actions: &ActionMap) -> Self {
        Self {
            name: alarm.name.clone(),
            id: measurement_id.to_string(),
            action: actions.get(&alarm.action).cloned(),
            cycles: alarm.cycles,
            repeat_cycles: alarm.repeat_cycles,
            recover_action: actions.get(&alarm.recover_action).cloned(),
            recover_cycles: alarm.recover_cycles,
            placeholders: alarm.placeholders.clone(),
            bad_cycles: 0,
            good_cycles: 0,
            good: true,
        }
    }
}
