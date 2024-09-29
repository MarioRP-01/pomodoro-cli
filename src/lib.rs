use std::cell::RefCell;
use std::io::{Stdout, Write};
use std::rc::Rc;
use std::time::Duration;

use async_std::channel::TryRecvError;
use async_std::task;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, event, ExecutableCommand, QueueableCommand};
use futures::FutureExt;

use command::TermAction;
use view::View;

use crate::prelude::*;

mod command;
mod error;
mod pomodoro;
mod view;
mod action;
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
        
        match clock_resume_rx.recv() {
            Ok(_) => (),
            Err(_) => break, // Exit if clock_resume_rx is closed
        }
    
        match clock_stop_rx.try_recv() {
            Ok(_) => (),
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Closed) => break, // Exit if clock_stop_rx is closed
        }
    }
}

async fn handle_input(tx: std::sync::mpsc::Sender<TermAction>) {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        })) = event::read()
        {
            match tx.send(TermAction::KeyboardInput(c)) {
                Ok(_) => (),
                Err(_) => break,
            }
        }
    }
}

struct Application {
    stdout: Rc<RefCell<Stdout>>
}

impl Application {
    fn build(stdout: Rc<RefCell<Stdout>>) -> Result<Self> {
        enable_raw_mode().unwrap();
        {
            let stdout_borrowed = &mut stdout.borrow_mut();

            stdout_borrowed.execute(cursor::Hide)?;
            stdout_borrowed.execute(crossterm::terminal::Clear(
                crossterm::terminal::ClearType::All,
            ))?;
        }
        Ok( 
            Application {
                stdout
            }
        )
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        
        if let Ok(mut stdout) = self.stdout.try_borrow_mut() {
            let _ = stdout
                .queue(cursor::MoveTo(0, 0))
                .and_then(|stdout| stdout.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All)))
                .and_then(|stdout| stdout.flush());
        }

        if let Err(e) = disable_raw_mode() {
            eprintln!("Failed to disable raw mode: {}", e);
        }
    }
}

pub fn run() -> Result<()> {

    // let out = Rc::new(RefCell::new(std::io::stdout()));

    // init(&out)?;
    let out = Rc::new(RefCell::new(std::io::stdout()));

    let _application = Application::build(out.clone())?;

    // init(&mut out.borrow_mut())?;

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
        pomodoro.display(&mut out.borrow_mut())?;
        out.borrow_mut().flush()?;

        let _ = pomodoro.execute(command_rx.recv().unwrap());
    };
}
