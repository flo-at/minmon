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
            if filesystem_usage.mountpoints.iter().any(|x| x.is_empty()) {
                Err(Error(String::from(
                    "'mountpoints' cannot contain empty paths.",
                )))
            } else {
                Ok(Self {
                    mountpoints: filesystem_usage.mountpoints.clone(),
                })
            }
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
                Err(err) => Err(Error(format!("Call to 'statvfs' failed: {err}"))),
                Ok(stat) => {
                    let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
                    Ok(usage as u8)
                }
            })
        }
        Ok(res)
    }

    fn format_data(data: &Self::Item) -> String {
        format!("usage level {data}%")
    }

    fn ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}
