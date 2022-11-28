use super::DataSource;
use crate::config;
use crate::{Error, Result};
use async_trait::async_trait;

pub struct FilesystemUsage {
    mountpoints: Vec<String>,
}

impl TryFrom<&config::Check> for FilesystemUsage {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, self::Error> {
        if let config::CheckType::FilesystemUsage(filesystem_usage) = &check.type_ {
            Ok(Self {
                mountpoints: filesystem_usage.mountpoints.clone(),
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for FilesystemUsage {
    type Item = u8;

    async fn get_data(&self) -> Result<Vec<Self::Item>> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            let stat = nix::sys::statvfs::statvfs(mountpoint.as_str())
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
