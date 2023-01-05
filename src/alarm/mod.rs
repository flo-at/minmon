use crate::action;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

mod binary_state;
mod level;
mod state_machine;
mod status_code;
#[cfg(feature = "sensors")]
mod temperature;

pub use binary_state::BinaryState;
pub use level::Level;
pub use state_machine::{StateHandler, StateMachine};
pub use status_code::StatusCode;
#[cfg(feature = "sensors")]
pub use temperature::Temperature;

#[cfg_attr(test, mockall::automock(type Item=u8;))]
pub trait DataSink: Send + Sync + Sized {
    type Item: Send + Sync;

    fn put_data(&mut self, data: &Self::Item) -> Result<SinkDecision>;
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

    fn log_id(&self) -> &str;

    async fn put_data(&mut self, data: &Self::Item, mut placeholders: PlaceholderMap)
        -> Result<()>;
    async fn put_error(&mut self, error: &Error, mut placeholders: PlaceholderMap) -> Result<()>;
}

pub struct AlarmBase<T, U = StateMachine>
where
    T: DataSink,
    U: StateHandler,
{
    name: String,
    id: String,
    action: std::sync::Arc<dyn action::Action>,
    placeholders: PlaceholderMap,
    recover_action: Option<std::sync::Arc<dyn action::Action>>,
    recover_placeholders: PlaceholderMap,
    error_action: Option<std::sync::Arc<dyn action::Action>>,
    error_placeholders: PlaceholderMap,
    invert: bool,
    state_machine: U,
    data_sink: T,
    log_id: String,
}

impl<T, U> AlarmBase<T, U>
where
    T: DataSink,
    U: StateHandler,
{
    pub fn new(
        name: String,
        id: String,
        action: std::sync::Arc<dyn action::Action>,
        placeholders: PlaceholderMap,
        recover_action: Option<std::sync::Arc<dyn action::Action>>,
        recover_placeholders: PlaceholderMap,
        error_action: Option<std::sync::Arc<dyn action::Action>>,
        error_placeholders: PlaceholderMap,
        invert: bool,
        state_machine: U,
        data_sink: T,
        log_id: String,
    ) -> Result<Self> {
        if name.is_empty() {
            Err(Error(String::from("'name' cannot be empty.")))
        } else {
            Ok(Self {
                name,
                id,
                action,
                placeholders,
                recover_action,
                recover_placeholders,
                error_action,
                error_placeholders,
                invert,
                state_machine,
                data_sink,
                log_id,
            })
        }
    }

    async fn error(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        if self.state_machine.error() {
            self.trigger_error(placeholders).await?;
        }
        Ok(())
    }

    async fn bad(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        if self.state_machine.bad() {
            self.trigger(placeholders).await?;
        }
        Ok(())
    }

    async fn good(&mut self, placeholders: PlaceholderMap) -> Result<()> {
        if self.state_machine.good() {
            self.trigger_recover(placeholders).await?;
        }
        Ok(())
    }

    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        self.state_machine.add_placeholders(&mut placeholders);
        self.action.trigger(placeholders).await
    }

    async fn trigger_recover(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        self.state_machine.add_placeholders(&mut placeholders);
        crate::merge_placeholders(&mut placeholders, &self.recover_placeholders);
        match &self.recover_action {
            Some(action) => action.trigger(placeholders).await,
            None => Ok(()),
        }
    }

    async fn trigger_error(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        self.state_machine.add_placeholders(&mut placeholders);
        crate::merge_placeholders(&mut placeholders, &self.error_placeholders);
        match &self.error_action {
            Some(action) => action.trigger(placeholders).await,
            None => Ok(()),
        }
    }

    fn add_placeholders(&self, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("alarm_name"), self.name.clone());
        placeholders.insert(String::from("check_id"), self.id.clone());
        crate::merge_placeholders(placeholders, &self.placeholders);
    }
}

#[async_trait]
impl<T, U> Alarm for AlarmBase<T, U>
where
    T: DataSink,
    U: StateHandler,
{
    type Item = T::Item;

    fn log_id(&self) -> &str {
        &self.log_id
    }

    async fn put_data(
        &mut self,
        data: &Self::Item,
        mut placeholders: PlaceholderMap,
    ) -> Result<()> {
        T::add_placeholders(data, &mut placeholders);
        self.add_placeholders(&mut placeholders);
        let mut decision = self.data_sink.put_data(data)?;
        if self.invert {
            decision = !decision;
        }
        match decision {
            SinkDecision::Good => self.good(placeholders).await,
            SinkDecision::Bad => {
                log::warn!("{}: Data is bad.", self.log_id);
                self.bad(placeholders).await
            }
        }
    }

    async fn put_error(&mut self, error: &Error, mut placeholders: PlaceholderMap) -> Result<()> {
        log::error!("{} got an error: {}", self.log_id, error);
        self.add_placeholders(&mut placeholders);
        self.error(placeholders).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::predicate::*;

    static SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

    fn times_action(times: usize) -> std::sync::Arc<dyn action::Action> {
        let mut mock_action = action::MockAction::new();
        mock_action
            .expect_trigger()
            .times(times)
            .returning(|_| Ok(()));
        std::sync::Arc::new(mock_action)
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
                placeholders.insert(String::from("data"), data.to_string());
            });
        let mock_data_sink = mock_data_sink();
        let mut mock_action = action::MockAction::new();
        mock_action
            .expect_trigger()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                assert_eq!(placeholders.get("alarm_name").unwrap(), "Name");
                assert_eq!(placeholders.get("check_id").unwrap(), "ID");
                assert_eq!(placeholders.get("Hello").unwrap(), "World");
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                assert_eq!(placeholders.get("data").unwrap(), "20");
                assert_eq!(placeholders.len(), 5);
                true
            }))
            .returning(|_| Ok(()));
        let mut mock_state_machine = state_machine::MockStateHandler::new();
        mock_state_machine.expect_bad().once().return_const(true);
        mock_state_machine
            .expect_add_placeholders()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                true
            }))
            .return_const(());
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            std::sync::Arc::new(mock_action),
            PlaceholderMap::from([(String::from("Hello"), String::from("World"))]),
            Some(times_action(0)),
            PlaceholderMap::new(),
            Some(times_action(0)),
            PlaceholderMap::new(),
            false,
            mock_state_machine,
            mock_data_sink,
            String::from(""),
        )
        .unwrap();
        alarm
            .put_data(
                &20,
                PlaceholderMap::from([(String::from("Foo"), String::from("Bar"))]),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_trigger_recover_action() {
        let _permit = SEMAPHORE.acquire().await.unwrap();
        let data_sink_ctx = MockDataSink::add_placeholders_context();
        data_sink_ctx
            .expect()
            .returning(|data: &u8, placeholders: &mut PlaceholderMap| {
                placeholders.insert(String::from("data"), data.to_string());
            });
        let mock_data_sink = mock_data_sink();
        let mut mock_state_machine = state_machine::MockStateHandler::new();
        mock_state_machine.expect_good().once().return_const(true);
        mock_state_machine
            .expect_add_placeholders()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                true
            }))
            .return_const(());
        let mut mock_recover_action = action::MockAction::new();
        mock_recover_action
            .expect_trigger()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                assert_eq!(placeholders.get("alarm_name").unwrap(), "Name");
                assert_eq!(placeholders.get("check_id").unwrap(), "ID");
                assert_eq!(placeholders.get("Hello").unwrap(), "World");
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                assert_eq!(placeholders.get("data").unwrap(), "10");
                assert_eq!(placeholders.len(), 5);
                true
            }))
            .returning(|_| Ok(()));
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            times_action(0),
            PlaceholderMap::new(),
            Some(std::sync::Arc::new(mock_recover_action)),
            PlaceholderMap::from([(String::from("Hello"), String::from("World"))]),
            Some(times_action(0)),
            PlaceholderMap::new(),
            false,
            mock_state_machine,
            mock_data_sink,
            String::from(""),
        )
        .unwrap();
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
                assert_eq!(placeholders.get("alarm_name").unwrap(), "Name");
                assert_eq!(placeholders.get("check_id").unwrap(), "ID");
                assert_eq!(placeholders.get("Hello").unwrap(), "World");
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                assert_eq!(placeholders.len(), 4);
                true
            }))
            .returning(|_| Ok(()));
        let mut mock_state_machine = state_machine::MockStateHandler::new();
        mock_state_machine.expect_error().once().return_const(true);
        mock_state_machine
            .expect_add_placeholders()
            .once()
            .with(function(|placeholders: &PlaceholderMap| {
                assert_eq!(placeholders.get("Foo").unwrap(), "Bar");
                true
            }))
            .return_const(());
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            std::sync::Arc::new(mock_action),
            PlaceholderMap::new(),
            Some(std::sync::Arc::new(mock_recover_action)),
            PlaceholderMap::new(),
            Some(std::sync::Arc::new(mock_error_action)),
            PlaceholderMap::from([(String::from("Hello"), String::from("World"))]),
            false,
            mock_state_machine,
            mock_data_sink,
            String::from(""),
        )
        .unwrap();
        alarm
            .put_error(
                &Error(String::from("Error")),
                PlaceholderMap::from([(String::from("Foo"), String::from("Bar"))]),
            )
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
                placeholders.insert(String::from("data"), data.to_string());
            });
        let mock_data_sink = mock_data_sink();
        let mut mock_state_machine = state_machine::MockStateHandler::new();
        mock_state_machine.expect_bad().once().return_const(true);
        mock_state_machine.expect_good().once().return_const(true);
        mock_state_machine
            .expect_add_placeholders()
            .times(2)
            .return_const(());
        let mut alarm = AlarmBase::new(
            String::from("Name"),
            String::from("ID"),
            times_action(1),
            PlaceholderMap::new(),
            Some(times_action(0)),
            PlaceholderMap::new(),
            Some(times_action(0)),
            PlaceholderMap::new(),
            true,
            mock_state_machine,
            mock_data_sink,
            String::from(""),
        )
        .unwrap();
        alarm.put_data(&10, PlaceholderMap::new()).await.unwrap();
        alarm.action = times_action(0);
        alarm.recover_action = Some(times_action(1));
        alarm.put_data(&20, PlaceholderMap::new()).await.unwrap();
    }
}
