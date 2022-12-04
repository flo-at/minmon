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
        config::ActionType::WebHook(_) => std::sync::Arc::new(ActionBase::new(
            action_config.name.clone(),
            action_config.placeholders.clone(),
            WebHook::try_from(action_config)?,
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
