use crate::{Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;

pub struct Temperature {
    temperature: i16,
}

impl TryFrom<&config::Alarm> for Temperature {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, self::Error> {
        if let config::AlarmType::Temperature(temperature) = &alarm.type_ {
            if temperature.temperature < -273 {
                Err(Error(String::from(
                    "'temperature' cannot be less than -273°C.",
                )))
            } else {
                Ok(Self {
                    temperature: temperature.temperature,
                })
            }
        } else {
            Err(Error(String::from("Expected temperature alarm config.")))
        }
    }
}

impl DataSink for Temperature {
    type Item = i16;

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
