use crate::PlaceholderMap;

#[cfg_attr(test, mockall::automock)]
pub trait StateHandler: Send + Sync + Sized {
    fn add_placeholders(&self, placeholders: &mut PlaceholderMap);

    fn error(&mut self) -> bool;
    fn bad(&mut self) -> bool;
    fn good(&mut self) -> bool;
}

pub struct StateMachine {
    cycles: u32,
    repeat_cycles: u32,
    recover_cycles: u32,
    error_repeat_cycles: u32,
    state: State,
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

#[derive(Clone)]
struct GoodState {
    timestamp: std::time::SystemTime,
    last_alarm: Option<BadState>,
    bad_cycles: u32,
}

impl Default for GoodState {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            last_alarm: None,
            bad_cycles: 0,
        }
    }
}

#[derive(Clone)]
struct BadState {
    timestamp: std::time::SystemTime,
    uuid: String,
    cycles: u32,
    good_cycles: u32,
}

#[derive(Clone)]
struct ErrorState {
    timestamp: std::time::SystemTime,
    uuid: String,
    shadowed_state: Box<State>,
    cycles: u32,
}

impl StateMachine {
    pub fn new(
        cycles: u32,
        repeat_cycles: u32,
        recover_cycles: u32,
        error_repeat_cycles: u32,
    ) -> Self {
        // TODO ensure cycles != 0 and recover_cycles != 0
        Self {
            cycles,
            repeat_cycles,
            recover_cycles,
            error_repeat_cycles,
            state: State::default(),
        }
    }
}

impl StateHandler for StateMachine {
    fn add_placeholders(&self, placeholders: &mut PlaceholderMap) {
        match &self.state {
            State::Bad(bad) => {
                placeholders.insert(
                    String::from("alarm_timestamp"),
                    crate::iso8601(bad.timestamp),
                );
                placeholders.insert(String::from("alarm_uuid"), bad.uuid.clone());
            }

            State::Good(good) => {
                if let Some(last_alarm) = &good.last_alarm {
                    placeholders.insert(String::from("alarm_uuid"), last_alarm.uuid.clone());
                    placeholders.insert(
                        String::from("alarm_timestamp"),
                        crate::iso8601(last_alarm.timestamp),
                    );
                } else {
                    panic!();
                }
            }

            State::Error(error) => {
                // TODO add info about shadowed_state (add bad uuid and timestamp, ..)
                placeholders.insert(String::from("error_uuid"), error.uuid.clone());
                placeholders.insert(
                    String::from("error_timestamp"),
                    crate::iso8601(error.timestamp),
                );
            }
        }
    }

    fn error(&mut self) -> bool {
        let mut trigger = false;
        self.state = match &self.state {
            State::Good(_) => {
                trigger = true;
                State::Error(ErrorState {
                    timestamp: std::time::SystemTime::now(),
                    uuid: uuid::Uuid::new_v4().to_string(),
                    shadowed_state: Box::new(self.state.clone()),
                    cycles: 1,
                })
            }

            State::Bad(_) => {
                trigger = true;
                State::Error(ErrorState {
                    timestamp: std::time::SystemTime::now(),
                    uuid: uuid::Uuid::new_v4().to_string(),
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
                    timestamp: error.timestamp,
                    uuid: error.uuid.clone(),
                    shadowed_state: error.shadowed_state.clone(),
                    cycles,
                })
            }
        };
        trigger
    }

    fn bad(&mut self) -> bool {
        let mut trigger = false;
        self.state = match &self.state {
            State::Good(good) => {
                if good.bad_cycles + 1 == self.cycles {
                    trigger = true;
                    State::Bad(BadState {
                        timestamp: std::time::SystemTime::now(),
                        uuid: uuid::Uuid::new_v4().to_string(),
                        cycles: 1,
                        good_cycles: 0,
                    })
                } else {
                    State::Good(GoodState {
                        timestamp: good.timestamp,
                        last_alarm: None,
                        bad_cycles: good.bad_cycles + 1,
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
                    timestamp: bad.timestamp,
                    uuid: bad.uuid.clone(),
                    cycles,
                    good_cycles: 0,
                })
            }

            State::Error(error) => {
                self.state = *error.shadowed_state.clone();
                let shadowed_trigger = self.bad();
                trigger = shadowed_trigger;
                self.state.clone()
            }
        };
        trigger
    }

    fn good(&mut self) -> bool {
        let mut trigger = false;
        self.state = match &self.state {
            State::Good(good) => State::Good(good.clone()),

            State::Bad(bad) => {
                if bad.good_cycles + 1 == self.recover_cycles {
                    trigger = true;
                    State::Good(GoodState {
                        timestamp: std::time::SystemTime::now(),
                        last_alarm: Some(bad.clone()),
                        bad_cycles: 0,
                    })
                } else {
                    State::Bad(BadState {
                        timestamp: bad.timestamp,
                        uuid: bad.uuid.clone(),
                        cycles: bad.cycles + 1,
                        good_cycles: bad.good_cycles + 1,
                    })
                }
            }

            State::Error(error) => {
                self.state = *error.shadowed_state.clone();
                let shadowed_trigger = self.good();
                trigger = shadowed_trigger;
                self.state.clone()
            }
        };
        trigger
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_trigger_action() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0);
        assert!(state_machine.bad());
    }

    #[test]
    fn test_trigger_action_repeat() {
        let mut state_machine = StateMachine::new(1, 7, 1, 0);
        assert!(state_machine.bad());
        for _ in 0..6 {
            assert!(!state_machine.bad());
        }
        assert!(state_machine.bad());
    }

    #[test]
    fn test_trigger_recover_action() {
        let mut state_machine = StateMachine::new(1, 0, 5, 0);
        assert!(state_machine.bad());
        for _ in 0..4 {
            assert!(!state_machine.good());
        }
        assert!(state_machine.good());
    }

    #[test]
    fn test_trigger_error_action() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0);
        assert!(state_machine.error());
    }

    #[test]
    fn test_trigger_error_action_repeat() {
        let mut state_machine = StateMachine::new(1, 0, 1, 7);
        assert!(state_machine.error());
        for _ in 0..6 {
            assert!(!state_machine.error());
        }
        assert!(state_machine.error());
    }

    #[test]
    fn test_add_placeholders_good() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0);
        let mut placeholders = PlaceholderMap::new();
        // starts in good state without "last alarm"
        state_machine.bad();
        state_machine.good();
        state_machine.add_placeholders(&mut placeholders);
        uuid::Uuid::parse_str(placeholders.get("alarm_uuid").unwrap()).unwrap();
        use std::str::FromStr;
        chrono::DateTime::<chrono::Utc>::from_str(placeholders.get("alarm_timestamp").unwrap())
            .unwrap();
        assert_eq!(placeholders.len(), 2);
    }

    #[test]
    fn test_add_placeholders_bad() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0);
        let mut placeholders = PlaceholderMap::new();
        state_machine.bad();
        state_machine.add_placeholders(&mut placeholders);
        uuid::Uuid::parse_str(placeholders.get("alarm_uuid").unwrap()).unwrap();
        use std::str::FromStr;
        chrono::DateTime::<chrono::Utc>::from_str(placeholders.get("alarm_timestamp").unwrap())
            .unwrap();
        assert_eq!(placeholders.len(), 2);
    }

    #[test]
    fn test_add_placeholders_error_without_bad() {
        let mut state_machine = StateMachine::new(1, 0, 1, 0);
        let mut placeholders = PlaceholderMap::new();
        state_machine.error();
        state_machine.add_placeholders(&mut placeholders);
        uuid::Uuid::parse_str(placeholders.get("error_uuid").unwrap()).unwrap();
        use std::str::FromStr;
        chrono::DateTime::<chrono::Utc>::from_str(placeholders.get("error_timestamp").unwrap())
            .unwrap();
        assert_eq!(placeholders.len(), 2);
    }

    #[test]
    fn test_trigger_error_shadowed_good() {
        let mut state_machine = StateMachine::new(2, 0, 1, 0);
        assert!(matches!(state_machine.state, State::Good(_)));
        state_machine.error();
        assert!(matches!(state_machine.state, State::Error(_)));
        state_machine.bad();
        assert!(matches!(state_machine.state, State::Good(_)));
    }

    #[test]
    fn test_trigger_error_shadowed_bad() {
        let mut state_machine = StateMachine::new(1, 0, 2, 0);
        state_machine.bad();
        assert!(matches!(state_machine.state, State::Bad(_)));
        state_machine.error();
        assert!(matches!(state_machine.state, State::Error(_)));
        state_machine.good();
        assert!(matches!(state_machine.state, State::Bad(_)));
    }
}
