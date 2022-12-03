use crate::config;
use crate::{PlaceholderMap, Result};
use async_trait::async_trait;

mod log;
mod process;
mod web_hook;
pub use self::log::Log;
pub use process::Process;
pub use web_hook::WebHook;

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
    pub fn new(name: &str, placeholders: &PlaceholderMap, action: T) -> Self {
        Self {
            name: name.to_string(),
            placeholders: placeholders.clone(),
            action,
        }
    }

    fn add_placeholders(&self, placeholders: &mut PlaceholderMap) -> Result<()> {
        placeholders.insert(String::from("action_name"), self.name.clone());
        for (key, value) in self.placeholders.iter() {
            placeholders.insert(key.clone(), value.clone());
        }
        Ok(())
    }
}

#[async_trait]
impl<T> Action for ActionBase<T>
where
    T: Action,
{
    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        self.add_placeholders(&mut placeholders)?;
        self.action.trigger(placeholders).await
    }
}

pub fn from_action_config(action_config: &config::Action) -> Result<std::sync::Arc<dyn Action>> {
    Ok(match &action_config.type_ {
        config::ActionType::WebHook(_) => std::sync::Arc::new(ActionBase::new(
            &action_config.name,
            &action_config.placeholders,
            WebHook::try_from(action_config)?,
        )),
        config::ActionType::Log(_) => std::sync::Arc::new(ActionBase::new(
            &action_config.name,
            &action_config.placeholders,
            Log::try_from(action_config)?,
        )),
        config::ActionType::Process(_) => std::sync::Arc::new(ActionBase::new(
            &action_config.name,
            &action_config.placeholders,
            Process::try_from(action_config)?,
        )),
    })
}
