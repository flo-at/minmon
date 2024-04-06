use std::str::FromStr;

use crate::action;
use crate::config;
use crate::ActionMap;
use crate::{Error, PlaceholderMap, Result};

pub struct Report {
    pub when: ReportWhen,
    placeholders: PlaceholderMap,
    events: Vec<Event>,
}

#[derive(Clone)]
pub enum ReportWhen {
    Interval(std::time::Duration),
    Cron(cron::Schedule),
}

impl Report {
    fn new(
        when: &config::ReportWhen,
        placeholders: PlaceholderMap,
        events: Vec<Event>,
    ) -> Result<Self> {
        if when.interval.is_some() && when.cron.is_some() {
            Err(Error(String::from(
                "'interval' and 'cron' cannot be set both.",
            )))
        } else if let Some(0) = when.interval {
            Err(Error(String::from("'interval' cannot be 0.")))
        } else {
            let when = if let Some(interval) = when.interval {
                ReportWhen::Interval(std::time::Duration::from_secs(interval.into()))
            } else if let Some(cron) = when.cron.as_ref() {
                let schedule = cron::Schedule::from_str(cron).map_err(|x| Error(x.to_string()))?;
                ReportWhen::Cron(schedule)
            } else {
                ReportWhen::Interval(std::time::Duration::from_secs(
                    config::default::report_interval().into(),
                ))
            };
            Ok(Self {
                when,
                placeholders,
                events,
            })
        }
    }

    pub async fn trigger(&mut self) {
        let mut placeholders = crate::global_placeholders();
        crate::merge_placeholders(&mut placeholders, &self.placeholders);
        for event in self.events.iter_mut() {
            let result = event.trigger(placeholders.clone()).await;
            if let Err(err) = result {
                log::error!("Error in report event '{}': {}", event.name, err);
            }
        }
    }
}

struct Event {
    name: String,
    placeholders: PlaceholderMap,
    action: std::sync::Arc<dyn action::Action>,
}

impl Event {
    fn new(
        name: String,
        placeholders: PlaceholderMap,
        action: std::sync::Arc<dyn action::Action>,
    ) -> Result<Self> {
        if name.is_empty() {
            Err(Error(String::from("'name' cannot be empty.")))
        } else {
            Ok(Self {
                name,
                placeholders,
                action,
            })
        }
    }

    fn add_placeholders(&self, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("event_name"), self.name.clone());
        crate::merge_placeholders(placeholders, &self.placeholders);
    }

    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        self.add_placeholders(&mut placeholders);
        self.action.trigger(placeholders).await
    }
}

pub fn from_report_config(report_config: &config::Report, actions: &ActionMap) -> Result<Report> {
    let mut events: Vec<Event> = Vec::new();
    let mut used_names = std::collections::HashSet::new();
    for event_config in report_config.events.iter() {
        if !used_names.insert(event_config.name.clone()) {
            return Err(Error(format!(
                "Found duplicate event name: {}",
                event_config.name
            )));
        }
        let event = Event::new(
            event_config.name.clone(),
            event_config.placeholders.clone(),
            action::get_action(&event_config.action, actions)?,
        )?;
        events.push(event);
    }
    Report::new(
        &report_config.when,
        report_config.placeholders.clone(),
        events,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_report_validation() {
        let invalid_interval = config::ReportWhen {
            interval: Some(0),
            ..Default::default()
        };
        assert!(matches!(
            Report::new(&invalid_interval, PlaceholderMap::new(), Vec::new()),
            Err(Error(_))
        ));
    }
}
