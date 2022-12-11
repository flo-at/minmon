#![deny(warnings)]

mod action;
mod alarm;
mod check;
pub mod config;
mod report;

pub type Result<T> = std::result::Result<T, Error>;
type PlaceholderMap = std::collections::HashMap<String, String>;
type ActionMap = std::collections::HashMap<String, std::sync::Arc<dyn action::Action>>;

pub fn user_agent() -> String {
    format!("MinMon/v{}", env!("CARGO_PKG_VERSION"))
}

#[derive(Debug)]
pub struct Error(pub String);
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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

fn iso8601(system_time: std::time::SystemTime) -> String {
    let date_time: chrono::DateTime<chrono::Utc> = system_time.into();
    date_time.format("%FT%TZ").to_string()
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
        log::info!(
            "Action {}::'{}' initialized.",
            action_config.type_,
            action_config.name
        );
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
            log::info!(
                "Check {}::'{}' is disabled.",
                check_config.type_,
                check_config.name
            );
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

pub fn from_config(
    config: &config::Config,
) -> Result<(Option<report::Report>, Vec<Box<dyn check::Check>>)> {
    let actions = init_actions(config)?;
    let report = init_report(config, &actions)?;
    let checks = init_checks(config, &actions)?;
    Ok((report, checks))
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
    fn test_iso8601() {
        let system_time = std::time::SystemTime::UNIX_EPOCH;
        assert_eq!(iso8601(system_time), "1970-01-01T00:00:00Z");
    }
}
