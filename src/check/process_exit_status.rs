use super::DataSource;
use crate::process::ProcessConfig;
use crate::{config, measurement};
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;
use measurement::Measurement;

pub struct ProcessExitStatus {
    id: Vec<String>,
    process_config: ProcessConfig,
}

impl TryFrom<&config::Check> for ProcessExitStatus {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
        if let config::CheckType::ProcessExitStatus(process_exit_status) = &check.type_ {
            let process_config = ProcessConfig::try_from(&process_exit_status.process_config)?;
            Ok(Self {
                id: vec![process_config.file_name().map(|x| x.into())?],
                process_config,
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for ProcessExitStatus {
    type Item = measurement::StatusCode;

    async fn get_data(
        &mut self,
        placeholders: &mut PlaceholderMap,
    ) -> Result<Vec<Result<Option<Self::Item>>>> {
        let result = self.process_config.run(None).await?;
        placeholders.insert(String::from("stdout"), result.stdout);
        placeholders.insert(String::from("stderr"), result.stderr);
        Ok(vec![Self::Item::new(result.code).map(Some)])
    }

    fn format_data(&self, data: &Self::Item) -> String {
        format!("exit code {data}")
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}
