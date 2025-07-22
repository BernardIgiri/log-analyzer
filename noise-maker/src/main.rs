mod args;
mod generator;
mod stream;

use args::CliArgs;
use clap::Parser;
use stream::run_log_stream;
use tokio::signal;
use tokio::task;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = CliArgs::parse();
    println!(
        "Starting {} log streams at {} logs/sec per stream",
        args.streams, args.rate
    );

    let mut handles = vec![];
    for stream_id in 0..args.streams {
        let handle = task::spawn(run_log_stream(
            args.folder.clone(),
            stream_id,
            args.rate,
            args.format,
        ));
        handles.push(handle);
    }

    // Wait for CTRL+C
    signal::ctrl_c().await.expect("failed to listen for ctrl_c");
    println!("\nStopping log generation...");
}
