use std::io::Stdout;

use async_std::channel::SendError;
use async_std::channel::TrySendError;
use crossterm::style::Print;
use crossterm::QueueableCommand;

use crate::view::View;
use crate::Error;
use crate::Result;

pub(crate) trait Action {
    fn get_shortcut(&self) -> char;
    fn get_description(&self) -> &str;
    fn execute(&self) -> Result<()>;
}

impl View for dyn Action {
    fn display(&self, stdout: &mut Stdout) -> Result<()> {
        let text = format!("\u{2192} ({}) {}", self.get_shortcut(), self.get_description());
        stdout.queue(Print(text))?;

        Ok(())
    }
}

pub(crate) struct StopAction {
    stop_clock_tx: async_std::channel::Sender<()>,
}

impl StopAction {
    pub(crate) fn new(stop_clock_tx: async_std::channel::Sender<()>) -> StopAction {
        StopAction {
            stop_clock_tx
        }
    }
}

impl Action for StopAction {
    fn get_shortcut(&self) -> char {
        's'
    }
    
    fn get_description(&self) -> &str {
        "stop"
    }

    fn execute(&self) -> Result<()> {
        self.stop_clock_tx.try_send(()).or_else(|e| match e {
            TrySendError::Full(_) => Ok(()),
            TrySendError::Closed(_) => Err(Error::Generic("stop_close_tx closed".to_string()))
        })
    }
}

pub(crate) struct ResumeAction {
    resume_clock_tx: std::sync::mpsc::Sender<()>
}

impl ResumeAction {
    pub(crate) fn new(resume_clock_tx: std::sync::mpsc::Sender<()>) -> Self {
        Self {
            resume_clock_tx
        }
    }
}

impl Action for ResumeAction {
    fn get_shortcut(&self) -> char {
        'c'
    }

    fn get_description(&self) -> &str {
        "continue"
    }

    fn execute(&self) -> Result<()> {
        self.resume_clock_tx.send(()).or(Err(Error::Generic("stop_close_tx closed".to_string())))
    }
}

pub(crate) fn new() {
    
}

// pub fn reset(&mut self) {
//     self.clock = Clock::build(0, 1, 0);
// }
