use super::DataSource;
use crate::process::ProcessConfig;
use crate::{config, measurement};
use crate::{Error, Result};
use async_trait::async_trait;
use measurement::Measurement;

const SYSTEMCTL_BINARY: &str = "/usr/bin/systemctl";
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

impl TryFrom<&config::Check> for SystemdUnitStatus {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
        if let config::CheckType::SystemdUnitStatus(unit_status) = &check.type_ {
            let mut id = Vec::new();
            let mut process_configs = Vec::new();
            for unit in unit_status.units.iter() {
                let mut arguments = Vec::new();
                arguments.push(STATUS_ARG.into());
                if unit.uid() != 0 {
                    arguments.push(USER_ARG.into());
                    id.push(format!("{}[{}]", unit.unit(), unit.uid()));
                } else {
                    id.push(unit.unit().to_string());
                }
                arguments.push(unit.unit().to_string());
                process_configs.push(ProcessConfig::new(
                    SYSTEMCTL_BINARY.into(),
                    arguments,
                    std::collections::HashMap::new(),
                    None,
                    match unit.uid() {
                        0 => None,
                        uid => Some(uid),
                    },
                    None,
                )?);
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

    async fn get_data(&self) -> Result<Vec<Result<Self::Item>>> {
        let mut res = Vec::new();
        for process_config in self.process_configs.iter() {
            let (code, _) = process_config.run(None).await?;
            res.push(match code {
                0 => Self::Item::new(true),
                SYSTEMCTL_STATUS_NOT_ACTIVE => Self::Item::new(false),
                SYSTEMCTL_STATUS_NO_SUCH_UNIT => Err(Error(String::from("No such unit."))),
                code => Err(Error(format!("Unknown error code {code}."))),
            });
        }
        Ok(res)
    }

    fn format_data(data: &Self::Item) -> String {
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
