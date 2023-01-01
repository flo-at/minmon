use crate::{Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;

pub struct StatusCode {
    status_codes: Vec<u8>,
}

impl TryFrom<&config::Alarm> for StatusCode {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, Self::Error> {
        match &alarm.type_ {
            config::AlarmType::StatusCode(status_code) => Ok(Self {
                status_codes: status_code.status_codes.clone(),
            }),
            config::AlarmType::Default(_) => Ok(Self {
                status_codes: vec![0],
            }),
            _ => Err(Error(String::from("Expected status code alarm config."))),
        }
    }
}

impl DataSink for StatusCode {
    type Item = u8;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        Ok(if self.status_codes.contains(data) {
            SinkDecision::Good
        } else {
            SinkDecision::Bad
        })
    }

    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("status_code"), data.to_string());
    }
}
