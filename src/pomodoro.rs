use async_std::channel::TrySendError;
use crossterm::{cursor, QueueableCommand};
use crossterm::style::Print;
use std::{fmt, io::Stdout};
use std::sync::Arc;
use std::time::Duration;

use crate::action::ResumeAction;
use crate::command::TermAction;
use crate::view::View;
use crate::{action::{Action, StopAction}, Error, Result};


fn is_valid_time(hours: u64, minutes: u64, seconds: u64) -> bool {
    return hours > 23 || minutes > 59 || seconds > 59
}

#[derive(Debug)]
pub(crate) struct Clock {
    duration: Duration,
}

impl Clock {
    pub(crate) fn build(hours: u64, minutes: u64, seconds: u64) -> Clock {
        if is_valid_time(hours, minutes, seconds) {
            panic!("Invalid time")
        }
        Clock {
            duration: Duration::from_secs(seconds + minutes * 60 + hours * 3600),
        }
    }

    pub(crate) fn reset(&mut self, hours: u64, minutes: u64, seconds: u64) -> Result<()> {
        if is_valid_time(hours, minutes, seconds) {
            return Err(Error::Generic("Invalid time".to_string()))
        }

        self.duration = Duration::from_secs(seconds + minutes * 60 + hours * 3600);
        Ok(())
    }

    pub(crate) fn decrement_second(&mut self) -> Result<()> {
        match self.duration.checked_sub(Duration::new(1, 0)) {
            Some(new_duration) => {
                self.duration = new_duration;
                Ok(())
            }
            None => Err(Error::Generic("Invalid substraction in clock".to_string())),
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

impl View for Clock {
    fn display(&self, stdout: &mut Stdout) -> Result<()> {

        stdout
            .queue(Print(&self))?;

        Ok(())
    }
}

pub struct Pomodoro {

    clock: Clock,
    actions: Arc<[Arc<dyn Action>]>,
    stop_action: Arc<StopAction>,
}

impl Pomodoro {
    pub fn new(
        stop_clock_tx: async_std::channel::Sender<()>,
        resume_clock_tx: std::sync::mpsc::Sender<()>,
    ) -> Pomodoro {

        let clock =  Clock::build(0, 1, 0);
        let stop_action = Arc::new(StopAction::new(stop_clock_tx));
        let resume_action = Arc::new(ResumeAction::new(resume_clock_tx));
        let reset_action = Arc::new(ResetAction::new(clock));

        Pomodoro {
            clock: Clock::build(0, 1, 0),
            actions: Arc::new([stop_action.clone(), resume_action]),
            stop_action,
        }
    }

    pub fn reset(&mut self) {
        self.clock = Clock::build(0, 1, 0);
    }

    pub fn tick(&mut self) -> Result<()> {
        match self.clock.decrement_second() {
            Ok(_) => Ok(()),
            Err(_) => self.stop_action.execute(),
        }
    }
    
    pub(crate) fn execute(&mut self, term_action: crate::command::TermAction) -> Result<()> {
        match term_action {
            TermAction::KeyboardInput(c) => self.actions.iter().find(|a| a.get_shortcut() == c).ok_or(Error::Generic("Action not found".to_string()))?.execute(),
            TermAction::ClockTick => self.tick(),
        }
    }
}

impl View for Pomodoro {
    fn display(&self, stdout: &mut Stdout) -> Result<()>{


        stdout
            .queue(cursor::MoveTo(2, 0))?;

        self.clock.display(stdout)?;

        stdout
            .queue(cursor::MoveTo(0, 0))?;

        for action in self.actions.iter() {
            stdout
                .queue(cursor::MoveDown(1))?;

            action.display(stdout)?;
        }

        Ok(())
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

        assert!(time_second.decrement_second().is_ok());
        assert!(time_minute.decrement_second().is_ok());
        assert!(time_hour.decrement_second().is_ok());

        assert_eq!(time_second.duration.as_secs(), 0);
        assert_eq!(time_minute.duration.as_secs(), 59);
        assert_eq!(time_hour.duration.as_secs(), 3599);
    }

    #[test]
    fn decrease_zero_invalid() {
        let mut time_zero: Clock = Clock::build(0, 0, 0);
        assert!(time_zero.decrement_second().is_err());
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
