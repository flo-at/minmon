use super::DataSource;
use crate::config;
use crate::{Error, Result};
use async_trait::async_trait;

pub struct FilesystemUsage {
    mountpoints: Vec<String>,
}

impl From<&config::Check> for FilesystemUsage {
    fn from(check: &config::Check) -> Self {
        if let config::CheckType::FilesystemUsage(filesystem_usage_config) = &check.type_ {
            Self {
                mountpoints: filesystem_usage_config.mountpoints.clone(),
            }
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for FilesystemUsage {
    type Item = u8;

    fn validate(&self) -> Result<()> {
        // TODO validate existance of mountpoints
        Ok(())
    }

    async fn get_data(&self) -> Result<Vec<Self::Item>> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            let stat = nix::sys::statvfs::statvfs(&mountpoint[..])
                .map_err(|x| Error(format!("Call to 'statvfs' failed: {}", x)))?;
            let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
            res.push(usage as u8);
        }
        Ok(res)
    }

    fn measurement_ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}
