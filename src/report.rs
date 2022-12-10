use crate::action;
use crate::config;
use crate::ActionMap;
use crate::{Error, PlaceholderMap, Result};

pub struct Report {
    interval: u32,
    placeholders: PlaceholderMap,
    events: Vec<Event>,
}

impl Report {
    fn new(interval: u32, placeholders: PlaceholderMap, events: Vec<Event>) -> Result<Self> {
        if interval == 0 {
            Err(Error(String::from("'interval' cannot be 0.")))
        } else {
            Ok(Self {
                interval,
                placeholders,
                events,
            })
        }
    }

    pub async fn trigger(&mut self) -> Result<()> {
        for event in self.events.iter_mut() {
            let result = event.trigger(self.placeholders.clone()).await;
            if let Err(err) = result {
                log::error!("Error in report event: {}", err); // TODO add event name
            }
        }
        Ok(())
    }

    pub fn interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.interval.into())
    }
}

struct Event {
    name: String,
    placeholders: PlaceholderMap,
    action: Option<std::sync::Arc<dyn action::Action>>,
}

impl Event {
    fn new(
        name: String,
        placeholders: PlaceholderMap,
        action: Option<std::sync::Arc<dyn action::Action>>,
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
        placeholders.insert(String::from("report_event_name"), self.name.clone());
        crate::merge_placeholders(placeholders, &self.placeholders);
    }

    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        self.add_placeholders(&mut placeholders);
        match &self.action {
            Some(action) => {
                log::debug!("Action 'TODO' for report event '{}' triggered.", self.name);
                action.trigger(placeholders).await
            }
            None => {
                log::debug!(
                    "Action for report event '{}' was triggered but is disabled.",
                    self.name
                );
                Ok(())
            }
        }
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
        report_config.interval,
        report_config.placeholders.clone(),
        events,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_report_validation() {
        assert!(matches!(
            Report::new(0, PlaceholderMap::new(), Vec::new()),
            Err(Error(_))
        ));
    }
}
