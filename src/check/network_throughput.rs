use super::DataSource;
use crate::{config, measurement};
use crate::{Error, Result};
use async_trait::async_trait;
use measurement::Measurement;

type Item = measurement::DataSize;

pub struct NetworkThroughput {
    id: Vec<String>,
    interfaces: Vec<String>,
    received: bool,
    sent: bool,
    log_format: config::DataSizeFormat,
    last_received: Vec<WrappingItem>,
    last_sent: Vec<WrappingItem>,
}

impl NetworkThroughput {
    pub async fn bytes_from_file(path: &str) -> Result<Item> {
        let buffer = tokio::fs::read_to_string(path)
            .await
            .map_err(|x| Error(format!("Could not open {path} for reading: {x}")))?;
        Item::new(
            buffer
                .trim()
                .parse::<u64>()
                .map_err(|x| Error(format!("Could not parse interface statistics file: {x}")))?,
        )
    }
}

impl TryFrom<&config::Check> for NetworkThroughput {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
        if let config::CheckType::NetworkThroughput(network_throughput) = &check.type_ {
            if !network_throughput.received && !network_throughput.sent {
                Err(Error(String::from(
                    "At least one of 'received' or 'sent' needs to be enabled.",
                )))
            } else {
                let mut id = Vec::new();
                for interface in network_throughput.interfaces.iter() {
                    if network_throughput.sent {
                        id.push(format!("{interface}[rx]"));
                    }
                    if network_throughput.received {
                        id.push(format!("{interface}[tx]"));
                    }
                }
                Ok(Self {
                    id,
                    interfaces: network_throughput.interfaces.clone(),
                    received: network_throughput.received,
                    sent: network_throughput.sent,
                    log_format: network_throughput.log_format,
                    last_received: vec![
                        WrappingItem::default();
                        network_throughput.interfaces.len()
                    ],
                    last_sent: vec![WrappingItem::default(); network_throughput.interfaces.len()],
                })
            }
        } else {
            panic!();
        }
    }
}

#[derive(Clone, Default)]
struct WrappingItem {
    last: Option<Item>,
}

impl WrappingItem {
    pub fn update(&mut self, data: Item) -> Option<Item> {
        let res = match self.last {
            Some(last) => {
                if data < last {
                    // wrap-around detected
                    Some(Item::MAX - last + Item::new(1).unwrap() + data)
                } else {
                    Some(data - last)
                }
            }
            None => None,
        };
        self.last = Some(data);
        res
    }
}

#[async_trait]
impl DataSource for NetworkThroughput {
    type Item = Item;

    async fn get_data(&mut self) -> Result<Vec<Result<Option<Self::Item>>>> {
        let mut res = Vec::new();
        for (last_sent, (last_received, interface)) in self
            .last_sent
            .iter_mut()
            .zip(self.last_received.iter_mut().zip(self.interfaces.iter()))
        {
            if self.received {
                res.push(
                    Self::bytes_from_file(&format!(
                        "/sys/class/net/{interface}/statistics/rx_bytes"
                    ))
                    .await
                    .map(|x| last_received.update(x)),
                );
            }
            if self.sent {
                res.push(
                    Self::bytes_from_file(&format!(
                        "/sys/class/net/{interface}/statistics/tx_bytes"
                    ))
                    .await
                    .map(|x| last_sent.update(x)),
                );
            }
        }
        Ok(res)
    }

    fn format_data(&self, data: &Self::Item) -> String {
        let throughput = match self.log_format {
            config::DataSizeFormat::Binary => data.as_string_binary(),
            config::DataSizeFormat::Decimal => data.as_string_decimal(),
            config::DataSizeFormat::Bytes => format!("{data}"),
        };
        format!("throughput {throughput}")
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_wrapping_item() {
        let mut wrapping = WrappingItem::default();
        let value = wrapping.update(Item::new(123).unwrap());
        assert_eq!(None, value);
        let value = wrapping.update(Item::new(234).unwrap());
        assert_eq!(Some(Item::new(111).unwrap()), value);
        wrapping.update(Item::new(u64::MAX - 10).unwrap());
        let value = wrapping.update(Item::new(10).unwrap());
        assert_eq!(Some(Item::new(21).unwrap()), value);
    }
}
