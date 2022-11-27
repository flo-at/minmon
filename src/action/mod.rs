use crate::placeholder::PlaceholderMap;
use crate::Result;
use async_trait::async_trait;

mod web_hook;
pub use web_hook::WebHook;

#[async_trait]
pub trait Action: Send + Sync {
    async fn trigger(&self, placeholders: &PlaceholderMap) -> Result<()>;
}
