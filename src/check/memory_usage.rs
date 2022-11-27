use std::io::BufRead;

use super::DataSource;
use crate::config;
use async_trait::async_trait;
use tokio::io::AsyncReadExt;

pub struct MemoryUsage {
    id: Vec<String>,
}

impl From<&config::Check> for MemoryUsage {
    fn from(check: &config::Check) -> Self {
        if let config::CheckType::MemoryUsage = &check.type_ {
            Self {
                id: vec![String::from("Memory")],
            }
        } else {
            panic!(); // TODO
        }
    }
}

#[async_trait]
impl DataSource for MemoryUsage {
    type Item = u8;

    fn validate(&self) -> bool {
        true
    }

    async fn get_data(&self) -> Vec<Self::Item> {
        let mut file = tokio::fs::File::open("/proc/meminfo").await.unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await.unwrap();
        let mut mem_total: Option<usize> = None;
        let mut mem_available: Option<usize> = None;
        for line in buffer.lines() {
            let line = line.unwrap();
            if line.starts_with("MemTotal") {
                mem_total = Some(line.split_whitespace().nth(1).unwrap().parse().unwrap());
            } else if line.starts_with("MemAvailable") {
                mem_available = Some(line.split_whitespace().nth(1).unwrap().parse().unwrap());
            }
            if let (Some(mem_total), Some(mem_available)) = (mem_total, mem_available) {
                let usage = ((mem_total - mem_available) * 100 / mem_total) as u8;
                log::debug!("Memory usage is {}%", usage);
                return vec![usage];
            }
        }
        panic!(); // TODO
    }

    fn measurement_ids(&self) -> &[String] {
        &self.id[..]
    }
}
