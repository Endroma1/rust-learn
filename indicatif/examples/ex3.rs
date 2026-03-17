use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use indicatif::{MultiProgress, ProgressBar};

type DynError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, DynError>;

/// Program that has a progress bar that is not created by the iterating process but rather reads
/// from a state API. This allows for easier control over multiprogress as the progressbar can be
/// used without the function having to implement it.
///
/// Uses message passing with crossbeam to communicate state between frontend and
/// backend.

const ITERATIONS: usize = 100;
const INTERVAL: Duration = Duration::from_millis(100);
const PROGRESSBARS: usize = 4;

fn main() -> Result<()> {
    let mut handles = Vec::new();
    let multi = MultiProgress::new();
    for _ in 1..=PROGRESSBARS {
        let process = Process::new(ITERATIONS);
        let proc_handle = process.spawn();

        let bar = ProgressBar::new(0);
        multi.add(bar.clone());

        let view_handle = progressbar_frontend(&bar, proc_handle.state);

        handles.push(view_handle);
        handles.push(proc_handle.join_handle);
    }
    handles.into_iter().for_each(|h| h.join().unwrap());
    Ok(())
}

fn progressbar_frontend(bar: &ProgressBar, rec: Receiver<State>) -> JoinHandle<()> {
    let bar = bar.clone();
    thread::spawn(move || {
        // Get the len from the process to simulate a situation where it is not obvious.
        while let Ok(s) = rec.recv() {
            match s {
                State::Init { total } => {
                    bar.set_length(total as u64);
                    bar.tick();
                }
                State::InProgress { delta } => {
                    bar.tick();
                    bar.inc(delta as u64);
                }
                State::Stopped => break,
            }
        }
    })
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
