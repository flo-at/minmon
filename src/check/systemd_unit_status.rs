use super::DataSource;
use crate::process::ProcessConfig;
use crate::{config, measurement};
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;
use measurement::Measurement;

const SYSTEMCTL_BINARY: &str = "/usr/bin/systemctl";
const SYSTEMD_RUN_BINARY: &str = "/usr/bin/systemd-run";
const USER_ARG: &str = "--user";
const STATUS_ARG: &str = "status";

const SYSTEMCTL_STATUS_NOT_ACTIVE: u8 = 3;
const SYSTEMCTL_STATUS_NO_SUCH_UNIT: u8 = 4;

// This calls the `systemctl` process instead of using the sd-bus interface because there is a lot
// of code that can be reused this way.
pub struct SystemdUnitStatus {
    id: Vec<String>,
    process_configs: Vec<ProcessConfig>,
}

fn process_config_system(unit: &str) -> Result<ProcessConfig> {
    let arguments = vec![STATUS_ARG.into(), unit.into()];
    ProcessConfig::new(
        SYSTEMCTL_BINARY.into(),
        arguments,
        std::collections::HashMap::new(),
        None,
        None,
        None,
        config::default::PROCESS_CONFIG_STDOUT_MAX,
        config::default::PROCESS_CONFIG_STDERR_MAX,
    )
}

fn process_config_user(uid: u32, unit: &str) -> Result<ProcessConfig> {
    let arguments = vec![
        "--quiet".into(),
        "--pipe".into(),
        "--wait".into(),
        "--collect".into(),
        "--user".into(),
        format!("--machine={uid}@.host"),
        SYSTEMCTL_BINARY.into(),
        USER_ARG.into(),
        STATUS_ARG.into(),
        unit.into(),
    ];
    ProcessConfig::new(
        SYSTEMD_RUN_BINARY.into(),
        arguments,
        std::collections::HashMap::new(),
        None,
        None,
        None,
        config::default::PROCESS_CONFIG_STDOUT_MAX,
        config::default::PROCESS_CONFIG_STDERR_MAX,
    )
}

impl TryFrom<&config::Check> for SystemdUnitStatus {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
        if let config::CheckType::SystemdUnitStatus(unit_status) = &check.type_ {
            let mut id = Vec::new();
            let mut process_configs = Vec::new();
            for unit in unit_status.units.iter() {
                if unit.uid() != 0 {
                    id.push(format!("{}[{}]", unit.unit(), unit.uid()));
                    process_configs.push(process_config_user(unit.uid(), unit.unit())?);
                } else {
                    id.push(unit.unit().into());
                    process_configs.push(process_config_system(unit.unit())?);
                }
            }
            Ok(Self {
                id,
                process_configs,
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for SystemdUnitStatus {
    type Item = measurement::BinaryState;

    async fn get_data(
        &mut self,
        _placeholders: &mut PlaceholderMap,
    ) -> Result<Vec<Result<Option<Self::Item>>>> {
        let mut res = Vec::new();
        for process_config in self.process_configs.iter() {
            let result = process_config.run(None).await?;
            res.push(match result.code {
                0 => Self::Item::new(true).map(Some),
                SYSTEMCTL_STATUS_NOT_ACTIVE => Self::Item::new(false).map(Some),
                SYSTEMCTL_STATUS_NO_SUCH_UNIT => Err(Error(String::from("No such unit."))),
                code => Err(Error(format!("Unknown error code {code}."))),
            });
        }
        Ok(res)
    }

    fn format_data(&self, data: &Self::Item) -> String {
        match data.data() {
            true => "unit active",
            false => "unit inactive",
        }
        .into()
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}
