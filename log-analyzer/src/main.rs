mod analytics;
mod ingest;
mod invariants;
mod metrics_server;
mod models;
mod prometheus;
mod worker;

use analytics::Analytics;
use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::{JoinError, JoinHandle, JoinSet},
    try_join,
};
use worker::{Metric, worker_loop};

const INGEST_BUFFER_SIZE: usize = 50;
const AGGREGATOR_BUFFER_SIZE: usize = 5;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Input folder to read logs from
    #[arg(short, long)]
    folder: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    let args = Args::parse();
    let analytics = Arc::new(Analytics::default());
    let metrics_handle = metrics_server::start(analytics.clone());
    let (ingest_tx, ingest_rx) = mpsc::channel(INGEST_BUFFER_SIZE);
    let (aggregator_tx, aggregator_rx) = mpsc::channel::<Vec<Metric>>(AGGREGATOR_BUFFER_SIZE);
    let file_task_handles = spawn_log_file_tasks(&args.folder, ingest_tx);
    let worker_handle = spawn_workers(ingest_rx, aggregator_tx);
    let aggregator_handle = spawn_aggregator(aggregator_rx, &analytics);
    file_task_handles
        .join_all()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, JoinError>>()?;
    try_join!(worker_handle, aggregator_handle, metrics_handle)?;
    Ok(())
}
fn spawn_log_file_tasks(folder: &Path, tx: Sender<String>) -> JoinSet<Result<(), JoinError>> {
    fs::read_dir(folder)
        .unwrap()
        .flat_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_file() {
                let tx_clone = tx.clone();
                Some(tokio::spawn(async move {
                    if let Err(e) = ingest::tail_file(&path, tx_clone).await {
                        eprintln!("Error tailing {path:?}: {e:?}");
                    }
                }))
            } else {
                None
            }
        })
        .collect()
}
fn spawn_workers(rx: Receiver<String>, tx: Sender<Vec<Metric>>) -> JoinHandle<()> {
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        worker_loop(tx_clone, rx).await;
    })
}
fn spawn_aggregator(mut rx: Receiver<Vec<Metric>>, analytics: &Arc<Analytics>) -> JoinHandle<()> {
    let analytics_clone = analytics.clone();
    tokio::spawn(async move {
        while let Some(batch) = rx.recv().await {
            for metric in batch {
                match metric {
                    Metric::Event(code) => analytics_clone.record_event(code),
                    Metric::Path(path) => analytics_clone.record_path(&path),
                    Metric::Host(host) => analytics_clone.record_host(&host),
                    Metric::Hit(moment) => analytics_clone.record_hour_hit(moment.into()),
                    Metric::HostBytes {
                        host,
                        timestamp,
                        bytes,
                    } => analytics_clone.record_host_hour_bytes(&host, timestamp.into(), bytes),
                }
            }
        }
    })
}
