use super::Action;
use crate::config;
use crate::placeholder::PlaceholderMap;
use crate::{Error, Result};
use async_trait::async_trait;

pub struct Log {
    name: String,
    template: String,
}

impl TryFrom<&config::Action> for Log {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::Log(log) = &action.type_ {
            Ok(Self {
                name: action.name.clone(),
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
        let template = text_placeholder::Template::new(&self.template[..]);
        let text = template.fill_with_hashmap(
            &placeholders
                .iter()
                .map(|(k, v)| (&k[..], &v[..]))
                .chain(std::iter::once(("action_name", &self.name[..])))
                .collect(),
        );
        log::error!("{}", text);
        Ok(())
    }
}
