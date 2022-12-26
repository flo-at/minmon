use crate::config;
use crate::{Error, PlaceholderMap, Result};

pub struct ProcessConfig {
    path: std::path::PathBuf,
    arguments: Vec<String>,
    environment_variables: std::collections::HashMap<String, String>,
    working_directory: Option<String>,
    uid: Option<u32>,
    gid: Option<u32>,
}

impl ProcessConfig {
    pub fn file_name(&self) -> Result<&str> {
        let error_str = "Could not parse file name from path.";
        self.path
            .file_name()
            .ok_or_else(|| Error(error_str.into()))?
            .to_str()
            .ok_or_else(|| Error(error_str.into()))
    }

    pub async fn run(&self, placeholders: Option<PlaceholderMap>) -> Result<(u8, Option<String>)> {
        let mut command = tokio::process::Command::new(&self.path);
        if let Some(placeholders) = placeholders {
            for argument in self.arguments.iter() {
                let argument = crate::fill_placeholders(argument.as_str(), &placeholders);
                command.arg(argument);
            }
            for (name, value) in self.environment_variables.iter() {
                let name = crate::fill_placeholders(name.as_str(), &placeholders);
                let value = crate::fill_placeholders(value.as_str(), &placeholders);
                command.env(name, value);
            }
        } else {
            command.args(&self.arguments);
            command.envs(&self.environment_variables);
        }
        if let Some(working_directory) = &self.working_directory {
            command.current_dir(working_directory);
        }
        if let Some(uid) = self.uid {
            command.uid(uid);
        }
        if let Some(gid) = self.gid {
            command.gid(gid);
        }
        log::debug!("Calling process: {}", self.path.display());
        let output = command
            .output()
            .await
            .map_err(|x| Error(format!("Failed to run process: {x}")))?;

        match output.status.code() {
            Some(code) => {
                let code = (code & 0xff) as u8;
                if output.stderr.is_empty() {
                    Ok((code, None))
                } else {
                    Ok((
                        code,
                        std::str::from_utf8(&output.stderr[..])
                            .map(|x| x.into())
                            .ok(),
                    ))
                }
            }
            None => Err(Error(String::from("Process was terminated by a signal."))),
        }
    }
}

impl TryFrom<&config::ProcessConfig> for ProcessConfig {
    type Error = Error;

    fn try_from(process: &config::ProcessConfig) -> std::result::Result<Self, self::Error> {
        if !process.path.is_file() {
            Err(Error(format!(
                "'path' is not a file: {}.",
                process.path.display()
            )))
        } else {
            Ok(Self {
                path: process.path.clone(),
                arguments: process.arguments.clone(),
                environment_variables: process.environment_variables.clone(),
                working_directory: process.working_directory.clone(),
                uid: process.uid,
                gid: process.gid,
            })
        }
    }
}
