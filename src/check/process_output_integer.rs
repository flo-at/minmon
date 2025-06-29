use super::DataSource;
use crate::process::ProcessConfig;
use crate::{config, measurement};
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;
use measurement::Measurement;
use regex::Regex;

pub struct ProcessOutputInteger {
    id: Vec<String>,
    process_config: ProcessConfig,
    output_source: config::OutputSource,
    output_regex: Option<Regex>,
}

impl TryFrom<&config::Check> for ProcessOutputInteger {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
        if let config::CheckType::ProcessOutputInteger(process_output_integer) = &check.type_ {
            let output_regex = process_output_integer
                .output_regex
                .as_ref()
                .map(|x| Regex::new(x))
                .transpose()
                .map_err(|x| Error(format!("Could not parse output regex: {x}")))?;
            if output_regex.as_ref().is_some_and(|x| x.captures_len() != 2) {
                return Err(Error(String::from(
                    "Output regex must have exactly one capture group.",
                )));
            }
            let process_config = ProcessConfig::try_from(&process_output_integer.process_config)?;
            Ok(Self {
                id: vec![process_config.file_name().map(|x| x.into())?],
                process_config,
                output_source: process_output_integer.output_source.clone(),
                output_regex,
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for ProcessOutputInteger {
    type Item = measurement::Integer;

    async fn get_data(
        &mut self,
        placeholders: &mut PlaceholderMap,
    ) -> Result<Vec<Result<Option<Self::Item>>>> {
        let result = self.process_config.run(None).await?;
        let output_str = match &self.output_source {
            config::OutputSource::Stdout => &result.stdout,
            config::OutputSource::Stderr => &result.stderr,
        };
        let output_str = if let Some(ref regex) = self.output_regex {
            if let Some(captures) = regex.captures(output_str) {
                placeholders.insert(
                    String::from("regex_match"),
                    captures.get(0).unwrap().as_str().to_owned(),
                );
                Ok(captures.get(1).unwrap().as_str().to_owned())
            } else {
                Err(Error(String::from(
                    "Output did not match the regex pattern.",
                )))
            }
        } else {
            Ok(output_str.clone())
        }?;
        let output_int = output_str
            .trim()
            .parse::<i64>()
            .map_err(|x| Error(format!("Could not parse output string into integer: {x}")))?;
        placeholders.insert(String::from("stdout"), result.stdout);
        placeholders.insert(String::from("stderr"), result.stderr);
        Ok(vec![Self::Item::new(output_int).map(Some)])
    }

    fn format_data(&self, data: &Self::Item) -> String {
        format!("integer output {data}")
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}
