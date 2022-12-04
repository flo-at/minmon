use crate::action;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

mod level;

pub use level::Level;

fn iso8601(system_time: &std::time::SystemTime) -> String {
    let date_time: chrono::DateTime<chrono::Utc> = (*system_time).into();
    date_time.format("%FT%T").to_string()
}

pub trait DataSink: Send + Sync + Sized {
    type Item: Send + Sync;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision>;
    fn format_data(data: &Self::Item) -> String;
}

pub enum SinkDecision {
    Good,
    Bad,
}

#[async_trait]
pub trait Alarm: Send + Sync + Sized {
    type Item: Send + Sync;

    async fn put_data(&mut self, data: &Self::Item, mut placeholders: PlaceholderMap)
        -> Result<()>;
    async fn put_error(&mut self, error: &Error, mut placeholders: PlaceholderMap) -> Result<()>;
}

pub struct AlarmBase<T>
where
    T: DataSink,
{
    name: String,
    id: String,
    action: Option<std::sync::Arc<dyn action::Action>>,
    placeholders: PlaceholderMap,
    cycles: u32,
    repeat_cycles: u32,
    recover_action: Option<std::sync::Arc<dyn action::Action>>,
    recover_placeholders: PlaceholderMap,
    recover_cycles: u32,
    error_action: Option<std::sync::Arc<dyn action::Action>>,
    error_placeholders: PlaceholderMap,
    error_repeat_cycles: u32,
    state: State,
    data_sink: T,
}

#[derive(Clone)]
enum State {
    Good(GoodState),
    Bad(BadState),
    Error(ErrorState),
}

impl Default for State {
    fn default() -> Self {
        Self::Good(GoodState::default())
    }
}

#[derive(Clone)]
struct GoodState {
    timestamp: std::time::SystemTime,
    last_alarm_uuid: Option<String>,
    bad_cycles: u32,
}

impl Default for GoodState {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            last_alarm_uuid: None,
            bad_cycles: 0,
        }
    }
}

#[derive(Clone)]
struct BadState {
    timestamp: std::time::SystemTime,
    uuid: String,
    cycles: u32,
    good_cycles: u32,
}

#[derive(Clone)]
struct ErrorState {
    timestamp: std::time::SystemTime,
    uuid: String,
    shadowed_state: Box<State>,
    cycles: u32,
}

impl<T> AlarmBase<T>
where
    T: DataSink,
{
    pub fn new(
        name: String,
        id: String,
        action: Option<std::sync::Arc<dyn action::Action>>,
        placeholders: PlaceholderMap,
        cycles: u32,
        repeat_cycles: u32,
        recover_action: Option<std::sync::Arc<dyn action::Action>>,
        recover_placeholders: PlaceholderMap,
        recover_cycles: u32,
        error_action: Option<std::sync::Arc<dyn action::Action>>,
        error_placeholders: PlaceholderMap,
        error_repeat_cycles: u32,
        data_sink: T,
    ) -> Self {
        Self {
            name,
            id,
            action,
            placeholders,
            cycles,
            repeat_cycles,
            recover_action,
            recover_placeholders,
            recover_cycles,
            error_action,
            error_placeholders,
            error_repeat_cycles,
            state: State::default(),
            data_sink,
        }
    }

    fn error_update_state(&self, state: &State) -> (State, bool) {
        let mut trigger = false;
        let new_state = match state {
            State::Good(_) => {
                trigger = true;
                State::Error(ErrorState {
                    timestamp: std::time::SystemTime::now(),
                    uuid: uuid::Uuid::new_v4().to_string(),
                    shadowed_state: Box::new(state.clone()),
                    cycles: 1,
                })
            }

            State::Bad(_) => {
                trigger = true;
                State::Error(ErrorState {
                    timestamp: std::time::SystemTime::now(),
                    uuid: uuid::Uuid::new_v4().to_string(),
                    shadowed_state: Box::new(state.clone()),
                    cycles: 1,
                })
            }

            State::Error(error) => {
                let cycles = if error.cycles + 1 == self.error_repeat_cycles {
                    trigger = true;
                    1
                } else {
                    error.cycles + 1
                };
                State::Error(ErrorState {
                    timestamp: error.timestamp,
                    uuid: error.uuid.clone(),
                    shadowed_state: error.shadowed_state.clone(),
                    cycles,
                })
            }
        };
        (new_state, trigger)
    }

    fn bad_update_state(&mut self, state: &State) -> (State, bool) {
        let mut trigger = false;
        let new_state = match state {
            State::Good(good) => {
                if good.bad_cycles + 1 == self.cycles {
                    trigger = true;
                    State::Bad(BadState {
                        timestamp: std::time::SystemTime::now(),
                        uuid: uuid::Uuid::new_v4().to_string(),
                        cycles: 1,
                        good_cycles: 0,
                    })
                } else {
                    State::Good(GoodState {
                        timestamp: good.timestamp,
                        last_alarm_uuid: None,
                        bad_cycles: good.bad_cycles + 1,
                    })
                }
            }

            State::Bad(bad) => {
                let cycles = if bad.cycles + 1 == self.repeat_cycles {
                    trigger = true;
                    1
                } else {
                    bad.cycles + 1
                };
                State::Bad(BadState {
                    timestamp: bad.timestamp,
                    uuid: bad.uuid.clone(),
                    cycles,
                    good_cycles: 0,
                })
            }

            State::Error(error) => {
                self.state = *error.shadowed_state.clone();
                let (shadowed_state, shadowed_trigger) =
                    self.bad_update_state(&error.shadowed_state);
                trigger = shadowed_trigger;
                shadowed_state
            }
        };
        (new_state, trigger)
    }

    fn good_update_state(&mut self, state: &State) -> (State, bool) {
        let mut trigger = false;
        let new_state = match state {
            State::Good(good) => State::Good(good.clone()),

            State::Bad(bad) => {
                if bad.good_cycles + 1 == self.recover_cycles {
                    trigger = true;
                    State::Good(GoodState {
                        timestamp: std::time::SystemTime::now(),
                        last_alarm_uuid: Some(bad.uuid.clone()),
                        bad_cycles: 0,
                    })
                } else {
                    State::Bad(BadState {
                        timestamp: bad.timestamp,
                        uuid: bad.uuid.clone(),
                        cycles: bad.cycles + 1, // TODO unsure about this one
                        good_cycles: bad.good_cycles + 1,
                    })
                }
            }

            State::Error(error) => {
                self.state = *error.shadowed_state.clone();
                let (shadowed_state, shadowed_trigger) =
                    self.bad_update_state(&error.shadowed_state);
                trigger = shadowed_trigger;
                shadowed_state
            }
        };
        (new_state, trigger)
    }

    async fn error(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        let (new_state, trigger) = self.error_update_state(&self.state);
        self.state = new_state;
        if trigger {
            self.trigger_error(placeholders).await?;
        }
        Ok(())
    }

    async fn bad(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        let (new_state, trigger) = self.bad_update_state(&self.state.clone());
        self.state = new_state;
        if trigger {
            self.trigger(placeholders).await?;
        }
        Ok(())
    }

    async fn good(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        let (new_state, trigger) = self.good_update_state(&self.state.clone());
        self.state = new_state;
        if trigger {
            self.trigger_recover(placeholders).await?;
        }
        Ok(())
    }

    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        if let State::Bad(bad) = &self.state {
            self.add_placeholders(&mut placeholders);
            placeholders.insert(String::from("alarm_timestamp"), iso8601(&bad.timestamp));
            placeholders.insert(String::from("alarm_uuid"), bad.uuid.clone());
            placeholders.insert(String::from("alarm_timestamp"), iso8601(&bad.timestamp));
            match &self.action {
                Some(action) => {
                    log::debug!("Action 'TODO' for alarm '{}' triggered.", self.name);
                    action.trigger(placeholders).await
                }
                None => {
                    log::debug!(
                        "Action for alarm '{}' was triggered but is disabled.",
                        self.name
                    );
                    Ok(())
                }
            }
        } else {
            panic!();
        }
    }

    async fn trigger_recover(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        if let State::Good(good) = &self.state {
            self.add_placeholders(&mut placeholders);
            if let Some(last_alarm_uuid) = &good.last_alarm_uuid {
                placeholders.insert(String::from("alarm_uuid"), last_alarm_uuid.clone());
            }
            crate::merge_placeholders(&mut placeholders, &self.recover_placeholders);
            match &self.recover_action {
                Some(action) => action.trigger(placeholders).await,
                None => Ok(()),
            }
        } else {
            panic!();
        }
    }

    async fn trigger_error(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        if let State::Error(error) = &self.state {
            self.add_placeholders(&mut placeholders);
            // TODO if shadowed_state == Bad -> add bad uuid and timestamp
            placeholders.insert(String::from("error_uuid"), error.uuid.clone());
            placeholders.insert(String::from("error_timestamp"), iso8601(&error.timestamp));
            crate::merge_placeholders(&mut placeholders, &self.error_placeholders);
            match &self.error_action {
                Some(action) => action.trigger(placeholders).await,
                None => Ok(()),
            }
        } else {
            panic!();
        }
    }

    fn add_placeholders(&self, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("alarm_name"), self.name.clone());
        crate::merge_placeholders(placeholders, &self.placeholders);
    }
}

#[async_trait]
impl<T> Alarm for AlarmBase<T>
where
    T: DataSink,
{
    type Item = T::Item;

    async fn put_data(
        &mut self,
        data: &Self::Item,
        mut placeholders: PlaceholderMap,
    ) -> Result<()> {
        log::debug!(
            "Got {} for alarm '{}' at id '{}'",
            T::format_data(data),
            self.name,
            self.id
        );
        placeholders.insert(String::from("alarm_name"), self.name.clone());
        let decision = self.data_sink.put_data(data)?;
        match decision {
            SinkDecision::Good => self.good(placeholders).await,
            SinkDecision::Bad => self.bad(placeholders).await,
        }
    }

    async fn put_error(&mut self, error: &Error, mut placeholders: PlaceholderMap) -> Result<()> {
        log::debug!(
            "Got error for level alarm '{}' at id '{}': {}",
            self.name,
            self.id,
            error
        );
        placeholders.insert(String::from("alarm_name"), self.name.clone());
        self.error(placeholders).await
    }
}
