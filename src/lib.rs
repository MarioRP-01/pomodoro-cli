use std::io::{Stdout, Write};
use std::time::Duration;

use async_std::task;
use crossterm::{Command, cursor, event, ExecutableCommand, QueueableCommand};
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::style::Print;
use futures::FutureExt;

use command::TermAction;

use crate::prelude::*;

mod command;
mod pomodoro;
mod error;
pub mod prelude;

async fn clock_tick_loop(
    command_tx: std::sync::mpsc::Sender<TermAction>,
    clock_stop_rx: async_std::channel::Receiver<()>,
    clock_resume_rx: std::sync::mpsc::Receiver<()>,
) {
    loop {
        loop {
            let sleep_future = task::sleep(Duration::from_secs(1)).fuse();
            let stop_future = clock_stop_rx.recv().fuse();

            futures::pin_mut!(sleep_future, stop_future);

            futures::select! {
            _ = sleep_future => command_tx.send(TermAction::ClockTick).unwrap(),
            _ = stop_future => break,
            }
        }
        clock_resume_rx.recv().unwrap();
    }
}

async fn handle_input(tx: std::sync::mpsc::Sender<TermAction>) {
    loop {
        match event::read() {
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            })) => {
                tx.send(TermAction::KeyboardInput(c)).unwrap();
            }
            _ => {}
        }
    }
}

fn init(mut stdout: &Stdout) -> Result<()> {
    stdout.execute(cursor::Hide)?;

    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;

    Ok(())
}

pub fn run() -> Result<()> {
    let mut stdout = std::io::stdout();

    init(&stdout)?;

    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (stop_clock_tx, stop_clock_rx) = async_std::channel::bounded(1);
    let (resume_clock_tx, resume_clock_rx) = std::sync::mpsc::channel();

    let mut pomodoro = pomodoro::Pomodoro::new(stop_clock_tx, resume_clock_tx);

    task::spawn(clock_tick_loop(
        command_tx.clone(),
        stop_clock_rx,
        resume_clock_rx,
    ));
    task::spawn(handle_input(command_tx));

    loop {
        stdout
            .queue(cursor::MoveTo(2, 0))?
            .queue(Print(&pomodoro.clock))?
            .queue(cursor::MoveTo(0, 2))?
            .queue(Print("\u{2192} (s) stop"))?
            .queue(cursor::MoveTo(0, 3))?
            .queue(Print("\u{2192} (c) continue"))?
            .queue(cursor::MoveTo(0, 4))?
            .queue(Print("\u{2192} (r) reset"))?
            .queue(cursor::MoveTo(0, 5))?
            .queue(Print("\u{2192} (q) quit"))?
            .flush()?;
        match command_rx.recv().unwrap() {
            TermAction::KeyboardInput(c) => match c {
                's' => pomodoro.stop(),
                'c' => pomodoro.resume(),
                'r' => pomodoro.reset(),
                'q' => std::process::exit(0),
                _ => {}
            },
            TermAction::ClockTick => pomodoro.tick(),
        }
    }
}
