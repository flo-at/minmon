use super::DataSource;
use crate::config;
use crate::{Error, Result};
use async_trait::async_trait;

const PRESSURE_CPU_PATH: &str = "/proc/pressure/cpu";
const PRESSURE_IO_PATH: &str = "/proc/pressure/io";
const PRESSURE_MEMORY_PATH: &str = "/proc/pressure/memory";

static PARSE_ERROR: &str = "Could not parse pressure file.";

pub struct PressureAverage {
    id: Vec<String>,
    cpu: bool,
    io: config::PressureChoice,
    memory: config::PressureChoice,
    avg10: bool,
    avg60: bool,
    avg300: bool,
}

impl PressureAverage {
    fn add_data_from_line(&self, line: &PressureFileLine, res: &mut Vec<Result<u8>>) {
        if self.avg10 {
            res.push(Ok(line.avg10));
        }
        if self.avg60 {
            res.push(Ok(line.avg60));
        }
        if self.avg300 {
            res.push(Ok(line.avg300));
        }
    }

    fn add_data_from_error(&self, error: &Error, res: &mut Vec<Result<u8>>) {
        if self.avg10 {
            res.push(Err(error.clone()));
        }
        if self.avg60 {
            res.push(Err(error.clone()));
        }
        if self.avg300 {
            res.push(Err(error.clone()));
        }
    }

    async fn add_data_from_file(
        &self,
        choice: config::PressureChoice,
        path: &str,
        res: &mut Vec<Result<u8>>,
    ) {
        if choice != config::PressureChoice::None {
            match PressureFileContent::try_from_file(path).await {
                Ok(pressure) => {
                    if choice == config::PressureChoice::Some
                        || choice == config::PressureChoice::Both
                    {
                        self.add_data_from_line(&pressure.some, res);
                    }
                    if choice == config::PressureChoice::Full
                        || choice == config::PressureChoice::Both
                    {
                        match pressure.full {
                            Some(full) => self.add_data_from_line(&full, res),
                            _ => self.add_data_from_error(&Error(PARSE_ERROR.to_string()), res),
                        }
                    }
                }
                Err(err) => {
                    self.add_data_from_error(&err, res);
                    if choice == config::PressureChoice::Both {
                        self.add_data_from_error(&err, res);
                    }
                }
            }
        }
    }
}

impl TryFrom<&config::Check> for PressureAverage {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, self::Error> {
        if let config::CheckType::PressureAverage(pressure) = &check.type_ {
            if !pressure.cpu
                && pressure.io == config::PressureChoice::None
                && pressure.memory == config::PressureChoice::None
            {
                Err(Error(String::from(
                    "At least one of 'cpu', 'io', or 'memory' needs to be enabled.",
                )))
            } else if !pressure.avg10 && !pressure.avg60 && !pressure.avg300 {
                Err(Error(String::from(
                    "At least one of 'avg10', 'avg60', or 'avg300' needs to be enabled.",
                )))
            } else {
                let mut avg_ids = Vec::new();
                if pressure.avg10 {
                    avg_ids.push(String::from("avg10"));
                }
                if pressure.avg60 {
                    avg_ids.push(String::from("avg60"));
                }
                if pressure.avg300 {
                    avg_ids.push(String::from("avg300"));
                }
                let mut id = Vec::new();
                if pressure.cpu {
                    for avg_id in avg_ids.iter() {
                        id.push(format!("cpu/{}", avg_id));
                    }
                }
                match pressure.io {
                    config::PressureChoice::Some => {
                        for avg_id in avg_ids.iter() {
                            id.push(format!("io/some/{}", avg_id));
                        }
                    }
                    config::PressureChoice::Full => {
                        for avg_id in avg_ids.iter() {
                            id.push(format!("io/full/{}", avg_id));
                        }
                    }
                    config::PressureChoice::Both => {
                        for avg_id in avg_ids.iter() {
                            id.push(format!("io/some/{}", avg_id));
                            id.push(format!("io/full/{}", avg_id));
                        }
                    }
                    config::PressureChoice::None => {}
                }
                match pressure.memory {
                    config::PressureChoice::Some => {
                        for avg_id in avg_ids.iter() {
                            id.push(format!("memory/some/{}", avg_id));
                        }
                    }
                    config::PressureChoice::Full => {
                        for avg_id in avg_ids.iter() {
                            id.push(format!("memory/full/{}", avg_id));
                        }
                    }
                    config::PressureChoice::Both => {
                        for avg_id in avg_ids.iter() {
                            id.push(format!("memory/some/{}", avg_id));
                            id.push(format!("memory/full/{}", avg_id));
                        }
                    }
                    config::PressureChoice::None => {}
                }
                Ok(Self {
                    id,
                    cpu: pressure.cpu,
                    io: pressure.io,
                    memory: pressure.memory,
                    avg10: pressure.avg10,
                    avg60: pressure.avg60,
                    avg300: pressure.avg300,
                })
            }
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for PressureAverage {
    type Item = u8;

    async fn get_data(&self) -> Result<Vec<Result<Self::Item>>> {
        let mut res = Vec::new();
        if self.cpu {
            self.add_data_from_file(config::PressureChoice::Some, PRESSURE_CPU_PATH, &mut res)
                .await;
        }

        self.add_data_from_file(self.io, PRESSURE_IO_PATH, &mut res)
            .await;

        self.add_data_from_file(self.memory, PRESSURE_MEMORY_PATH, &mut res)
            .await;

        Ok(res)
    }

    fn format_data(data: &Self::Item) -> String {
        format!("pressure level {}%", data)
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}

struct PressureFileContent {
    some: PressureFileLine,
    full: Option<PressureFileLine>,
}

impl PressureFileContent {
    async fn try_from_file(path: &str) -> Result<Self> {
        let buffer = tokio::fs::read_to_string(path)
            .await
            .map_err(|x| Error(format!("Could not open {} for reading: {}", path, x)))?;
        Self::try_from(&*buffer)
    }
}

impl TryFrom<&str> for PressureFileContent {
    type Error = Error;

    fn try_from(text: &str) -> std::result::Result<Self, self::Error> {
        let mut some: Option<PressureFileLine> = None;
        let mut full: Option<PressureFileLine> = None;
        for line in text.lines() {
            let parsed_line = PressureFileLine::try_from(line)?;
            match parsed_line.label {
                PressureLabel::Some => some = Some(parsed_line),
                PressureLabel::Full => full = Some(parsed_line),
            }
        }
        Ok(Self {
            some: some.ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
            full,
        })
    }
}

struct PressureFileLine {
    label: PressureLabel,
    avg10: u8,
    avg60: u8,
    avg300: u8,
}

impl PressureFileLine {
    fn parse_avg_str(avg_str: &str, label: &str) -> Result<u8> {
        let mut parts = avg_str.split('=');
        if parts.next().ok_or_else(|| Error(PARSE_ERROR.to_string()))? != label {
            return Err(Error(PARSE_ERROR.to_string()));
        }
        match parts.next() {
            Some(avg_str) => avg_str
                .parse::<f32>()
                .map(|x| x as u8)
                .map_err(|x| Error(x.to_string())),
            _ => Err(Error(PARSE_ERROR.into())),
        }
    }
}

impl TryFrom<&str> for PressureFileLine {
    type Error = Error;

    fn try_from(line: &str) -> std::result::Result<Self, self::Error> {
        let mut parts = line.split_whitespace();
        let label = match parts.next() {
            Some("some") => Ok(PressureLabel::Some),
            Some("full") => Ok(PressureLabel::Full),
            _ => Err(Error(PARSE_ERROR.into())),
        }?;
        let avg10 = Self::parse_avg_str(
            parts.next().ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
            "avg10",
        )?;
        let avg60 = Self::parse_avg_str(
            parts.next().ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
            "avg60",
        )?;
        let avg300 = Self::parse_avg_str(
            parts.next().ok_or_else(|| Error(PARSE_ERROR.to_string()))?,
            "avg300",
        )?;
        Ok(PressureFileLine {
            label,
            avg10,
            avg60,
            avg300,
        })
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
enum PressureLabel {
    Some,
    Full,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pressure_file_line_from_str() {
        let line_str = "some avg10=1.00 avg60=2.00 avg300=3.99 total=4";
        let line = PressureFileLine::try_from(line_str).unwrap();
        assert_eq!(line.label, PressureLabel::Some);
        assert_eq!(line.avg10, 1);
        assert_eq!(line.avg60, 2);
        assert_eq!(line.avg300, 3);
        let line_str = "full avg10=5.00 avg60=6.00 avg300=7.99 total=8";
        let line = PressureFileLine::try_from(line_str).unwrap();
        assert_eq!(line.label, PressureLabel::Full);
        assert_eq!(line.avg10, 5);
        assert_eq!(line.avg60, 6);
        assert_eq!(line.avg300, 7);
    }

    #[test]
    fn test_pressure_file_content_from_str() {
        let content_str = "some avg10=1.00 avg60=2.00 avg300=3.99 total=4\n\
                           full avg10=5.00 avg60=6.00 avg300=7.99 total=8";
        let content = PressureFileContent::try_from(content_str).unwrap();
        assert!(content.full.is_some());
    }
}
