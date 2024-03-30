use std::io::Write;
use std::time::Duration;

use async_std::task;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{event, ExecutableCommand, QueueableCommand};
use futures::FutureExt;

use command::PomodoroCommand;


mod command;
mod pomodoro;

async fn clock_tick_loop(
    command_tx: std::sync::mpsc::Sender<PomodoroCommand>,
    clock_stop_rx: async_std::channel::Receiver<()>,
    clock_resume_rx: std::sync::mpsc::Receiver<()>,
) {
    loop {
        loop {
            let sleep_future = task::sleep(Duration::from_secs(1)).fuse();
            let stop_future = clock_stop_rx.recv().fuse();

            futures::pin_mut!(sleep_future, stop_future);

            futures::select! {
            _ = sleep_future => command_tx.send(PomodoroCommand::ClockTick).unwrap(),
            _ = stop_future => break,
            }
        }
        clock_resume_rx.recv().unwrap();
    }
}

async fn handle_input(tx: std::sync::mpsc::Sender<PomodoroCommand>) {
    loop {
        match event::read() {
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            })) => {
                tx.send(PomodoroCommand::KeyboardInput(c)).unwrap();
            }
            _ => {}
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();

    stdout.execute(crossterm::cursor::Hide)?;

    let (command_tx, command_rx) = std::sync::mpsc::channel();
    let (stop_clock_tx, stop_clock_rx) = async_std::channel::bounded(1);
    let (resume_clock_tx, resume_clock_rx) = std::sync::mpsc::channel();

    let mut pomodoro = pomodoro::Pomodoro::new(stop_clock_tx, resume_clock_tx);

    stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ))?;

    task::spawn(clock_tick_loop(
        command_tx.clone(),
        stop_clock_rx,
        resume_clock_rx,
    ));
    task::spawn(handle_input(command_tx));

    loop {
        stdout
            .queue(crossterm::cursor::MoveTo(2, 0))?
            .queue(crossterm::style::Print(&pomodoro.clock))?
            .queue(crossterm::cursor::MoveTo(0, 2))?
            .queue(crossterm::style::Print("\u{2192} (s) stop"))?
            .queue(crossterm::cursor::MoveTo(0, 3))?
            .queue(crossterm::style::Print("\u{2192} (c) continue"))?
            .queue(crossterm::cursor::MoveTo(0, 4))?
            .queue(crossterm::style::Print("\u{2192} (r) reset"))?
            .queue(crossterm::cursor::MoveTo(0, 5))?
            .queue(crossterm::style::Print("\u{2192} (q) quit"))?
            .flush()?;
        match command_rx.recv().unwrap() {
            PomodoroCommand::KeyboardInput(c) => match c {
                's' => pomodoro.stop(),
                'c' => pomodoro.resume(),
                'r' => pomodoro.reset(),
                'q' => std::process::exit(0),
                _ => {}
            },
            PomodoroCommand::ClockTick => pomodoro.tick(),
        }
    }
}
