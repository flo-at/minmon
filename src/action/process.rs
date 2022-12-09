use super::Action;
use crate::config;
use crate::{Error, PlaceholderMap, Result};
use async_trait::async_trait;

pub struct Process {
    path: std::path::PathBuf,
    arguments: Vec<String>,
    environment_variables: std::collections::HashMap<String, String>,
    working_directory: Option<String>,
    uid: Option<u32>,
    gid: Option<u32>,
}

impl TryFrom<&config::Action> for Process {
    type Error = Error;

    fn try_from(action: &config::Action) -> std::result::Result<Self, Self::Error> {
        if let config::ActionType::Process(process) = &action.type_ {
            Ok(Self {
                path: process.path.clone(),
                arguments: process.arguments.clone(),
                environment_variables: process.environment_variables.clone(),
                working_directory: process.working_directory.clone(),
                uid: process.uid,
                gid: process.gid,
            })
        } else {
            panic!();
        }
    }
}

#[async_trait]
impl Action for Process {
    async fn trigger(&self, placeholders: PlaceholderMap) -> Result<()> {
        let mut command = tokio::process::Command::new(&self.path);
        for argument in self.arguments.iter() {
            let argument = crate::fill_placeholders(argument.as_str(), &placeholders);
            command.arg(argument);
        }
        for (name, value) in self.environment_variables.iter() {
            let name = crate::fill_placeholders(name.as_str(), &placeholders);
            let value = crate::fill_placeholders(value.as_str(), &placeholders);
            command.env(name, value);
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
            .map_err(|x| Error(format!("Failed to run process: {}", x)))?;
        match output.status.code() {
            Some(0) => Ok(()),
            Some(code) => {
                if output.stderr.is_empty() {
                    Err(Error(format!("Process failed with code {}.", code)))
                } else {
                    Err(Error(format!(
                        "Process failed with code {}: {}",
                        code,
                        std::str::from_utf8(&output.stderr[..]).unwrap()
                    )))
                }
            }
            None => Err(Error(String::from("Process was terminated by a signal."))),
        }
    }
}
