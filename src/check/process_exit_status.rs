use super::DataSource;
use crate::config;
use crate::process::ProcessConfig;
use crate::{Error, Result};
use async_trait::async_trait;

pub struct ProcessExitStatus {
    id: Vec<String>,
    process_config: ProcessConfig,
}

impl TryFrom<&config::Check> for ProcessExitStatus {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, self::Error> {
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
    type Item = u8;

    async fn get_data(&self) -> Result<Vec<Result<Self::Item>>> {
        let (code, _) = self.process_config.run(None).await?;
        Ok(vec![Ok(code)])
    }

    fn format_data(data: &Self::Item) -> String {
        format!("exit code {data}")
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}
