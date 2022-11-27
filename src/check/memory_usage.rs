use std::io::BufRead;

use super::DataSource;
use crate::config;
use crate::{Error, Result};
use async_trait::async_trait;
use tokio::io::AsyncReadExt;

const MEMINFO_PATH: &str = "/proc/meminfo";

pub struct MemoryUsage {
    id: Vec<String>,
}

// TODO implement swap
impl MemoryUsage {
    fn get_number(id: &str, line: &str) -> Result<usize> {
        {
            line.split_whitespace()
                .nth(1)
                .ok_or_else(|| Error(String::from("Second column not found.")))?
                .parse()
                .map_err(|x| Error(format!("{}", x)))
        }
        .map_err(|x| {
            Error(format!(
                "Could not parse {} from {}: {}",
                id, MEMINFO_PATH, x
            ))
        })
    }
}

impl TryFrom<&config::Check> for MemoryUsage {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, self::Error> {
        if let config::CheckType::MemoryUsage = &check.type_ {
            Ok(Self {
                id: vec![String::from("Memory")],
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for MemoryUsage {
    type Item = u8;

    async fn get_data(&self) -> Result<Vec<Self::Item>> {
        let mut file = tokio::fs::File::open(MEMINFO_PATH).await.map_err(|x| {
            Error(format!(
                "Could not open {} for reading: {}",
                MEMINFO_PATH, x
            ))
        })?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|x| Error(format!("Could not read from {}: {}", MEMINFO_PATH, x)))?;
        let mut mem_total: Option<usize> = None;
        let mut mem_available: Option<usize> = None;
        for line in buffer.lines() {
            let line = line.map_err(|x| Error(format!("Error reading line: {}", x)))?;
            if line.starts_with("MemTotal") {
                mem_total = Some(Self::get_number("MemTotal", &line)?);
            } else if line.starts_with("MemAvailable") {
                mem_available = Some(Self::get_number("MemAvailable", &line)?);
            }
            if let (Some(mem_total), Some(mem_available)) = (mem_total, mem_available) {
                let usage = ((mem_total - mem_available) * 100 / mem_total) as u8;
                log::debug!("Memory usage is {}%", usage);
                return Ok(vec![usage]);
            }
        }
        Err(Error(format!("Failed to parse {}.", MEMINFO_PATH)))
    }

    fn measurement_ids(&self) -> &[String] {
        &self.id[..]
    }
}
