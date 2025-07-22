use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "log-gen-cli")]
#[command(about = "Generate fake log streams for testing", long_about = None)]
pub struct CliArgs {
    #[arg(short, long, default_value_t = 5)]
    pub streams: usize,

    #[arg(short, long, default_value_t = 10)]
    pub rate: u64,

    #[arg(short, long, value_enum, default_value_t = LogFormat::Apache)]
    pub format: LogFormat,

    #[arg(long)]
    pub folder: PathBuf,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum LogFormat {
    Apache,
    Json,
}
