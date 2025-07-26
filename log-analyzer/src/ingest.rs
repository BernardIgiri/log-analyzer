use async_nats::Client;
use futures_util::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tryhard::{RetryFutureConfig, retry_fn};

pub async fn consume_nats(
    nats_url: String,
    subject: String,
    tx: Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = RetryFutureConfig::new(10)
        .exponential_backoff(Duration::from_millis(100))
        .max_delay(Duration::from_secs(5));
    let client: Client = retry_fn(|| async {
        println!("Attempting to connect to NATS at {nats_url}");
        async_nats::connect(&nats_url).await
    })
    .with_config(config)
    .await?;
    let mut sub = client.subscribe(subject.clone()).await?;
    while let Some(msg) = sub.next().await {
        let payload = String::from_utf8_lossy(&msg.payload).to_string();
        if tx.send(payload).await.is_err() {
            break;
        }
    }
    Ok(())
}
