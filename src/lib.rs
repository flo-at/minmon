mod action;
mod alarm;
mod check;
pub mod config;

pub type Result<T> = std::result::Result<T, Error>;
type PlaceholderMap = std::collections::HashMap<String, String>;
type ActionMap = std::collections::HashMap<String, std::sync::Arc<dyn action::Action>>;

#[derive(Debug)]
pub struct Error(pub String); // TODO
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn init_actions(config: &config::Config) -> Result<ActionMap> {
    log::info!("Initializing {} actions(s)..", config.actions.len());
    let mut res = ActionMap::new();
    for action_config in config.actions.iter() {
        if action_config.disable {
            log::info!(
                "Action {}::'{}' is disabled.",
                action_config.type_,
                action_config.name
            );
            continue;
        }
        // TODO an init_checks angleichen (match ins action module verschieben)
        res.insert(
            action_config.name.clone(),
            match &action_config.type_ {
                config::ActionType::WebHook(_) => {
                    std::sync::Arc::new(action::WebHook::try_from(action_config)?)
                }
                config::ActionType::Log(_) => {
                    std::sync::Arc::new(action::Log::try_from(action_config)?)
                }
            },
        );
        log::info!(
            "Action {}::'{}' initialized.",
            action_config.type_,
            action_config.name
        );
    }
    Ok(res)
}

fn init_checks(config: &config::Config, actions: &ActionMap) -> Result<Vec<Box<dyn check::Check>>> {
    log::info!("Initializing {} check(s)..", config.checks.len());
    let mut res: Vec<Box<dyn check::Check>> = Vec::new();
    for check_config in config.checks.iter() {
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
            "Check {} will be triggered every {} seconds.",
            check.name(),
            check.interval().as_secs()
        );
        res.push(check);
    }
    Ok(res)
}

pub fn from_config(config: &config::Config) -> Result<Vec<Box<dyn check::Check>>> {
    let actions = init_actions(config)?;
    init_checks(config, &actions)
}
