use super::DataSource;
use crate::process::ProcessConfig;
use crate::{config, measurement};
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;
use measurement::Measurement;
use regex::Regex;

pub struct ProcessOutputMatch {
    id: Vec<String>,
    process_config: ProcessConfig,
    output_source: config::OutputSource,
    output_regex: Regex,
    invert_match: bool,
}

impl TryFrom<&config::Check> for ProcessOutputMatch {
    type Error = Error;

    fn try_from(check: &config::Check) -> std::result::Result<Self, Self::Error> {
        if let config::CheckType::ProcessOutputMatch(process_output_match) = &check.type_ {
            let output_regex = Regex::new(&process_output_match.output_regex)
                .map_err(|x| Error(format!("Could not parse output regex: {x}")))?;
            let process_config = ProcessConfig::try_from(&process_output_match.process_config)?;
            Ok(Self {
                id: vec![process_config.file_name().map(|x| x.into())?],
                process_config,
                output_source: process_output_match.output_source.clone(),
                output_regex,
                invert_match: process_output_match.invert_match,
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl DataSource for ProcessOutputMatch {
    type Item = measurement::BinaryState;

    async fn get_data(
        &mut self,
        placeholders: &mut PlaceholderMap,
    ) -> Result<Vec<Result<Option<Self::Item>>>> {
        let result = self.process_config.run(None).await?;
        let output_str = match &self.output_source {
            config::OutputSource::Stdout => &result.stdout,
            config::OutputSource::Stderr => &result.stderr,
        };
        let res = if let Some(captures) = self.output_regex.captures(output_str) {
            for (i, capture) in captures.iter().enumerate() {
                if let Some(value) = capture {
                    placeholders.insert(format!("capture[{i}]"), value.as_str().to_owned());
                }
            }
            true ^ self.invert_match
        } else {
            false ^ self.invert_match
        };
        placeholders.insert(String::from("stdout"), result.stdout);
        placeholders.insert(String::from("stderr"), result.stderr);
        Ok(vec![Self::Item::new(res).map(Some)])
    }

    fn format_data(&self, data: &Self::Item) -> String {
        match data.data() ^ self.invert_match {
            true => "output matched",
            false => "output did not match",
        }
        .into()
    }

    fn ids(&self) -> &[String] {
        &self.id[..]
    }
}
