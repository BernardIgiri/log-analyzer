mod analytics;
mod ingest;
mod invariants;
mod metrics_server;
mod models;
mod prometheus;
mod worker;

use analytics::Analytics;
use clap::Parser;
use ingest::consume_nats;
use std::sync::Arc;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::{JoinError, JoinHandle},
    try_join,
};
use worker::{Metric, worker_loop};

const INGEST_BUFFER_SIZE: usize = 50;
const AGGREGATOR_BUFFER_SIZE: usize = 5;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "nats://127.0.0.1:4222")]
    nats_url: String,

    #[arg(long, default_value = "logs")]
    subject: String,

    #[arg(long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    let args = Args::parse();
    let analytics = Arc::new(Analytics::default());
    let metrics_handle = metrics_server::start(analytics.clone(), args.port);

    let (ingest_tx, ingest_rx) = mpsc::channel(INGEST_BUFFER_SIZE);
    let (aggregator_tx, aggregator_rx) = mpsc::channel::<Vec<Metric>>(AGGREGATOR_BUFFER_SIZE);

    let nats_handle = spawn_nats_ingest(args.nats_url, args.subject, ingest_tx);
    let worker_handle = spawn_workers(ingest_rx, aggregator_tx);
    let aggregator_handle = spawn_aggregator(aggregator_rx, &analytics);

    try_join!(
        nats_handle,
        worker_handle,
        aggregator_handle,
        metrics_handle
    )?;

    Ok(())
}

fn spawn_nats_ingest(nats_url: String, subject: String, tx: Sender<String>) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = consume_nats(nats_url, subject, tx).await {
            eprintln!("NATS ingest error: {e}");
        }
    })
}

fn spawn_workers(rx: Receiver<String>, tx: Sender<Vec<Metric>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        worker_loop(tx, rx).await;
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
