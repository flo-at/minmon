use crate::{Error, PlaceholderMap};
use std::collections::HashMap;

use serde::Deserialize;

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
    pub placeholders: PlaceholderMap,
    #[serde(default)]
    pub events: Vec<ReportEvent>,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            disable: true,
            interval: default::REPORT_INTERVAL,
            events: Vec::new(),
            placeholders: PlaceholderMap::new(),
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ReportEvent {
    #[serde(default)]
    pub disable: bool,
    pub name: String,
    pub action: String,
    #[serde(default)]
    pub placeholders: PlaceholderMap,
}

#[derive(Deserialize)]
pub struct Action {
    #[serde(default)]
    pub disable: bool,
    pub name: String,
    #[serde(default = "default::action_timeout")]
    pub timeout: u32,
    #[serde(default)]
    pub placeholders: PlaceholderMap,
    #[serde(flatten)]
    pub type_: ActionType,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(tag = "type")]
pub enum ActionType {
    Email(ActionEmail),
    Log(ActionLog),
    Process(ActionProcess),
    Webhook(ActionWebhook),
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActionEmail {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub reply_to: Option<String>,
    pub subject: String,
    pub body: String,
    pub smtp_server: String,
    #[serde(default)]
    pub smtp_port: Option<u16>,
    #[serde(default)]
    pub smtp_security: SmtpSecurity,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, PartialEq, Debug, Clone, Copy, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum SmtpSecurity {
    #[default]
    TLS,
    STARTTLS,
    Plain,
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
    #[serde(flatten)]
    pub process_config: ProcessConfig,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActionWebhook {
    pub url: String,
    #[serde(default)]
    pub method: HttpMethod,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: String,
}

#[derive(Deserialize, PartialEq, Debug, Clone, Copy, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum HttpMethod {
    GET,
    #[default]
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(Deserialize)]
pub struct Check {
    #[serde(default)]
    pub disable: bool,
    #[serde(default = "default::check_interval")]
    pub interval: u32,
    pub name: String,
    #[serde(default = "default::check_timeout")]
    pub timeout: u32,
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
    PressureAverage(CheckPressureAverage),
    ProcessExitStatus(CheckProcessExitStatus),
    SystemdUnitStatus(CheckSystemdUnitStatus),
    #[cfg(feature = "sensors")]
    Temperature(CheckTemperature),
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckFilesystemUsage {
    pub mountpoints: Vec<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckMemoryUsage {
    #[serde(default)]
    pub memory: bool,
    #[serde(default)]
    pub swap: bool,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckPressureAverage {
    #[serde(default)]
    pub cpu: bool,
    #[serde(default)]
    pub io: PressureChoice,
    #[serde(default)]
    pub memory: PressureChoice,
    #[serde(default)]
    pub avg10: bool,
    #[serde(default)]
    pub avg60: bool,
    #[serde(default)]
    pub avg300: bool,
}

#[derive(Deserialize, PartialEq, Debug, Clone, Copy, Default)]
pub enum PressureChoice {
    #[default]
    None,
    Some,
    Full,
    Both,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckSystemdUnitStatus {
    pub units: Vec<SystemdUnitConfig>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum SystemdUnitConfig {
    System(String),
    User(SystemdUnitConfigUser),
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SystemdUnitConfigUser {
    pub unit: String,
    #[serde(default)]
    pub uid: u32,
}

impl SystemdUnitConfig {
    pub fn unit(&self) -> &str {
        match self {
            SystemdUnitConfig::System(unit) => unit,
            SystemdUnitConfig::User(config) => &config.unit,
        }
    }

    pub fn uid(&self) -> u32 {
        match self {
            SystemdUnitConfig::System(_) => 0,
            SystemdUnitConfig::User(config) => config.uid,
        }
    }
}

#[cfg(feature = "sensors")]
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckTemperature {
    pub sensors: Vec<SensorsId>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum SensorsId {
    Sensor(String),
    SensorWithLabel(SensorsIdLabel),
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub struct SensorsIdLabel {
    pub sensor: String,
    pub label: String,
}

impl SensorsId {
    pub fn sensor(&self) -> &str {
        match self {
            SensorsId::Sensor(sensor) => sensor,
            SensorsId::SensorWithLabel(config) => &config.sensor,
        }
    }

    pub fn label(&self) -> Option<&str> {
        match self {
            SensorsId::Sensor(_) => None,
            SensorsId::SensorWithLabel(config) => Some(&config.label),
        }
    }
}

impl std::fmt::Display for SensorsId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorsId::Sensor(sensor) => write!(f, "{sensor}"),
            SensorsId::SensorWithLabel(config) => write!(f, "{}[{}]", config.sensor, config.label),
        }
    }
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CheckProcessExitStatus {
    #[serde(flatten)]
    pub process_config: ProcessConfig,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
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

#[derive(Deserialize, PartialEq, Debug)]
pub struct Alarm {
    #[serde(default)]
    pub disable: bool,
    pub name: String,
    pub action: String,
    #[serde(default)]
    pub placeholders: PlaceholderMap,
    #[serde(default = "default::check_alarm_cycles")]
    pub cycles: u32,
    #[serde(default)]
    pub repeat_cycles: u32,
    #[serde(default)]
    pub recover_action: Option<String>,
    #[serde(default)]
    pub recover_placeholders: PlaceholderMap,
    #[serde(default = "default::check_alarm_recover_cycles")]
    pub recover_cycles: u32,
    #[serde(default)]
    pub error_action: Option<String>,
    #[serde(default)]
    pub error_placeholders: PlaceholderMap,
    #[serde(default)]
    pub error_repeat_cycles: u32,
    #[serde(default)]
    pub invert: bool,
    #[serde(flatten)]
    pub type_: AlarmType,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum AlarmType {
    Default(AlarmDefault),
    StatusCode(AlarmStatusCode),
    Level(AlarmLevel),
    #[cfg(feature = "sensors")]
    Temperature(AlarmTemperature),
}

// This is a dummy that is used if no alarm specific fields are found.
// Works only for alarms with only optional/defaulted fields.
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AlarmDefault {}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AlarmLevel {
    pub level: u8,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AlarmStatusCode {
    pub status_codes: Vec<u8>,
}

#[cfg(feature = "sensors")]
#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AlarmTemperature {
    pub temperature: i16,
}

mod default {
    pub const REPORT_INTERVAL: u32 = 604800;
    pub fn report_interval() -> u32 {
        REPORT_INTERVAL
    }

    pub const ACTION_TIMEOUT: u32 = 10;
    pub fn action_timeout() -> u32 {
        ACTION_TIMEOUT
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

    pub const CHECK_TIMEOUT: u32 = 5;
    pub fn check_timeout() -> u32 {
        CHECK_TIMEOUT
    }
}

impl TryFrom<&str> for Config {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        toml::from_str(text).map_err(|x| Error(x.to_string()))
    }
}

impl TryFrom<&std::path::Path> for Config {
    type Error = Error;

    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(|x| Error(x.to_string()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|x| Error(x.to_string()))?;
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
        assert!(config.report.disable);
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
            disable = true
            name = "report-event"
            action = "report-action"

            [[actions]]
            disable = true
            name = "test-action"
            type = "Webhook"
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

        assert_eq!(config.report.events.len(), 1);
        let event = config.report.events.first().unwrap();
        assert!(event.disable);
        assert_eq!(event.name, "report-event");
        assert_eq!(event.action, "report-action");

        assert_eq!(config.actions.len(), 1);
        let action = config.actions.first().unwrap();
        assert!(action.disable);
        assert_eq!(action.name, "test-action");
        assert_eq!(
            action.type_,
            ActionType::Webhook(ActionWebhook {
                url: String::from("http://example.com/webhook"),
                method: HttpMethod::GET,
                headers: HashMap::from([(
                    String::from("Content-Type"),
                    String::from("application/json")
                )]),
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
        assert_eq!(alarm.recover_action, Some(String::from("test-action")));
    }
}
