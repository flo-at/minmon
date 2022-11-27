use super::DataSource;
use crate::config;
use async_trait::async_trait;

pub struct FilesystemUsage {
    mountpoints: Vec<String>, // TODO possible to store a reference?
}

impl From<&config::Check> for FilesystemUsage {
    fn from(check: &config::Check) -> Self {
        if let config::CheckType::FilesystemUsage(filesystem_usage_config) = &check.type_ {
            Self {
                mountpoints: filesystem_usage_config.mountpoints.clone(),
            }
        } else {
            panic!(); // TODO
        }
    }
}

#[async_trait]
impl DataSource for FilesystemUsage {
    type Item = u8;

    fn validate(&self) -> bool {
        // TODO validate existance of mountpoints
        true
    }

    async fn get_data(&self) -> Vec<Self::Item> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            let stat = nix::sys::statvfs::statvfs(&mountpoint[..]).unwrap();
            let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
            res.push(usage as u8);
        }
        res
    }

    fn measurement_ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}
