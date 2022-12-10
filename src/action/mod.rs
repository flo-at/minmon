use crate::config;
use crate::ActionMap;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;
extern crate log as log_ext;

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
    pub fn new(name: String, placeholders: PlaceholderMap, action: T) -> Result<Self> {
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

struct DisabledAction {}

#[async_trait]
impl Action for DisabledAction {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        if placeholders.contains_key("event_name") {
            log_ext::debug!(
                "Disabled action '{}' triggered for report event '{}'.",
                placeholders.get("action_name").unwrap(),
                placeholders.get("event_name").unwrap()
            );
        } else {
            log_ext::debug!(
                "Disabled action '{}' triggered for alarm '{}' from check '{}'.",
                placeholders.get("action_name").unwrap(),
                placeholders.get("alarm_name").unwrap(),
                placeholders.get("check_name").unwrap()
            );
        }
        Ok(())
    }
}

pub fn from_action_config(action_config: &config::Action) -> Result<std::sync::Arc<dyn Action>> {
    if action_config.disable {
        log_ext::info!(
            "Action {}::'{}' is disabled.",
            action_config.type_,
            action_config.name
        );
        Ok(std::sync::Arc::new(ActionBase::new(
            action_config.name.clone(),
            action_config.placeholders.clone(),
            DisabledAction {},
        )?))
    } else {
        Ok(match &action_config.type_ {
            config::ActionType::Webhook(_) => std::sync::Arc::new(ActionBase::new(
                action_config.name.clone(),
                action_config.placeholders.clone(),
                Webhook::try_from(action_config)?,
            )?),
            config::ActionType::Log(_) => std::sync::Arc::new(ActionBase::new(
                action_config.name.clone(),
                action_config.placeholders.clone(),
                Log::try_from(action_config)?,
            )?),
            config::ActionType::Process(_) => std::sync::Arc::new(ActionBase::new(
                action_config.name.clone(),
                action_config.placeholders.clone(),
                Process::try_from(action_config)?,
            )?),
        })
    }
}

pub fn get_action(action: &String, actions: &ActionMap) -> Result<std::sync::Arc<dyn Action>> {
    if action.is_empty() {
        Err(Error(String::from("'name' cannot be empty.")))
    } else {
        Ok(actions
            .get(action)
            .ok_or_else(|| Error(format!("Action '{}' not found.", action)))?
            .clone())
    }
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
        )
        .unwrap();
        action
            .trigger(PlaceholderMap::from([(
                String::from("Foo"),
                String::from("Bar"),
            )]))
            .await
            .unwrap();
    }
}
