use crate::{Error, Result};

use super::{DataSink, SinkDecision};
use crate::config;

pub struct Level {
    level: u8,
}

impl TryFrom<&config::Alarm> for Level {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, self::Error> {
        if let config::AlarmType::Level(level) = &alarm.type_ {
            Ok(Self { level: level.level })
        } else {
            panic!();
        }
    }
}

impl DataSink for Level {
    type Item = u8;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        if *data >= self.level {
            Ok(SinkDecision::Bad)
        } else {
            Ok(SinkDecision::Good)
        }
    }

    fn format_data(data: &Self::Item) -> String {
        format!("level {}", data)
    }
}
