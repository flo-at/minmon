use super::Action;
use crate::config;
use crate::process::ProcessConfig;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

pub struct Process {
    process_config: ProcessConfig,
}

impl TryFrom<&config::Action> for Process {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::Process(process) = &action.type_ {
            Ok(Self {
                process_config: ProcessConfig::try_from(&process.process_config)?,
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl Action for Process {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        let result = self.process_config.run(Some(placeholders)).await?;
        let code = result.code;
        if code != 0 {
            return Err(Error(if result.stderr.is_empty() {
                format!("Process failed with code {code}.")
            } else {
                format!("Process failed with code {code}: {}", result.stderr)
            }));
        }
        Ok(())
    }
}
