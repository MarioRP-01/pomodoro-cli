use std::fmt;
use std::io::Write;
use std::time::Duration;

use async_std::task;
use crossterm::{Command, ExecutableCommand, QueueableCommand};

#[derive(Debug)]
struct Time {
    hours: u8,
    minutes: u8,
    seconds: u8,
}

impl Time {
    fn new() -> Time {
        Time {
            seconds: 0,
            minutes: 0,
            hours: 0,
        }
    }

    fn increment_second(&mut self) {
        self.seconds = (self.seconds + 1) % 60;
        if self.seconds != 0 {
            return;
        }

        self.minutes = (self.minutes + 1) % 60;
        if self.minutes != 0 {
            return;
        }

        self.hours += 1;
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}",
            self.hours, self.minutes, self.seconds
        )
    }
}

async fn clock_loop(tx: std::sync::mpsc::Sender<()>) {
    loop {
        task::sleep(Duration::from_secs(1)).await;
        tx.send(()).unwrap();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut time = Time::new();

    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;

    task::spawn(clock_loop(tx));

    loop {
        stdout
            .queue(crossterm::cursor::MoveTo(0, 0))?
            .queue(crossterm::style::Print(&time))?
            .flush()?;
        rx.recv().unwrap();
        time.increment_second();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn increments_when_new_clock() {
        let mut clock: Time = Time::new();

        assert_eq!(clock.seconds, 0);
        assert_eq!(clock.minutes, 0);
        assert_eq!(clock.hours, 0);

        clock.increment_second();
        assert_eq!(clock.seconds, 1);
        assert_eq!(clock.minutes, 0);
        assert_eq!(clock.hours, 0);
    }

    #[test]
    fn increments_when_to_minute() {
        let mut clock: Time = Time::new();
        clock.seconds = 59;

        clock.increment_second();
        assert_eq!(clock.seconds, 0);
        assert_eq!(clock.minutes, 1);
        assert_eq!(clock.hours, 0);
    }

    #[test]
    fn increments_when_to_hour() {
        let mut clock = Time::new();
        clock.seconds = 59;
        clock.minutes = 59;

        clock.increment_second();
        assert_eq!(clock.seconds, 0);
        assert_eq!(clock.minutes, 0);
        assert_eq!(clock.hours, 1);
    }
}
