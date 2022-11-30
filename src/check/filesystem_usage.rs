use super::DataSource;
use crate::config;
use crate::{Error, Result};
use async_trait::async_trait;

// NOTE uses "available blocks" (not "free blocks") i.e. blocks available to unpriv. users
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

    async fn get_data(&self) -> Result<Vec<Result<Self::Item>>> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            res.push(match nix::sys::statvfs::statvfs(mountpoint.as_str()) {
                Err(err) => Err(Error(format!("Call to 'statvfs' failed: {}", err))),
                Ok(stat) => {
                    let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
                    Ok(usage as u8)
                }
            })
        }
        Ok(res)
    }

    fn measurement_ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}
