use crate::config;
use crate::{PlaceholderMap, Result};
use async_trait::async_trait;

mod log;
mod web_hook;
pub use self::log::Log;
pub use web_hook::WebHook;

#[async_trait]
pub trait Action: Send + Sync {
    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()>;
}

pub struct ActionBase {
    name: String,
    placeholders: PlaceholderMap,
}

impl ActionBase {
    #[cfg(test)]
    pub fn new(name: &str, placeholders: PlaceholderMap) -> Self {
        Self {
            name: name.to_string(),
            placeholders,
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

impl From<&config::Action> for ActionBase {
    fn from(action: &config::Action) -> Self {
        Self {
            name: action.name.clone(),
            placeholders: action.placeholders.clone(),
        }
    }
}
