#![deny(warnings)]
#![allow(clippy::too_many_arguments)]
#[cfg(not(target_os = "linux"))]
compile_error!("Only Linux is supported");

mod action;
mod alarm;
mod check;
pub mod config;
mod filter;
mod measurement;
mod process;
mod report;
pub mod uptime;
mod window_buffer;

pub type Result<T> = std::result::Result<T, Error>;
type PlaceholderMap = std::collections::HashMap<String, String>;
type ActionMap = std::collections::HashMap<String, std::sync::Arc<dyn action::Action>>;

pub fn user_agent() -> String {
    format!("MinMon/v{}", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug, Clone)]
pub struct Error(pub String);
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn global_placeholders() -> PlaceholderMap {
    let mut res = PlaceholderMap::new();
    let system_uptime = uptime::system();
    let minmon_uptime = uptime::process();
    res.insert(
        String::from("system_uptime"),
        system_uptime.as_secs().to_string(),
    );
    res.insert(
        String::from("system_uptime_iso"),
        duration_iso8601(system_uptime),
    );
    res.insert(
        String::from("minmon_uptime"),
        minmon_uptime.as_secs().to_string(),
    );
    res.insert(
        String::from("minmon_uptime_iso"),
        duration_iso8601(minmon_uptime),
    );
    res
}

fn merge_placeholders(target: &mut PlaceholderMap, source: &PlaceholderMap) {
    for (key, value) in source.iter() {
        target.insert(key.clone(), value.clone());
    }
}

fn fill_placeholders(template: &str, placeholders: &PlaceholderMap) -> String {
    let template = text_placeholder::Template::new(template);
    template.fill_with_hashmap(
        &placeholders
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect(),
    )
}

fn datetime_iso8601(system_time: std::time::SystemTime) -> String {
    let date_time: chrono::DateTime<chrono::Utc> = system_time.into();
    date_time.format("%FT%TZ").to_string()
}

// only up to "days" because the number of days in a month/year are not defined in the standard
fn duration_iso8601(duration: std::time::Duration) -> String {
    const SECONDS_PER_MINUTE: u64 = 60;
    const MINUTES_PER_HOUR: u64 = 60;
    const HOURS_PER_DAY: u64 = 24;
    const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;
    const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * HOURS_PER_DAY;
    let mut remainder = duration.as_secs();
    if remainder == 0 {
        return String::from("PT0S");
    }
    let mut res = String::new();
    let days = remainder / SECONDS_PER_DAY;
    if days > 0 {
        res = format!("P{days}D");
    }
    remainder %= SECONDS_PER_DAY;
    if remainder == 0 {
        return res;
    }
    res.push('T');
    let hours = remainder / SECONDS_PER_HOUR;
    if hours > 0 {
        res = format!("{res}{hours}H");
    }
    remainder %= SECONDS_PER_HOUR;
    if remainder == 0 {
        return res;
    }
    let minutes = remainder / SECONDS_PER_MINUTE;
    if minutes > 0 {
        res = format!("{res}{minutes}M");
    }
    remainder %= SECONDS_PER_MINUTE;
    if remainder > 0 {
        res = format!("{res}{remainder}S");
    }
    res
}

fn init_actions(config: &config::Config) -> Result<ActionMap> {
    log::info!("Initializing {} actions(s)..", config.actions.len());
    let mut res = ActionMap::new();
    for action_config in config.actions.iter() {
        if res.contains_key(&action_config.name) {
            return Err(Error(format!(
                "Found duplicate action name: {}",
                action_config.name
            )));
        }
        let action = action::from_action_config(action_config)?;
        res.insert(action_config.name.clone(), action);
        log::info!("Action '{}' initialized.", action_config.name);
    }
    Ok(res)
}

fn init_report(config: &config::Config, actions: &ActionMap) -> Result<Option<report::Report>> {
    log::info!("Initializing report..");
    let report_config = &config.report;
    if report_config.disable {
        log::info!("Report is disabled.");
        return Ok(None);
    }
    let report = report::from_report_config(report_config, actions)?;
    log::info!(
        "Report will be triggered every {} seconds.",
        report.interval().as_secs()
    );
    Ok(Some(report))
}

fn init_checks(config: &config::Config, actions: &ActionMap) -> Result<Vec<Box<dyn check::Check>>> {
    log::info!("Initializing {} check(s)..", config.checks.len());
    let mut res: Vec<Box<dyn check::Check>> = Vec::new();
    let mut used_names = std::collections::HashSet::new();
    for check_config in config.checks.iter() {
        if !used_names.insert(check_config.name.clone()) {
            return Err(Error(format!(
                "Found duplicate check name: {}",
                check_config.name
            )));
        }
        if check_config.disable {
            log::info!("Check '{}' is disabled.", check_config.name);
            continue;
        }
        let check = check::from_check_config(check_config, actions)?;
        log::info!(
            "Check '{}' will be triggered every {} seconds.",
            check.name(),
            check.interval().as_secs()
        );
        res.push(check);
    }
    Ok(res)
}

type ConfigState = (Option<report::Report>, Vec<Box<dyn check::Check>>);

pub fn from_config(config: &config::Config) -> Result<ConfigState> {
    let actions = init_actions(config)?;
    let report = init_report(config, &actions)?;
    let checks = init_checks(config, &actions)?;
    Ok((report, checks))
}

fn get_number<T>(error_message: &str, line: &str, column: usize) -> Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    {
        line.split_whitespace()
            .nth(column)
            .ok_or_else(|| Error(String::from("Column not found.")))?
            .parse()
            .map_err(|x| Error(format!("{x}")))
    }
    .map_err(|x| Error(format!("{error_message}: {x}")))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_merge_placeholders() {
        let mut target = PlaceholderMap::from([(String::from("A"), String::from("?"))]);
        let source = PlaceholderMap::from([
            (String::from("A"), String::from("B")),
            (String::from("C"), String::from("D")),
        ]);
        merge_placeholders(&mut target, &source);
        assert_eq!(target.get("A").unwrap(), "B");
        assert_eq!(target.get("C").unwrap(), "D");
    }

    #[test]
    fn test_fill_placeholders() {
        let template = "X{{A}}{{missing}}Z";
        let placeholders = PlaceholderMap::from([(String::from("A"), String::from("Y"))]);
        let filled = fill_placeholders(template, &placeholders);
        assert_eq!(filled, "XYZ");
    }

    #[test]
    fn test_datetime_iso8601() {
        let system_time = std::time::SystemTime::UNIX_EPOCH;
        assert_eq!(datetime_iso8601(system_time), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_duration_iso8601() {
        let duration = std::time::Duration::from_secs(123630);
        assert_eq!(duration_iso8601(duration), "P1DT10H20M30S");
        let duration = std::time::Duration::from_secs(37230);
        assert_eq!(duration_iso8601(duration), "T10H20M30S");
        let duration = std::time::Duration::from_secs(0);
        assert_eq!(duration_iso8601(duration), "PT0S");
    }

    #[test]
    fn test_get_number() {
        let line = "0 1 2 3 4 5";
        assert_eq!(get_number::<u32>("error", line, 0).unwrap(), 0);
        assert_eq!(get_number::<u32>("error", line, 5).unwrap(), 5);
        assert!(get_number::<u32>("error", line, 6).is_err());
    }
}
