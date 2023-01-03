use crate::{measurement, Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;
use crate::measurement::Measurement;

type Item = measurement::Temperature;

pub struct Temperature {
    temperature: Item,
}

impl TryFrom<&config::Alarm> for Temperature {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, Self::Error> {
        if let config::AlarmType::Temperature(temperature) = &alarm.type_ {
            Ok(Self {
                temperature: Item::new(temperature.temperature)?,
            })
        } else {
            Err(Error(String::from("Expected temperature alarm config.")))
        }
    }
}

impl DataSink for Temperature {
    type Item = Item;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        Ok(if *data > self.temperature {
            SinkDecision::Bad
        } else {
            SinkDecision::Good
        })
    }

    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("temperature"), data.to_string());
    }
}
