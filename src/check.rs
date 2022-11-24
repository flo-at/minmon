use crate::config;

// for checks that return "levels" (u8 in [0..100])
pub trait LevelSource {
    fn get_levels(&self) -> Vec<(String, u8)>;
}

pub trait MeasurementIds {
    fn measurement_ids(&self) -> &[String];
}

pub struct FilesystemUsage {
    mountpoints: Vec<String>, // TODO possible to store a reference?
}

impl From<&config::CheckFilesystemUsage> for FilesystemUsage {
    fn from(filesystem_usage: &config::CheckFilesystemUsage) -> Self {
        Self {
            mountpoints: filesystem_usage.mountpoints.clone(),
        }
    }
}

impl LevelSource for FilesystemUsage {
    fn get_levels(&self) -> Vec<(String, u8)> {
        let mut res = Vec::new();
        for mountpoint in self.mountpoints.iter() {
            let stat = nix::sys::statvfs::statvfs(&mountpoint[..]).unwrap();
            let usage = (stat.blocks() - stat.blocks_available()) * 100 / stat.blocks();
            res.push((mountpoint.clone(), usage as u8));
        }
        res
    }
}

impl MeasurementIds for FilesystemUsage {
    fn measurement_ids(&self) -> &[String] {
        &self.mountpoints[..]
    }
}
