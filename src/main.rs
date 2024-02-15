use std::thread;
use std::time::Duration;

use async_std::task;
use async_std::task::sleep;

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
    task::spawn(clock_loop());
    thread::sleep(Duration::from_secs(5));
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
