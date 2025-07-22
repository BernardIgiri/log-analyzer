use std::path::PathBuf;

use crate::args::LogFormat;
use crate::generator::{generate_apache_log, generate_json_log};
use rand::{SeedableRng, rngs::StdRng};
use tokio::{
    fs::OpenOptions,
    io::AsyncWriteExt,
    time::{Duration, interval},
};

pub async fn run_log_stream(root: PathBuf, id: usize, rate: u64, format: LogFormat) {
    let mut rng = StdRng::from_os_rng();
    let mut ticker = interval(Duration::from_millis(1000 / rate.max(1)));

    let mut filepath = root.clone();
    filepath.push(format!("log-{id}.log"));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)
        .await
        .expect("Failed to open log file");

    loop {
        ticker.tick().await;
        let log = match format {
            LogFormat::Apache => generate_apache_log(id, &mut rng),
            LogFormat::Json => generate_json_log(id, &mut rng),
        };

        if let Err(e) = file.write_all(log.as_bytes()).await {
            eprintln!("[Stream {id}] Write error: {e}");
        }
        if let Err(e) = file.write_all(b"\n").await {
            eprintln!("[Stream {id}] Newline write error: {e}");
        }
    }
}
