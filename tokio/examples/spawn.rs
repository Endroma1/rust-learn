use tokio::{join, task::yield_now};

/// Shows that spawning async task that waits allows for other tasks to run.
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async move {
        // yields control to main
        yield_now().await;
        println!("Hello from async task");
    });

    println!("Hello from main");
    if let Err(e) = join!(handle).0 {
        eprintln!("Failed to wait for task: {e}");
    };
}
