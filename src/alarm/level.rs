use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

use super::config;
use super::{Alarm, AlarmBase};
use crate::ActionMap;

pub struct Level {
    alarm: AlarmBase,
    level: u8,
}

#[async_trait]
impl Alarm for Level {
    type Item = u8;

    fn new(measurement_id: &str, alarm: &config::Alarm, actions: &ActionMap) -> Result<Self> {
        if let config::AlarmType::Level(level) = &alarm.type_ {
            Ok(Self {
                alarm: AlarmBase::new(measurement_id, alarm, actions)?,
                level: level.level,
            })
        } else {
            panic!();
        }
    }

    async fn put_data(
        &mut self,
        data: &Self::Item,
        mut placeholders: PlaceholderMap,
    ) -> Result<()> {
        placeholders.insert(String::from("alarm_level"), format!("{}", data));
        log::debug!(
            "Got level {} for alarm '{}' at id '{}'",
            data,
            self.alarm.name,
            self.alarm.id
        );
        if *data >= self.level {
            self.alarm.bad(placeholders).await?;
        } else {
            self.alarm.good(placeholders).await?;
        }
        Ok(())
    }

    async fn put_error(&mut self, error: &Error, mut placeholders: PlaceholderMap) -> Result<()> {
        placeholders.insert(String::from("error"), format!("{}", error));
        self.alarm.error(placeholders).await
    }
}
