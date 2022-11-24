use crate::config;

pub trait DataSink {
    type Item;
    fn put_data(&mut self, id: &str, data: &Self::Item);
}

pub struct Alarm {
    name: String,
    action: String,
    cycles: u32,
    repeat_cycles: u32,
    recover_action: String,
    recover_cycles: u32,
    // --
    bad_cycles: u32,
    good_cycles: u32,
    good: bool,
}

impl Alarm {
    fn bad(&mut self) -> bool {
        self.good_cycles = 0;
        self.bad_cycles += 1;
        if self.bad_cycles >= self.cycles {
            let good_old = self.good;
            self.good = false;
            return good_old
                || (self.repeat_cycles > 0
                    && (self.bad_cycles - self.cycles) % self.repeat_cycles == 0);
        }
        false
    }

    fn good(&mut self) -> bool {
        self.bad_cycles = 0;
        self.good_cycles += 1;
        if self.good_cycles == self.recover_cycles {
            let good_old = self.good;
            self.good = true;
            return !good_old;
        }
        false
    }
}

impl From<&config::Alarm> for Alarm {
    fn from(alarm: &config::Alarm) -> Self {
        Self {
            name: alarm.name.clone(),
            action: alarm.action.clone(),
            cycles: alarm.cycles,
            repeat_cycles: alarm.repeat_cycles,
            recover_action: alarm.recover_action.clone(),
            recover_cycles: alarm.recover_cycles,
            bad_cycles: 0,
            good_cycles: 0,
            good: true,
        }
    }
}

pub struct Level {
    alarm: Alarm,
    level: u8,
}

impl From<&config::Alarm> for Level {
    fn from(alarm: &config::Alarm) -> Self {
        let config::AlarmType::Level(level_config) = &alarm.type_;
        Self {
            alarm: Alarm::from(alarm),
            level: level_config.level,
        }
    }
}

impl DataSink for Level {
    type Item = u8;
    fn put_data(&mut self, id: &str, data: &Self::Item) {
        log::debug!(
            "Got level {} for alarm '{}' at id '{}'",
            data,
            self.alarm.name,
            id
        );
        if *data >= self.level {
            if self.alarm.bad() {
                log::debug!("BAD action triggered!");
            }
        } else {
            if self.alarm.good() {
                log::debug!("GOOD action triggered!");
            }
        }
    }
}
