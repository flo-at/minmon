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

    fn add_placeholders(&self, mut placeholders: PlaceholderMap) -> Result<PlaceholderMap> {
        placeholders.insert(String::from("action_name"), self.name.clone());
        for (key, value) in self.placeholders.iter() {
            placeholders.insert(key.clone(), value.clone());
        }
        Ok(placeholders)
    }
}

#[async_trait]
impl<T> Action for ActionBase<T>
where
    T: Action,
{
    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()> {
        let placeholders = self.add_placeholders(placeholders)?;
        self.action.trigger(placeholders).await
    }
}
