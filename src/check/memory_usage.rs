use super::DataSource;
use crate::{config, measurement};
use crate::{Error, Result};
use async_trait::async_trait;
use measurement::Measurement;

const MEMINFO_PATH: &str = "/proc/meminfo";

static PARSE_ERROR: &str = "Could not parse meminfo file.";

pub struct MemoryUsage {
    id: Vec<String>,
    memory: bool,
    swap: bool,
}

impl TryFrom<&config::Check> for MemoryUsage {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
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
    type Item = measurement::Level;

    async fn get_data(&mut self) -> Result<Vec<Result<Option<Self::Item>>>> {
        let meminfo = MeminfoFileContent::try_from_file(MEMINFO_PATH).await?;
        let mut res = Vec::new();
        if self.memory {
            res.push(if meminfo.mem_total != 0 {
                Ok(((meminfo.mem_total - meminfo.mem_available) * 100 / meminfo.mem_total) as u8)
                    .and_then(Self::Item::new)
                    .map(Some)
            } else {
                Err(Error(String::from("Could not read memory usage.")))
            });
        }
        if self.swap {
            res.push(if meminfo.swap_total != 0 {
                Ok(((meminfo.swap_total - meminfo.swap_free) * 100 / meminfo.swap_total) as u8)
                    .and_then(Self::Item::new)
                    .map(Some)
            } else {
                Err(Error(String::from("Could not read swap usage.")))
            });
        }
        Ok(res)
    }

    fn format_data(&self, data: &Self::Item) -> String {
        format!("usage level {data}")
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}

struct MeminfoFileContent {
    mem_total: usize,
    mem_available: usize,
    swap_total: usize,
    swap_free: usize,
}

impl MeminfoFileContent {
    async fn try_from_file(path: &str) -> Result<Self> {
        let buffer = tokio::fs::read_to_string(path)
            .await
            .map_err(|x| Error(format!("Could not open {path} for reading: {x}")))?;
        Self::try_from(&*buffer)
    }

    fn get_number(id: &str, line: &str) -> Result<usize> {
        crate::get_number(&format!("Could not read {id} from {MEMINFO_PATH}"), line, 1)
    }
}

impl TryFrom<&str> for MeminfoFileContent {
    type Error = Error;

    fn try_from(text: &str) -> std::result::Result<Self, Self::Error> {
        let mut mem_total: Option<usize> = None;
        let mut mem_available: Option<usize> = None;
        let mut swap_total: Option<usize> = None;
        let mut swap_free: Option<usize> = None;
        for line in text.lines() {
            if line.starts_with("MemTotal") {
                mem_total = Some(Self::get_number("MemTotal", line)?);
            } else if line.starts_with("MemAvailable") {
                mem_available = Some(Self::get_number("MemAvailable", line)?);
            } else if line.starts_with("SwapTotal") {
                swap_total = Some(Self::get_number("SwapTotal", line)?);
            } else if line.starts_with("SwapFree") {
                swap_free = Some(Self::get_number("SwapFree", line)?);
            }
        }
        Ok(Self {
            mem_total: mem_total.ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
            mem_available: mem_available.ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
            swap_total: swap_total.ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
            swap_free: swap_free.ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_meminfo_file_content_from_str() {
        let content_str = "MemTotal:           1234 kB\n\
                           MemFree:            2345 kB\n\
                           MemAvailable:       3456 kB\n\
                           Cached:             4567 kB\n\
                           SwapTotal:          5678 kB\n\
                           SwapFree:           6789 kB";
        let content = MeminfoFileContent::try_from(content_str).unwrap();
        assert_eq!(content.mem_total, 1234);
        assert_eq!(content.mem_available, 3456);
        assert_eq!(content.swap_total, 5678);
        assert_eq!(content.swap_free, 6789);
    }
}
