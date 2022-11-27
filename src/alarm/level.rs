use crate::Result;
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
        if let config::AlarmType::Level(level_config) = &alarm.type_ {
            Self {
                alarm: AlarmBase::new(measurement_id, alarm, actions),
                level: level_config.level,
            }
        } else {
            panic!();
        }
    }

    async fn put_data(&mut self, data: &Self::Item) -> Result<()> {
        log::debug!(
            "Got level {} for alarm '{}' at id '{}'",
            data,
            self.alarm.name,
            self.alarm.id
        );
        if *data >= self.level {
            self.alarm.bad().await?;
        } else {
            self.alarm.good().await?;
        }
        Ok(())
    }
}
