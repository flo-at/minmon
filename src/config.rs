use crate::{Error, PlaceholderMap};
use std::collections::HashMap;

use serde::Deserialize;

// TODO check syntactically valid but semantically invalid values (like interval: 0)

trait Validate {
    fn validate(&self) -> bool;
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub log: Log,
    #[serde(default)]
    pub report: Report,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub checks: Vec<Check>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Log {
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default)]
    pub target: LogTarget,
}

#[derive(Default, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warning,
    Error,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        log::Level::from(level).to_level_filter()
    }
}

impl From<LogLevel> for log::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Info => log::Level::Info,
            LogLevel::Warning => log::Level::Warn,
            LogLevel::Error => log::Level::Error,
        }
    }
}

#[derive(Default, Deserialize, PartialEq, Debug)]
pub enum LogTarget {
    #[default]
    Stdout,
    #[cfg(feature = "systemd")]
    Journal,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[serde(default)]
    pub disable: bool,
    #[serde(default = "default::report_interval")]
    pub interval: u32,
    #[serde(default)]
    pub events: Vec<ReportEvent>,
}

// TODO maybe move into default module
impl Default for Report {
    fn default() -> Self {
        Self {
            // TODO improve this.. s.th. lile ..Default::default()
            // https://stackoverflow.com/questions/69712973/only-setting-one-field-in-a-rust-default-implementation
            disable: bool::default(),
            interval: default::REPORT_INTERVAL,
            events: Vec::new(),
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ReportEvent {
    pub action: String,
}

#[derive(Deserialize)]
pub struct Action {
    #[serde(default)]
    pub disable: bool,
    pub name: String,
    #[serde(default)]
    pub placeholders: PlaceholderMap,
    #[serde(flatten)]
    pub type_: ActionType,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum ActionType {
    WebHook(ActionWebHook),
    Log(ActionLog),
    Process(ActionProcess),
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ActionType::WebHook(_) => write!(f, "WebHook"),
            ActionType::Log(_) => write!(f, "Log"),
            ActionType::Process(_) => write!(f, "Process"),
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActionWebHook {
    pub url: String,
    pub method: HttpMethod,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default = "default::action_web_hook_timeout")]
    pub timeout: u32,
    #[serde(default)]
    pub body: String,
}

#[derive(Deserialize, PartialEq, Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActionLog {
    #[serde(default)]
    pub level: LogLevel,
    pub template: String,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActionProcess {
    #[serde(default)]
    pub path: std::path::PathBuf,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub environment_variables: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub working_directory: Option<String>,
    #[serde(default)]
    pub uid: Option<u32>,
    #[serde(default)]
    pub gid: Option<u32>,
}

#[derive(Deserialize)]
pub struct Check {
    #[serde(default)]
    pub disable: bool,
    #[serde(default = "default::check_interval")]
    pub interval: u32,
    pub name: String,
    #[serde(default)]
    pub placeholders: PlaceholderMap,
    #[serde(flatten)]
    pub type_: CheckType,
    #[serde(default)]
    pub alarms: Vec<Alarm>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum CheckType {
    FilesystemUsage(CheckFilesystemUsage),
    MemoryUsage(CheckMemoryUsage),
}

impl std::fmt::Display for CheckType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CheckType::FilesystemUsage(_) => write!(f, "FilesystemUsage"),
            CheckType::MemoryUsage(_) => write!(f, "MemoryUsage"),
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckFilesystemUsage {
    #[serde(default)]
    pub mountpoints: Vec<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckMemoryUsage {
    #[serde(default = "default::check_memory_usage_memory")]
    pub memory: bool,
    #[serde(default = "default::check_memory_usage_swap")]
    pub swap: bool,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Alarm {
    #[serde(default)]
    pub disable: bool,
    pub name: String,
    pub action: String,
    #[serde(default = "default::check_alarm_cycles")]
    pub cycles: u32,
    #[serde(default)]
    pub repeat_cycles: u32,
    #[serde(default)]
    pub recover_action: String,
    #[serde(default = "default::check_alarm_recover_cycles")]
    pub recover_cycles: u32,
    #[serde(default)]
    pub error_action: String,
    #[serde(default)]
    pub error_repeat_cycles: u32,
    #[serde(default)]
    pub placeholders: PlaceholderMap,
    #[serde(flatten)]
    pub type_: AlarmType,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum AlarmType {
    Level(AlarmLevel),
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AlarmLevel {
    pub level: u8,
}

mod default {
    pub const REPORT_INTERVAL: u32 = 604800;
    pub fn report_interval() -> u32 {
        REPORT_INTERVAL
    }

    pub const ACTION_WEB_HOOK_TIMEOUT: u32 = 10;
    pub fn action_web_hook_timeout() -> u32 {
        ACTION_WEB_HOOK_TIMEOUT
    }

    pub const CHECK_INTERVAL: u32 = 300;
    pub fn check_interval() -> u32 {
        CHECK_INTERVAL
    }

    pub const CHECK_ALARM_CYCLES: u32 = 1;
    pub fn check_alarm_cycles() -> u32 {
        CHECK_ALARM_CYCLES
    }

    pub const CHECK_ALARM_RECOVER_CYCLES: u32 = 1;
    pub fn check_alarm_recover_cycles() -> u32 {
        CHECK_ALARM_RECOVER_CYCLES
    }

    pub const CHECK_MEMORY_USAGE_MEMORY: bool = true;
    pub fn check_memory_usage_memory() -> bool {
        CHECK_MEMORY_USAGE_MEMORY
    }

    pub const CHECK_MEMORY_USAGE_SWAP: bool = false;
    pub fn check_memory_usage_swap() -> bool {
        CHECK_MEMORY_USAGE_SWAP
    }
}

impl TryFrom<&str> for Config {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        toml::from_str(text).map_err(|x| Error(format!("{}", x)))
    }
}

impl TryFrom<&std::path::Path> for Config {
    type Error = Error;

    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(|x| Error(format!("{}", x)))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|x| Error(format!("{}", x)))?;
        Config::try_from(content.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from_str_defaults() {
        let text = r#"
        "#;
        let config = Config::try_from(text).unwrap();
        assert_eq!(config.log.level, LogLevel::default());
        assert_eq!(config.log.target, LogTarget::default());
        assert!(!config.report.disable);
        assert_eq!(config.report.interval, default::REPORT_INTERVAL);
        assert_eq!(config.report.events.len(), 0);
        assert_eq!(config.actions.len(), 0);
        assert_eq!(config.checks.len(), 0);
    }

    #[test]
    #[cfg(feature = "systemd")]
    fn test_from_str_non_defaults() {
        let text = r#"
            [log]
            level = "Error"
            target = "Journal"

            [report]
            disable = true
            interval = 12345

            [[report.events]]
            action = "report-action"

            [[actions]]
            disable = true
            name = "test-action"
            type = "WebHook"
            url = "http://example.com/webhook"
            method = "GET"
            headers = {"Content-Type" = "application/json"}
            timeout = 5
            body = """{"name": "{{ name }}"}"""

            [[checks]]
            disable = true
            name = "test-check"
            type = "FilesystemUsage"
            mountpoints = ["/home", "/srv"]

            [[checks.alarms]]
            disable = true
            name = "test-alarm"
            level = 75
            cycles = 3
            action = "test-action"
            repeat_cycles = 600
            recover_cycles = 4
            recover_action = "test-action"
        "#;
        let config = Config::try_from(text).unwrap();
        assert_eq!(config.log.target, LogTarget::Journal);
        assert_eq!(config.log.level, LogLevel::Error);
        assert!(config.report.disable);
        assert_eq!(config.report.interval, 12345);

        assert_eq!(config.actions.len(), 1);
        let action = config.actions.first().unwrap();
        assert!(action.disable);
        assert_eq!(action.name, "test-action");
        assert_eq!(
            action.type_,
            ActionType::WebHook(ActionWebHook {
                url: String::from("http://example.com/webhook"),
                method: HttpMethod::GET,
                headers: HashMap::from([(
                    String::from("Content-Type"),
                    String::from("application/json")
                )]),
                timeout: 5,
                body: String::from(r#"{"name": "{{ name }}"}"#),
            })
        );

        assert_eq!(config.checks.len(), 1);
        let check = config.checks.first().unwrap();
        assert!(check.disable);
        assert_eq!(check.name, "test-check");
        assert_eq!(
            check.type_,
            CheckType::FilesystemUsage(CheckFilesystemUsage {
                mountpoints: vec![String::from("/home"), String::from("/srv")]
            })
        );

        assert_eq!(check.alarms.len(), 1);
        let alarm = check.alarms.first().unwrap();
        assert!(alarm.disable);
        assert_eq!(alarm.name, "test-alarm");
        assert_eq!(alarm.type_, AlarmType::Level(AlarmLevel { level: 75 }));
        assert_eq!(alarm.cycles, 3);
        assert_eq!(alarm.repeat_cycles, 600);
        assert_eq!(alarm.action, "test-action");
        assert_eq!(alarm.recover_cycles, 4);
        assert_eq!(alarm.recover_action, "test-action");
    }
}
