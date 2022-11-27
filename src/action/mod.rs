use crate::placeholder::PlaceholderMap;
use crate::Result;
use async_trait::async_trait;

mod log;
mod web_hook;
pub use self::log::Log;
pub use web_hook::WebHook;

#[async_trait]
pub trait Action: Send + Sync {
    async fn trigger(&self, mut placeholders: PlaceholderMap) -> Result<()>;
}
