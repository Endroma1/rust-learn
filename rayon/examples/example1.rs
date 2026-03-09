/// A small program that executes a cpu bound operation in parallell with rayon and consumes it
/// asynchronously with tokio
use std::{thread, time::Duration};

use futures::future::join_all;
use rand::random_range;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::time;
use tracing::Level;

type DynError = Box<dyn std::error::Error>;

const TASKS: i32 = 100;

#[tokio::main]
async fn main() -> Result<(), DynError> {
    init_logger()?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
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

    let mut handles = Vec::new();
    while let Some(res) = rx.recv().await {
        handles.push(tokio::spawn(async move { send_to_db(res).await }));
    }
    join_all(handles).await;
    Ok(())
}

/// Simulating a blocking operation that returns a value.
fn sum_blocking(input1: i32, input2: i32) -> i32 {
    thread::sleep(Duration::from_millis(1000));
    input1 + input2
}

/// Simulating sending to db
async fn send_to_db(res: i32) {
    time::sleep(Duration::from_millis(1000)).await;
    tracing::info!("Sent {} to value", res);
}

fn init_logger() -> Result<(), DynError> {
    let sub = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(sub)?;
    Ok(())
}
