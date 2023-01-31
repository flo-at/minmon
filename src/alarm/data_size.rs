use crate::{measurement, Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;
use crate::measurement::Measurement;

type Item = measurement::DataSize;

pub struct DataSize {
    data_size: Item,
}

impl TryFrom<&config::Alarm> for DataSize {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, Self::Error> {
        if let config::AlarmType::DataSize(data_size) = &alarm.type_ {
            Ok(Self {
                data_size: Item::new(data_size.bytes())?,
            })
        } else {
            Err(Error(String::from("Expected data size alarm config.")))
        }
    }
}

impl DataSink for DataSize {
    type Item = Item;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        Ok(if *data > self.data_size {
            SinkDecision::Bad
        } else {
            SinkDecision::Good
        })
    }

    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("data_size"), data.to_string());
        placeholders.insert(String::from("data_size_bin"), data.as_string_binary());
        placeholders.insert(String::from("data_size_dec"), data.as_string_decimal());
    }
}
