use super::{Action, ActionBase};
use crate::config;
use crate::placeholder::PlaceholderMap;
use crate::{Error, Result};
use async_trait::async_trait;

pub struct Log {
    action: ActionBase,
    template: String,
}

impl TryFrom<&config::Action> for Log {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::Log(log) = &action.type_ {
            Ok(Self {
                action: ActionBase::from(action),
                template: log.template.clone(),
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl Action for Log {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        // TODO irgendwie in Actionbase verschieben
        let placeholders = self.action.add_placeholders(placeholders)?;
        let template = text_placeholder::Template::new(self.template.as_str());
        let text = template.fill_with_hashmap(
            &placeholders
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect(),
        );
        log::error!("{}", text);
        Ok(())
    }
}
