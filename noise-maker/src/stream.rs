use crate::args::LogFormat;
use crate::generator::{generate_apache_log, generate_json_log};
use async_nats::Client;
use rand::{SeedableRng, rngs::StdRng};
use tokio::time::{Duration, sleep};
use tryhard::{RetryFutureConfig, retry_fn};

pub async fn run_log_stream(
    rate: u64,
    format: LogFormat,
    nats_url: &str,
    subject: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = RetryFutureConfig::new(10)
        .exponential_backoff(Duration::from_millis(100))
        .max_delay(Duration::from_secs(5));
    let client: Client = retry_fn(|| async {
        println!("Attempting to connect to NATS at {nats_url}");
        async_nats::connect(nats_url).await
    })
    .with_config(config)
    .await?;
    let mut rng = StdRng::from_os_rng();
    let delay = Duration::from_millis(1000 / rate.max(1));
    loop {
        let log = match format {
            LogFormat::Apache => generate_apache_log(&mut rng),
            LogFormat::Json => generate_json_log(&mut rng),
        };
        client.publish(subject.to_string(), log.into()).await?;
        sleep(delay).await;
    }
}
