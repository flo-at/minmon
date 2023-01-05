use crate::measurement::Measurement;
use crate::{measurement, Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;

type Item = measurement::BinaryState;

pub struct BinaryState {}

impl TryFrom<&config::Alarm> for BinaryState {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, Self::Error> {
        if let config::AlarmType::Default(_) = &alarm.type_ {
            Ok(Self {})
        } else {
            Err(Error(String::from("Did not expect any alarm config.")))
        }
    }
}

impl DataSink for BinaryState {
    type Item = Item;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        Ok(match data.data() {
            true => SinkDecision::Good,
            false => SinkDecision::Bad,
        })
    }

    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("state"), data.to_string());
    }
}
