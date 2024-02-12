use crate::{duration_iso8601, Error, PlaceholderMap, Result};

#[cfg_attr(test, mockall::automock)]
pub trait StateHandler: Send + Sync + Sized {
    fn add_placeholders(&self, placeholders: &mut PlaceholderMap);

    fn error(&mut self) -> bool;
    fn bad(&mut self) -> (bool, bool);
    fn good(&mut self) -> (bool, bool);
}

pub struct StateMachine {
    cycles: u32,
    repeat_cycles: u32,
    recover_cycles: u32,
    error_repeat_cycles: u32,
    state: State,
    log_id: String,
}

#[derive(Clone)]
enum State {
    Good(GoodState),
    Bad(BadState),
    Error(ErrorState),
}

impl Default for State {
    fn default() -> Self {
        Self::Good(GoodState::default())
    }
}

impl State {
    fn name(&self) -> &str {
        match self {
            State::Good(_) => "good",
            State::Bad(_) => "bad",
            State::Error(_) => "error",
        }
    }
}

#[derive(Clone)]
struct GoodState {
    timestamp: std::time::SystemTime,
    instant: std::time::Instant,
    last_state_duration: Option<std::time::Duration>,
    bad_cycles: u32,
}

impl Default for GoodState {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            instant: std::time::Instant::now(),
            last_state_duration: None,
            bad_cycles: 0,
        }
    }
}

#[derive(Clone)]
struct BadState {
    timestamp: std::time::SystemTime,
    instant: std::time::Instant,
    last_state_duration: std::time::Duration,
    cycles: u32,
    good_cycles: u32,
}

#[derive(Clone)]
struct ErrorState {
    timestamp: std::time::SystemTime,
    last_state_duration: std::time::Duration,
    shadowed_state: Box<State>,
    cycles: u32,
}

impl StateMachine {
    pub fn new(
        cycles: u32,
        repeat_cycles: u32,
        recover_cycles: u32,
        error_repeat_cycles: u32,
        log_id: String,
    ) -> Result<Self> {
        if cycles == 0 {
            Err(Error(String::from("'cycles' cannot be 0.")))
        } else if recover_cycles == 0 {
            Err(Error(String::from("'recover_cycles' cannot be 0.")))
        } else {
            Ok(Self {
                cycles,
                repeat_cycles,
                recover_cycles,
                error_repeat_cycles,
                state: State::default(),
                log_id,
            })
        }
    }
}

impl StateHandler for StateMachine {
    fn add_placeholders(&self, placeholders: &mut PlaceholderMap) {
        match &self.state {
            State::Bad(bad) => {
                placeholders.insert(String::from("alarm_state"), String::from("Bad"));
                placeholders.insert(
                    String::from("alarm_timestamp"),
                    crate::datetime_iso8601(bad.timestamp),
                );
                placeholders.insert(
                    String::from("alarm_last_duration"),
                    bad.last_state_duration.as_secs().to_string(),
                );
                placeholders.insert(
                    String::from("alarm_last_duration_iso"),
                    duration_iso8601(bad.last_state_duration),
                );
            }

            State::Good(good) => {
                placeholders.insert(String::from("alarm_state"), String::from("Good"));
                placeholders.insert(
                    String::from("alarm_timestamp"),
                    crate::datetime_iso8601(good.timestamp),
                );
                if let Some(last_state_duration) = good.last_state_duration {
                    placeholders.insert(
                        String::from("alarm_last_duration"),
                        last_state_duration.as_secs().to_string(),
                    );
                    placeholders.insert(
                        String::from("alarm_last_duration_iso"),
                        duration_iso8601(last_state_duration),
                    );
                }
            }

            State::Error(error) => {
                placeholders.insert(String::from("alarm_state"), String::from("Error"));
                placeholders.insert(
                    String::from("alarm_timestamp"),
                    crate::datetime_iso8601(error.timestamp),
                );
                placeholders.insert(
                    String::from("alarm_last_duration"),
                    error.last_state_duration.as_secs().to_string(),
                );
                placeholders.insert(
                    String::from("alarm_last_duration_iso"),
                    duration_iso8601(error.last_state_duration),
                );
            }
        }
    }

    fn error(&mut self) -> bool {
        let mut trigger = false;
        self.state = match &self.state {
            State::Good(good) => {
                trigger = true;
                log::warn!("{} changing from good to error state.", self.log_id);
                State::Error(ErrorState {
                    timestamp: std::time::SystemTime::now(),
                    last_state_duration: good.instant.elapsed(),
                    shadowed_state: Box::new(self.state.clone()),
                    cycles: 1,
                })
            }

            State::Bad(bad) => {
                trigger = true;
                log::warn!("{} changing from bad to error state.", self.log_id);
                State::Error(ErrorState {
                    timestamp: std::time::SystemTime::now(),
                    last_state_duration: bad.instant.elapsed(),
                    shadowed_state: Box::new(self.state.clone()),
                    cycles: 1,
                })
            }

            State::Error(error) => {
                let cycles = if error.cycles == self.error_repeat_cycles {
                    trigger = true;
                    1
                } else {
                    error.cycles + 1
                };
                State::Error(ErrorState {
                    cycles,
                    ..error.clone()
                })
            }
        };
        trigger
    }

    fn bad(&mut self) -> (bool, bool) {
        let mut trigger = false;
        let mut trigger_error_recover = false;
        self.state = match &self.state {
            State::Good(good) => {
                if good.bad_cycles + 1 == self.cycles {
                    trigger = true;
                    log::warn!("{} changing from good to bad state.", self.log_id);
                    State::Bad(BadState {
                        timestamp: std::time::SystemTime::now(),
                        instant: std::time::Instant::now(),
                        last_state_duration: good.instant.elapsed(),
                        cycles: 1,
                        good_cycles: 0,
                    })
                } else {
                    State::Good(GoodState {
                        bad_cycles: good.bad_cycles + 1,
                        ..good.clone()
                    })
                }
            }

            State::Bad(bad) => {
                let cycles = if bad.cycles == self.repeat_cycles {
                    trigger = true;
                    1
                } else {
                    bad.cycles + 1
                };
                State::Bad(BadState {
                    cycles,
                    good_cycles: 0,
                    ..bad.clone()
                })
            }

            State::Error(error) => {
                self.state = *error.shadowed_state.clone();
                let (shadowed_trigger, _) = self.bad();
                trigger = shadowed_trigger;
                trigger_error_recover = true;
                log::warn!(
                    "{} changing from error to {} state.",
                    self.log_id,
                    self.state.name(),
                );
                self.state.clone()
            }
        };
        (trigger, trigger_error_recover)
    }

    fn good(&mut self) -> (bool, bool) {
        let mut trigger = false;
        let mut trigger_error_recover = false;
        self.state = match &self.state {
            State::Good(good) => State::Good(good.clone()),

            State::Bad(bad) => {
                if bad.good_cycles + 1 == self.recover_cycles {
                    trigger = true;
                    log::info!("{} changing from bad to good state.", self.log_id);
                    State::Good(GoodState {
                        timestamp: std::time::SystemTime::now(),
                        instant: std::time::Instant::now(),
                        last_state_duration: Some(bad.instant.elapsed()),
                        bad_cycles: 0,
                    })
                } else {
                    State::Bad(BadState {
                        cycles: bad.cycles + 1,
                        good_cycles: bad.good_cycles + 1,
                        ..bad.clone()
                    })
                }
            }

            State::Error(error) => {
                self.state = *error.shadowed_state.clone();
                let (shadowed_trigger, _) = self.good();
                trigger = shadowed_trigger;
                trigger_error_recover = true;
                log::info!(
                    "{} changing from error to {} state.",
                    self.log_id,
                    self.state.name(),
                );
                self.state.clone()
            }
        };
        (trigger, trigger_error_recover)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_validation() {
        assert!(matches!(
            StateMachine::new(0, 0, 1, 0, String::from("")),
            Err(Error(_))
        ));
        assert!(matches!(
            StateMachine::new(1, 0, 0, 0, String::from("")),
            Err(Error(_))
        ));
    }

    #[test]
    fn test_trigger_action() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0, String::from("")).unwrap();
        assert_eq!((true, false), state_machine.bad());
    }

    #[test]
    fn test_trigger_action_repeat() {
        let mut state_machine = StateMachine::new(1, 7, 1, 0, String::from("")).unwrap();
        assert_eq!((true, false), state_machine.bad());
        for _ in 0..6 {
            assert_eq!((false, false), state_machine.bad());
        }
        assert_eq!((true, false), state_machine.bad());
    }

    #[test]
    fn test_trigger_recover_action() {
        let mut state_machine = StateMachine::new(1, 0, 5, 0, String::from("")).unwrap();
        assert_eq!((true, false), state_machine.bad());
        for _ in 0..4 {
            assert_eq!((false, false), state_machine.good());
        }
        assert_eq!((true, false), state_machine.good());
    }

    #[test]
    fn test_trigger_error_action() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0, String::from("")).unwrap();
        assert!(state_machine.error());
    }

    #[test]
    fn test_trigger_error_action_repeat() {
        let mut state_machine = StateMachine::new(1, 0, 1, 7, String::from("")).unwrap();
        assert!(state_machine.error());
        for _ in 0..6 {
            assert!(!state_machine.error());
        }
        assert!(state_machine.error());
    }

    #[test]
    fn test_trigger_error_recover_action() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0, String::from("")).unwrap();
        state_machine.error();
        assert_eq!((false, true), state_machine.good());
    }

    #[test]
    fn test_add_placeholders_good() {
        let state_machine = StateMachine::new(1, 0, 1, 0, String::from("")).unwrap();
        let mut placeholders = PlaceholderMap::new();
        state_machine.add_placeholders(&mut placeholders);
        use std::str::FromStr;
        chrono::DateTime::<chrono::Utc>::from_str(placeholders.get("alarm_timestamp").unwrap())
            .unwrap();
        assert_eq!(placeholders.get("alarm_state").unwrap(), "Good");
        assert_eq!(placeholders.len(), 2);
    }

    #[test]
    fn test_add_placeholders_bad() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0, String::from("")).unwrap();
        let mut placeholders = PlaceholderMap::new();
        state_machine.bad();
        state_machine.add_placeholders(&mut placeholders);
        use std::str::FromStr;
        chrono::DateTime::<chrono::Utc>::from_str(placeholders.get("alarm_timestamp").unwrap())
            .unwrap();
        assert_eq!(placeholders.get("alarm_state").unwrap(), "Bad");
        assert!(placeholders.contains_key("alarm_last_duration"));
        assert!(placeholders.contains_key("alarm_last_duration_iso"));
        assert_eq!(placeholders.len(), 4);
    }

    #[test]
    fn test_add_placeholders_error_without_bad() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0, String::from("")).unwrap();
        let mut placeholders = PlaceholderMap::new();
        state_machine.error();
        state_machine.add_placeholders(&mut placeholders);
        use std::str::FromStr;
        chrono::DateTime::<chrono::Utc>::from_str(placeholders.get("alarm_timestamp").unwrap())
            .unwrap();
        assert_eq!(placeholders.get("alarm_state").unwrap(), "Error");
        assert!(placeholders.contains_key("alarm_last_duration"));
        assert!(placeholders.contains_key("alarm_last_duration_iso"));
        assert_eq!(placeholders.len(), 4);
    }

    #[test]
    fn test_trigger_error_shadowed_good() {
        let mut state_machine = StateMachine::new(2, 0, 1, 0, String::from("")).unwrap();
        assert!(matches!(state_machine.state, State::Good(_)));
        state_machine.error();
        assert!(matches!(state_machine.state, State::Error(_)));
        state_machine.bad();
        assert!(matches!(state_machine.state, State::Good(_)));
    }

    #[test]
    fn test_trigger_error_shadowed_bad() {
        let mut state_machine = StateMachine::new(1, 0, 2, 0, String::from("")).unwrap();
        state_machine.bad();
        assert!(matches!(state_machine.state, State::Bad(_)));
        state_machine.error();
        assert!(matches!(state_machine.state, State::Error(_)));
        state_machine.good();
        assert!(matches!(state_machine.state, State::Bad(_)));
    }
}
