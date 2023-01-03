use crate::measurement::Measurement;
use crate::{measurement, Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;

type Item = measurement::Level;

pub struct Level {
    level: Item,
}

impl TryFrom<&config::Alarm> for Level {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, Self::Error> {
        if let config::AlarmType::Level(level) = &alarm.type_ {
            Ok(Self {
                level: Item::new(level.level)?,
            })
        } else {
            Err(Error(String::from("Expected level alarm config.")))
        }
    }
}

impl DataSink for Level {
    type Item = Item;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        Ok(if *data > self.level {
            SinkDecision::Bad
        } else {
            SinkDecision::Good
        })
    }

    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("level"), data.to_string());
    }
}
