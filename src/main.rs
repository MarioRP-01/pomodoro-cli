use std::io;
use std::io::{StdoutLock, Write};
use std::time::Duration;

use async_std::sync::channel;
use async_std::task;
use async_std::task::sleep;
use termion::{async_stdin, terminal_size};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

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

async fn clock_loop() {
    let mut clock: Time = Time::new();
    loop {
        println!("{:?}", clock);
        sleep(Duration::from_secs(1)).await;
        clock.increment_second()
    }
}

fn main() {
    let stdout = io::stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = async_stdin().keys();

    term_init(&mut stdout);

    task::spawn(clock_loop());

    let ()

    loop {
        let b = stdin.next();
        match b {
            Some(Ok(key)) => {
                match key {
                    termion::event::Key::Char('q') => break,
                    _ => {
                        let (_, height) = terminal_size().unwrap();
                        write!(stdout, "{}{}", termion::cursor::Goto(1, height), "Invalid command. Press 'q' to exit.").unwrap();
                        stdout.flush().unwrap();
                    }
                }
            },
            _ => {}
        }
    }
}

fn term_init(stdout: &mut RawTerminal<StdoutLock>) {
    let (_, height) = terminal_size().unwrap();
    write!(stdout, "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, height)
    ).unwrap();
    stdout.flush().unwrap();
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
