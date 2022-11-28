use std::io::BufRead;

use super::DataSource;
use crate::config;
use crate::{Error, Result};
use async_trait::async_trait;
use tokio::io::AsyncReadExt;

const MEMINFO_PATH: &str = "/proc/meminfo";

pub struct MemoryUsage {
    id: Vec<String>,
    memory: bool,
    swap: bool,
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
        if let config::CheckType::MemoryUsage(memory_usage) = &check.type_ {
            if !memory_usage.memory && !memory_usage.swap {
                Err(Error(String::from(
                    "Either 'memory' or 'swap' or both need to be enabled.",
                )))
            } else {
                let mut id = Vec::new();
                if memory_usage.memory {
                    id.push(String::from("Memory"));
                }
                if memory_usage.swap {
                    id.push(String::from("Swap"));
                }
                Ok(Self {
                    id,
                    memory: memory_usage.memory,
                    swap: memory_usage.swap,
                })
            }
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for MemoryUsage {
    type Item = u8;

    async fn get_data(&self) -> Result<Vec<Result<Self::Item>>> {
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
        let mut mem_usage: Option<u8> = None;
        let mut swap_total: Option<usize> = None;
        let mut swap_free: Option<usize> = None;
        let mut swap_usage: Option<u8> = None;
        for line in buffer.lines() {
            let line = line.map_err(|x| Error(format!("Error reading line: {}", x)))?;
            if self.memory {
                if line.starts_with("MemTotal") {
                    mem_total = Some(Self::get_number("MemTotal", &line)?);
                } else if line.starts_with("MemAvailable") {
                    mem_available = Some(Self::get_number("MemAvailable", &line)?);
                }
            }
            if self.swap {
                if line.starts_with("SwapTotal") {
                    swap_total = Some(Self::get_number("SwapTotal", &line)?);
                } else if line.starts_with("SwapFree") {
                    swap_free = Some(Self::get_number("SwapFree", &line)?);
                }
            }
        }
        if let (Some(mem_total), Some(mem_available)) = (mem_total, mem_available) {
            if mem_total != 0 {
                mem_usage = Some(((mem_total - mem_available) * 100 / mem_total) as u8);
            }
        }
        if let (Some(swap_total), Some(swap_free)) = (swap_total, swap_free) {
            if swap_total != 0 {
                swap_usage = Some(((swap_total - swap_free) * 100 / swap_total) as u8);
            }
        }
        let mut res = Vec::new();
        if self.memory {
            res.push(mem_usage.ok_or_else(|| Error(String::from("Could not parse memory usage."))));
        }
        if self.swap {
            res.push(swap_usage.ok_or_else(|| Error(String::from("Could not parse swap usage."))));
        }
        Ok(res)
    }

    fn measurement_ids(&self) -> &[String] {
        &self.id[..]
    }
}
