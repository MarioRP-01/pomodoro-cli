use std::io::{Stdout, Write};
use std::time::Duration;

use async_std::task;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, event, ExecutableCommand, QueueableCommand};
use futures::FutureExt;

use command::TermAction;

use crate::prelude::*;

mod command;
mod error;
mod pomodoro;
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
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        })) = event::read()
        {
            tx.send(TermAction::KeyboardInput(c)).unwrap();
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
    enable_raw_mode().unwrap();

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

    let result = 'outer: loop {
        stdout
            .queue(cursor::MoveTo(2, 0))?
            .queue(Print(&pomodoro.clock))?
            .queue(cursor::MoveTo(0, 2))?
            .queue(pomodoro.stop_command())?
            .queue(cursor::MoveTo(0, 3))?
            .queue(pomodoro.resume_command())?
            .queue(cursor::MoveTo(0, 4))?
            .queue(pomodoro.reset_command())?
            .queue(cursor::MoveTo(0, 5))?
            .queue(pomodoro.quit_command())?
            .queue(cursor::MoveTo(0, 6))?
            .flush()?;
        match command_rx.recv().unwrap() {
            TermAction::KeyboardInput(c) => match c {
                's' => pomodoro.stop(),
                'c' => pomodoro.resume(),
                'r' => pomodoro.reset(),
                'q' => break 'outer Ok(()),
                _ => {}
            },
            TermAction::ClockTick => pomodoro.tick(),
        }
    };

    stdout
        .queue(cursor::MoveTo(0, 0))?
        .queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?
        .flush()?;

    disable_raw_mode().unwrap();

    result
}
