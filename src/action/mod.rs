use crate::config;
use crate::{PlaceholderMap, Result};
use async_trait::async_trait;

mod log;
mod process;
mod webhook;
pub use self::log::Log;
pub use process::Process;
pub use webhook::Webhook;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Action: Send + Sync {
    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()>;
}

pub struct ActionBase<T>
where
    T: Action,
{
    name: String,
    placeholders: PlaceholderMap,
    action: T,
}

impl<T> ActionBase<T>
where
    T: Action,
{
    pub fn new(name: String, placeholders: PlaceholderMap, action: T) -> Self {
        Self {
            name,
            placeholders,
            action,
        }
    }

    fn add_placeholders(&self, placeholders: &mut PlaceholderMap) {
        placeholders.insert(String::from("action_name"), self.name.clone());
        crate::merge_placeholders(placeholders, &self.placeholders);
    }
}

#[async_trait]
impl<T> Action for ActionBase<T>
where
    T: Action,
{
    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        self.add_placeholders(&mut placeholders);
        self.action.trigger(placeholders).await
    }
}

pub fn from_action_config(action_config: &config::Action) -> Result<std::sync::Arc<dyn Action>> {
    Ok(match &action_config.type_ {
        config::ActionType::Webhook(_) => std::sync::Arc::new(ActionBase::new(
            action_config.name.clone(),
            action_config.placeholders.clone(),
            Webhook::try_from(action_config)?,
        )),
        config::ActionType::Log(_) => std::sync::Arc::new(ActionBase::new(
            action_config.name.clone(),
            action_config.placeholders.clone(),
            Log::try_from(action_config)?,
        )),
        config::ActionType::Process(_) => std::sync::Arc::new(ActionBase::new(
            action_config.name.clone(),
            action_config.placeholders.clone(),
            Process::try_from(action_config)?,
        )),
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_placeholders() {
        let mut mock_action = MockAction::new();
        mock_action
            .expect_trigger()
            .once()
            .with(eq(PlaceholderMap::from([
                (String::from("action_name"), String::from("Name")),
                (String::from("Hello"), String::from("World")),
                (String::from("Foo"), String::from("Bar")),
            ])))
            .returning(|_| Ok(()));
        let action = ActionBase::new(
            String::from("Name"),
            PlaceholderMap::from([(String::from("Hello"), String::from("World"))]),
            mock_action,
        );
        action
            .trigger(PlaceholderMap::from([(
                String::from("Foo"),
                String::from("Bar"),
            )]))
            .await
            .unwrap();
    }
}
