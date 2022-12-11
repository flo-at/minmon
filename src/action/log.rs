use super::Action;
use crate::config;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

pub struct Log {
    level: log::Level,
    template: String,
}

impl TryFrom<&config::Action> for Log {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::Log(log) = &action.type_ {
            if log.template.is_empty() {
                Err(Error(String::from("'template' cannot be empty.")))
            } else {
                Ok(Self {
                    level: log.level.into(),
                    template: log.template.clone(),
                })
            }
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl Action for Log {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        let text = crate::fill_placeholders(self.template.as_str(), &placeholders);
        log::log!(self.level, "{}", text);
        Ok(())
    }
}
