mod pomodoro;
mod command;

use std::io::Write;
use std::time::Duration;
use async_std::task;
use crossterm::{event, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode, KeyEvent};
use command::PomodoroCommand;
use pomodoro::Time;


async fn clock_loop(tx: std::sync::mpsc::Sender<PomodoroCommand>) {
    loop {
        task::sleep(Duration::from_secs(1)).await;
        tx.send(PomodoroCommand::ClockIncrement).unwrap();
    }
}

async fn handle_input(tx: std::sync::mpsc::Sender<PomodoroCommand>) {
    loop {
        if let Ok(event) = event::read() {
            match event {
                Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                    tx.send(PomodoroCommand::KeyboardInput(c)).unwrap();
                }
                _ => {}
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut time = Time::new();

    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;

    task::spawn(clock_loop(tx.clone()));
    task::spawn(handle_input(tx));

    loop {
        stdout
            .queue(crossterm::cursor::MoveTo(2, 0))?
            .queue(crossterm::style::Print(&time))?
            .queue(crossterm::cursor::MoveTo(0, 2))?
            .queue(crossterm::style::Print("\u{2192} (s) stop"))?
            .queue(crossterm::cursor::MoveTo(0, 3))?
            .queue(crossterm::style::Print("\u{2192} (r) reset"))?
            .queue(crossterm::cursor::MoveTo(0, 4))?
            .queue(crossterm::style::Print("\u{2192} (q) quit"))?
            .flush()?;
        match rx.recv().unwrap() {
            PomodoroCommand::KeyboardInput(c) => {
                match c {
                    's' => {},
                    'r' => time = Time::new(),
                    'q' => std::process::exit(0),
                    _ => {}
                }
            }
            PomodoroCommand::ClockIncrement => time.increment_second(),
        }
    }
}
