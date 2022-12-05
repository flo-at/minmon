use crate::action;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

mod level;

pub use level::Level;

#[cfg_attr(test, mockall::automock(type Item=u8;))]
pub trait DataSink: Send + Sync + Sized {
    type Item: Send + Sync;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision>;
    fn format_data(data: &Self::Item) -> String;
    fn add_placeholders(data: &Self::Item, placeholders: &mut PlaceholderMap);
}

pub enum SinkDecision {
    Good,
    Bad,
}

impl std::ops::Not for SinkDecision {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            SinkDecision::Good => SinkDecision::Bad,
            SinkDecision::Bad => SinkDecision::Good,
        }
    }
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
    invert: bool,
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
    last_alarm: Option<BadState>,
    bad_cycles: u32,
}

impl Default for GoodState {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            last_alarm: None,
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
        invert: bool,
        data_sink: T,
    ) -> Self {
        // TODO ensure cycles != 0 and recover_cycles != 0
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
            invert,
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
                let cycles = if error.cycles == self.error_repeat_cycles {
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
                        last_alarm: None,
                        bad_cycles: good.bad_cycles + 1,
                    })
                }
            }

            State::Bad(bad) => {
                let cycles = if bad.cycles == self.repeat_cycles {
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
            State::Good(good) => State::Good(good.clone()), // TODO maybe unset last_alarm

            State::Bad(bad) => {
                if bad.good_cycles + 1 == self.recover_cycles {
                    trigger = true;
                    State::Good(GoodState {
                        timestamp: std::time::SystemTime::now(),
                        last_alarm: Some(bad.clone()),
                        bad_cycles: 0,
                    })
                } else {
                    State::Bad(BadState {
                        timestamp: bad.timestamp,
                        uuid: bad.uuid.clone(),
                        cycles: bad.cycles + 1,
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
            placeholders.insert(
                String::from("alarm_timestamp"),
                crate::iso8601(bad.timestamp),
            );
            placeholders.insert(String::from("alarm_uuid"), bad.uuid.clone());
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
            if let Some(last_alarm) = &good.last_alarm {
                placeholders.insert(String::from("alarm_uuid"), last_alarm.uuid.clone());
                placeholders.insert(
                    String::from("alarm_timestamp"),
                    crate::iso8601(last_alarm.timestamp),
                );
            } else {
                panic!();
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
            // TODO add info about shadowed_state (add bad uuid and timestamp, ..)
            placeholders.insert(String::from("error_uuid"), error.uuid.clone());
            placeholders.insert(
                String::from("error_timestamp"),
                crate::iso8601(error.timestamp),
            );
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
        placeholders.insert(String::from("alarm_id"), self.id.clone());
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
        T::add_placeholders(data, &mut placeholders);
        self.add_placeholders(&mut placeholders);
        let mut decision = self.data_sink.put_data(data)?;
        if self.invert {
            decision = !decision;
        }
        match decision {
            SinkDecision::Good => self.good(placeholders).await,
            SinkDecision::Bad => self.bad(placeholders).await,
        }
    }

    async fn put_error(&mut self, error: &Error, mut placeholders: PlaceholderMap) -> Result<()> {
        log::debug!(
            "Got error for alarm '{}' at id '{}': {}",
            self.name,
            self.id,
            error
        );
        self.add_placeholders(&mut placeholders);
        self.error(placeholders).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::predicate::*;

    // TODO check if this is the "right way"
    //      https://docs.rs/mockall/latest/mockall/#static-methods
    // TODO test state shadowing
    static SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

    fn times_action(times: usize) -> Option<std::sync::Arc<dyn action::Action>> {
        let mut mock_action = action::MockAction::new();
        mock_action
            .expect_trigger()
            .times(times)
            .returning(|_| Ok(()));
        Some(std::sync::Arc::new(mock_action))
    }

    fn mock_data_sink() -> MockDataSink {
        let mut mock_data_sink = MockDataSink::new();
        mock_data_sink
            .expect_put_data()
            .with(eq(10))
            .returning(|_| Ok(SinkDecision::Good));
        mock_data_sink
            .expect_put_data()
            .with(eq(20))
            .returning(|_| Ok(SinkDecision::Bad));
        mock_data_sink
    }

    #[tokio::test]
    async fn test_trigger_action() {
        let _permit = SEMAPHORE.acquire().await.unwrap();
        let data_sink_ctx = MockDataSink::add_placeholders_context();
        data_sink_ctx
            .expect()
            .returning(|data: &u8, placeholders: &mut PlaceholderMap| {
                placeholders.insert(String::from("data"), format!("{}", data));
            });
        let mock_data_sink = mock_data_sink();
        let mut mock_action = action::MockAction::new();
        mock_action
            .expect_trigger()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                uuid::Uuid::parse_str(placeholders.get("alarm_uuid").unwrap()).unwrap();
                use std::str::FromStr;
                chrono::DateTime::<chrono::Utc>::from_str(
                    placeholders.get("alarm_timestamp").unwrap(),
                )
                .unwrap();
                assert_eq!(placeholders.get("alarm_name").unwrap(), "Name");
                assert_eq!(placeholders.get("alarm_id").unwrap(), "ID");
                assert_eq!(placeholders.get("Hello").unwrap(), "World");
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                assert_eq!(placeholders.get("data").unwrap(), "20");
                assert_eq!(placeholders.len(), 7);
                true
            }))
            .returning(|_| Ok(()));
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            Some(std::sync::Arc::new(mock_action)),
            PlaceholderMap::from([(String::from("Hello"), String::from("World"))]),
            1,
            0,
            times_action(0),
            PlaceholderMap::new(),
            1,
            times_action(0),
            PlaceholderMap::new(),
            0,
            false,
            mock_data_sink,
        );
        alarm
            .put_data(
                &20,
                PlaceholderMap::from([(String::from("Foo"), String::from("Bar"))]),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_trigger_action_repeat() {
        let _permit = SEMAPHORE.acquire().await.unwrap();
        let data_sink_ctx = MockDataSink::add_placeholders_context();
        data_sink_ctx
            .expect()
            .returning(|data: &u8, placeholders: &mut PlaceholderMap| {
                placeholders.insert(String::from("data"), format!("{}", data));
            });
        let mock_data_sink = mock_data_sink();
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            times_action(1),
            PlaceholderMap::new(),
            1,
            7,
            times_action(0),
            PlaceholderMap::new(),
            1,
            times_action(0),
            PlaceholderMap::new(),
            0,
            false,
            mock_data_sink,
        );
        alarm.put_data(&20, PlaceholderMap::new()).await.unwrap();
        alarm.action = times_action(0);
        for _ in 0..6 {
            alarm.put_data(&20, PlaceholderMap::new()).await.unwrap();
        }
        alarm.action = times_action(1);
        alarm.put_data(&20, PlaceholderMap::new()).await.unwrap();
    }

    #[tokio::test]
    async fn test_trigger_recover_action() {
        let _permit = SEMAPHORE.acquire().await.unwrap();
        let data_sink_ctx = MockDataSink::add_placeholders_context();
        data_sink_ctx
            .expect()
            .returning(|data: &u8, placeholders: &mut PlaceholderMap| {
                placeholders.insert(String::from("data"), format!("{}", data));
            });
        let mock_data_sink = mock_data_sink();
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            times_action(1),
            PlaceholderMap::new(),
            1,
            0,
            times_action(0),
            PlaceholderMap::from([(String::from("Hello"), String::from("World"))]),
            5,
            times_action(0),
            PlaceholderMap::new(),
            0,
            false,
            mock_data_sink,
        );
        alarm.put_data(&20, PlaceholderMap::new()).await.unwrap();
        for _ in 0..4 {
            alarm.put_data(&10, PlaceholderMap::new()).await.unwrap();
        }
        let mut mock_recover_action = action::MockAction::new();
        mock_recover_action
            .expect_trigger()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                uuid::Uuid::parse_str(placeholders.get("alarm_uuid").unwrap()).unwrap();
                use std::str::FromStr;
                chrono::DateTime::<chrono::Utc>::from_str(
                    placeholders.get("alarm_timestamp").unwrap(),
                )
                .unwrap();
                assert_eq!(placeholders.get("alarm_name").unwrap(), "Name");
                assert_eq!(placeholders.get("alarm_id").unwrap(), "ID");
                assert_eq!(placeholders.get("Hello").unwrap(), "World");
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                assert_eq!(placeholders.get("data").unwrap(), "10");
                assert_eq!(placeholders.len(), 7);
                true
            }))
            .returning(|_| Ok(()));
        alarm.recover_action = Some(std::sync::Arc::new(mock_recover_action));
        alarm
            .put_data(
                &10,
                PlaceholderMap::from([(String::from("Foo"), String::from("Bar"))]),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_trigger_error_action() {
        let mut mock_data_sink = MockDataSink::new();
        mock_data_sink.expect_put_data().never();
        let mut mock_action = action::MockAction::new();
        mock_action.expect_trigger().never();
        let mut mock_recover_action = action::MockAction::new();
        mock_recover_action.expect_trigger().never();
        let mut mock_error_action = action::MockAction::new();
        mock_error_action
            .expect_trigger()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                uuid::Uuid::parse_str(placeholders.get("error_uuid").unwrap()).unwrap();
                use std::str::FromStr;
                chrono::DateTime::<chrono::Utc>::from_str(
                    placeholders.get("error_timestamp").unwrap(),
                )
                .unwrap();
                assert_eq!(placeholders.get("alarm_name").unwrap(), "Name");
                assert_eq!(placeholders.get("alarm_id").unwrap(), "ID");
                assert_eq!(placeholders.get("Hello").unwrap(), "World");
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                assert_eq!(placeholders.len(), 6);
                true
            }))
            .returning(|_| Ok(()));
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            Some(std::sync::Arc::new(mock_action)),
            PlaceholderMap::new(),
            1,
            0,
            Some(std::sync::Arc::new(mock_recover_action)),
            PlaceholderMap::new(),
            1,
            Some(std::sync::Arc::new(mock_error_action)),
            PlaceholderMap::from([(String::from("Hello"), String::from("World"))]),
            3,
            false,
            mock_data_sink,
        );
        alarm
            .put_error(
                &Error(String::from("Error")),
                PlaceholderMap::from([(String::from("Foo"), String::from("Bar"))]),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_trigger_error_action_repeat() {
        let _permit = SEMAPHORE.acquire().await.unwrap();
        let data_sink_ctx = MockDataSink::add_placeholders_context();
        data_sink_ctx
            .expect()
            .returning(|data: &u8, placeholders: &mut PlaceholderMap| {
                placeholders.insert(String::from("data"), format!("{}", data));
            });
        let mock_data_sink = mock_data_sink();
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            times_action(0),
            PlaceholderMap::new(),
            1,
            0,
            times_action(0),
            PlaceholderMap::new(),
            1,
            times_action(1),
            PlaceholderMap::new(),
            7,
            false,
            mock_data_sink,
        );
        alarm
            .put_error(&Error(String::from("Error")), PlaceholderMap::new())
            .await
            .unwrap();
        alarm.error_action = times_action(0);
        for _ in 0..6 {
            alarm
                .put_error(&Error(String::from("Error")), PlaceholderMap::new())
                .await
                .unwrap();
        }
        alarm.error_action = times_action(1);
        alarm
            .put_error(&Error(String::from("Error")), PlaceholderMap::new())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_invert() {
        let _permit = SEMAPHORE.acquire().await.unwrap();
        let data_sink_ctx = MockDataSink::add_placeholders_context();
        data_sink_ctx
            .expect()
            .returning(|data: &u8, placeholders: &mut PlaceholderMap| {
                placeholders.insert(String::from("data"), format!("{}", data));
            });
        let mock_data_sink = mock_data_sink();
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            times_action(1),
            PlaceholderMap::new(),
            1,
            0,
            times_action(0),
            PlaceholderMap::new(),
            1,
            times_action(0),
            PlaceholderMap::new(),
            0,
            true,
            mock_data_sink,
        );
        alarm.put_data(&10, PlaceholderMap::new()).await.unwrap();
        alarm.action = times_action(0);
        alarm.recover_action = times_action(1);
        alarm.put_data(&20, PlaceholderMap::new()).await.unwrap();
    }
}
