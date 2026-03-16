use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use indicatif::ProgressBar;

type DynError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, DynError>;

/// Program that has a progress bar that is not created by the iterating process but rather reads
/// from a state API. Uses message passing with mpsc to communicate state between frontend and
/// backend.

const ITERATIONS: usize = 100;
const INTERVAL: Duration = Duration::from_millis(100);

fn main() -> Result<()> {
    let process = Process::new(ITERATIONS);

    let handle = process.spawn();

    // ProgressBar is initiablized on first iteration.
    let mut bar = None;

    // Get the len from the process to simulate a situation where it is not obvious.
    while let Ok(s) = handle.state.recv() {
        match s {
            State::Init { total } => {
                let b = bar.get_or_insert_with(|| ProgressBar::new(total as u64));
                b.tick();
            }
            State::InProgress { delta } => {
                let b = bar.clone().expect("Got InProgress message before Init");
                b.tick();
                b.inc(delta as u64);
            }
            State::Stopped => break,
        }
    }

    handle.join_handle.join().unwrap();
    Ok(())
}

pub struct Process {
    iterations: usize,
}
impl Process {
    fn new(iterations: usize) -> Self {
        Process { iterations }
    }
    fn run(&self, sender: Sender<State>) {
        let mut progress = Progress::new(self.iterations);
        if sender
            .send(State::Init {
                total: self.iterations,
            })
            .is_err()
        {
            return;
        };

        for _ in 1..=self.iterations {
            sleep(INTERVAL);
            progress.inc(1);
            if sender.send(State::InProgress { delta: 1 }).is_err() {
                break;
            };
        }

        let _ = sender.send(State::Stopped);
    }
    fn spawn(self) -> ProcessHandle<()> {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || self.run(tx));
        ProcessHandle::new(handle, rx)
    }
}

pub struct ProcessHandle<T> {
    join_handle: JoinHandle<T>,
    state: Receiver<State>,
}
impl<T> ProcessHandle<T> {
    pub fn new(join_handle: JoinHandle<T>, state: Receiver<State>) -> Self {
        Self { join_handle, state }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum State {
    /// The size of the task.
    Init { total: usize },
    /// Progress of the task for the current tick.
    InProgress { delta: usize },
    #[default]
    Stopped,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Progress {
    total: usize,
    processed: usize,
}
impl Progress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            processed: 0,
        }
    }
    pub fn inc(&mut self, delta: usize) {
        self.processed += delta;
    }
    pub fn is_finished(&self) -> bool {
        self.processed >= self.total
    }
}
