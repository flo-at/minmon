use crate::{Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;

pub struct Level {
    level: u8,
}

impl TryFrom<&config::Alarm> for Level {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, self::Error> {
        #[allow(irrefutable_let_patterns)] // there are no other types yet
        if let config::AlarmType::Level(level) = &alarm.type_ {
            if level.level > 100 {
                Err(Error(String::from("'level' cannot be greater than 100.")))
            } else {
                Ok(Self { level: level.level })
            }
        } else {
            panic!();
        }
    }
}

impl DataSink for Level {
    type Item = u8;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        Ok(if *data > self.level {
            SinkDecision::Bad
        } else {
            SinkDecision::Good
        })
    }

    fn format_data(data: &Self::Item) -> String {
        format!("level {}", data)
    }

    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("level"), format!("{}", data));
    }
}
