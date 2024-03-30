use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub(crate) struct Clock {
    duration: Duration,
}

impl Clock {
    pub(crate) fn build(hours: u64, minutes: u64, seconds: u64) -> Clock {
        if hours > 23 || minutes > 59 || seconds > 59 {
            panic!("Invalid time")
        }
        Clock {
            duration: Duration::from_secs(seconds + minutes * 60 + hours * 3600),
        }
    }

    pub(crate) fn decrement_second(&mut self) -> Result<(), ()> {
        match self.duration.checked_sub(Duration::new(1, 0)) {
            Some(new_duration) => {
                self.duration = new_duration;
                Ok(())
            }
            None => Err(()),
        }
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_seconds = self.duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        write!(f, "{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

pub struct Pomodoro {
    pub clock: Clock,
    stop_clock_tx: async_std::channel::Sender<()>,
    resume_clock_tx: std::sync::mpsc::Sender<()>
}

impl Pomodoro {
    pub fn new(
        stop_clock_tx: async_std::channel::Sender<()>,
        resume_clock_tx: std::sync::mpsc::Sender<()>
    ) -> Pomodoro {
        Pomodoro {
            clock: Clock::build(0, 1, 0),
            stop_clock_tx,
            resume_clock_tx
        }
    }

    pub fn stop(&self) {
        self.stop_clock_tx.try_send(()).unwrap();
    }

    pub fn resume(&self) {
        self.resume_clock_tx.send(()).unwrap();
    }

    pub fn reset(&mut self) {
        self.clock = Clock::build(0, 1, 0);
    }

    pub fn tick(&mut self) {
        match self.clock.decrement_second() {
            Ok(_) => {}
            Err(_) => self.stop(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decreases_non_zero() {
        let mut time_second: Clock = Clock::build(0, 0, 1);
        let mut time_minute: Clock = Clock::build(0, 1, 0);
        let mut time_hour: Clock = Clock::build(1, 0, 0);

        assert_eq!(time_second.decrement_second(), Ok(()));
        assert_eq!(time_minute.decrement_second(), Ok(()));
        assert_eq!(time_hour.decrement_second(), Ok(()));

        assert_eq!(time_second.duration.as_secs(), 0);
        assert_eq!(time_minute.duration.as_secs(), 59);
        assert_eq!(time_hour.duration.as_secs(), 3599);
    }

    #[test]
    fn decrease_zero_invalid() {
        let mut time_zero: Clock = Clock::build(0, 0, 0);
        assert_eq!(time_zero.decrement_second(), Err(()));
        assert_eq!(time_zero.duration.as_secs(), 0);
    }

    #[test]
    fn displays() {
        let time_zero: Clock = Clock::build(0, 0, 0);
        let time_minute: Clock = Clock::build(0, 59, 59);
        let time_hour: Clock = Clock::build(23, 59, 59);

        assert_eq!(time_zero.to_string(), "00:00:00");
        assert_eq!(time_minute.to_string(), "00:59:59");
        assert_eq!(time_hour.to_string(), "23:59:59");
    }
}
