use super::DataSource;
use crate::{config, measurement};
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;
use measurement::Measurement;

pub struct FilesystemUsage {
    mountpoints: Vec<String>,
}

impl TryFrom<&config::Check> for FilesystemUsage {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
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
    type Item = measurement::Level;

    async fn get_data(
        &mut self,
        _placeholders: &mut PlaceholderMap,
    ) -> Result<Vec<Result<Option<Self::Item>>>> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            res.push(match nix::sys::statvfs::statvfs(mountpoint.as_str()) {
                Err(err) => Err(Error(format!("Call to 'statvfs' failed: {err}"))),
                Ok(stat) => {
                    let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
                    Self::Item::new(usage as u8).map(Some)
                }
            })
        }
        Ok(res)
    }

    fn format_data(&self, data: &Self::Item) -> String {
        format!("usage level {data}")
    }

    fn ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}
