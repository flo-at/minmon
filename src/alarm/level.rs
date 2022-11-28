use crate::{PlaceholderMap, Result};
use async_trait::async_trait;

use super::config;
use super::{Alarm, AlarmBase};
use crate::ActionMap; // TODO move

pub struct Level {
    alarm: AlarmBase,
    level: u8,
}

#[async_trait]
impl Alarm for Level {
    type Item = u8;

    fn new(measurement_id: &str, alarm: &config::Alarm, actions: &ActionMap) -> Self {
        if let config::AlarmType::Level(level) = &alarm.type_ {
            Self {
                alarm: AlarmBase::new(measurement_id, alarm, actions),
                level: level.level,
            }
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
        // TODO pass on placeholders to bad() and good()
        if *data >= self.level {
            self.alarm.bad(placeholders).await?;
        } else {
            self.alarm.good(placeholders).await?;
        }
        Ok(())
    }
}
