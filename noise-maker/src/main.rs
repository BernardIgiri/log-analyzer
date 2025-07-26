use args::CliArgs;
use clap::Parser;
use stream::run_log_stream;

mod args;
mod generator;
mod stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();

    run_log_stream(
        *args.rate(),
        *args.format(),
        args.nats_url().as_str(),
        args.subject().as_str(),
    )
    .await?;

    Ok(())
}
