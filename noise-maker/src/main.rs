use std::fs::File;

use args::CliArgs;
use clap::Parser;
use stream::run_log_stream;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt::writer::BoxMakeWriter};

mod args;
mod generator;
mod stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    {
        #[allow(clippy::expect_used)]
        let file = File::create(args.log_file()).expect("Could not open log file");
        let writer = BoxMakeWriter::new(file);
        tracing_subscriber::fmt()
            .with_writer(writer)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
    }
    info!("Starting noise-maker");
    run_log_stream(
        *args.rate(),
        *args.batch_size(),
        *args.format(),
        args.nats_url().as_str(),
        args.subject().as_str(),
    )
    .await?;

    Ok(())
}
