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
        let (code, stderr) = self.process_config.run(Some(placeholders)).await?;
        if code != 0 {
            return match stderr {
                None => Err(Error(format!("Process failed with code {code}."))),
                Some(stderr) => Err(Error(format!("Process failed with code {code}: {stderr}"))),
            };
        }
        Ok(())
    }
}
