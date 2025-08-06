use clap::{Parser, ValueEnum};
use derive_getters::Getters;

#[derive(Parser, Debug, Getters)]
#[command(name = "log-gen-cli")]
#[command(about = "Generate fake log streams for testing", long_about = None)]
pub struct CliArgs {
    #[arg(long, default_value = "nats://nats:4222")]
    nats_url: String,

    #[arg(long, default_value = "logs")]
    subject: String,

    #[arg(long, default_value_t = 10)]
    rate: u64,

    #[arg(long, default_value_t = 10000)]
    batch_size: usize,

    #[arg(long, value_enum, default_value_t = LogFormat::Apache)]
    format: LogFormat,

    #[arg(long, default_value = "server.log")]
    log_file: String,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LogFormat {
    Apache,
    Json,
}
