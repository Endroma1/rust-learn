/// A small program that executes a cpu bound operation in parallell with rayon and consumes it
/// asynchronously with tokio, streaming each result.
use std::{
    thread,
    time::{Duration, Instant},
};

use futures::future::join_all;
use indicatif::ProgressStyle;
use rand::random_range;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time,
};
use tracing::Level;

type DynError = Box<dyn std::error::Error>;

const TASKS: u32 = 1000;

const SYNC_BLOCK_TIME: Duration = Duration::from_millis(1000);
const ASYNC_BLOCK_TIME: Duration = Duration::from_millis(1000);

/// Spawns the rayon task. It must spawn in rayon::spawn to not block and then use the par_iter to
/// do the calculation in parallell.
fn spawn_task(tx: Sender<u32>) {
    rayon::spawn(move || {
        (1..TASKS).into_par_iter().for_each(move |_| {
            let rand_int1 = random_range(1..1000);
            let rand_int2 = random_range(1..1000);
            let res = sum_blocking(rand_int1, rand_int2);

            tracing::info!("Calculated {} + {} = {}", rand_int1, rand_int2, res);

            if let Err(e) = tx.blocking_send(res) {
                tracing::error!("{e}");
            }
        });
    });
}

/// Consumes result asynchronously and blocks until all is consumed.
async fn blocking_consumer(mut rx: Receiver<u32>) {
    let mut handles = Vec::new();

    let style =
        ProgressStyle::with_template("[{elapsed_precise}] | [{eta}] [{wide_bar}] [{percent}]%")
            .unwrap();
    let bar = indicatif::ProgressBar::new(TASKS as u64).with_style(style);
    bar.tick();

    // WARNING: Could potentially spawn infinite amount of workers.
    while let Some(res) = rx.recv().await {
        let bar = bar.clone();

        handles.push(tokio::spawn(async move {
            send_to_db(res).await;
            bar.inc(1);
        }));
    }
    join_all(handles).await;
}

#[tokio::main]
async fn main() -> Result<(), DynError> {
    init_logger()?;

    let now = Instant::now();

    // Does not matter if std or tokio mpsc is used in this example as it is awaited synchronously.
    // See: blocking_consumer()
    let (tx, rx) = tokio::sync::mpsc::channel(10);

    spawn_task(tx);
    blocking_consumer(rx).await;

    let elapsed = now.elapsed();
    println!("Time taken: {}s", elapsed.as_secs());
    println!(
        "Time expected if sync: {}s",
        ((SYNC_BLOCK_TIME + ASYNC_BLOCK_TIME) * TASKS).as_secs()
    );
    Ok(())
}

/// Simulating a blocking operation that returns a value.
fn sum_blocking(input1: u32, input2: u32) -> u32 {
    thread::sleep(SYNC_BLOCK_TIME);
    input1 + input2
}

/// Simulating sending to db
async fn send_to_db(res: u32) {
    time::sleep(ASYNC_BLOCK_TIME).await;
    tracing::info!("Sent {} to value", res);
}

fn init_logger() -> Result<(), DynError> {
    let sub = tracing_subscriber::fmt()
        .with_max_level(Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(sub)?;
    Ok(())
}
