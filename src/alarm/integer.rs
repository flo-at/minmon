use crate::measurement::Measurement;
use crate::{measurement, Error, PlaceholderMap, Result};

use super::{DataSink, SinkDecision};
use crate::config;

type Item = measurement::Integer;

pub struct Integer {
    min: Option<Item>,
    max: Option<Item>,
}

impl TryFrom<&config::Alarm> for Integer {
    type Error = Error;

    fn try_from(alarm: &config::Alarm) -> std::result::Result<Self, Self::Error> {
        if let config::AlarmType::Integer(integer) = &alarm.type_ {
            if integer.min.is_none() && integer.max.is_none() {
                return Err(Error(String::from(
                    "At least one of 'min' or 'max' needs to be set",
                )));
            }
            Ok(Self {
                min: integer.min.map(Item::new).transpose()?,
                max: integer.max.map(Item::new).transpose()?,
            })
        } else {
            Err(Error(String::from("Expected integer alarm config.")))
        }
    }
}

impl DataSink for Integer {
    type Item = Item;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision> {
        Ok(
            if (self.min.is_some_and(|x| *data < x)) || (self.max.is_some_and(|x| *data > x)) {
                SinkDecision::Bad
            } else {
                SinkDecision::Good
            },
        )
    }

    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("integer"), data.to_string());
    }
}
