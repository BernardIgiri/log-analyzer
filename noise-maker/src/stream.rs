use crate::args::LogFormat;
use crate::generator::{generate_apache_log, generate_json_log};
use async_nats::Client;
use rand::{SeedableRng, rngs::StdRng};
use tokio::time::{Duration, sleep};
use tryhard::{RetryFutureConfig, retry_fn};

const MAX_RATE_BEFORE_DISABLING_THROTTLING: u64 = 10_000;

pub async fn run_log_stream(
    rate: u64,
    batch_size: usize,
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
    let delay = if rate < MAX_RATE_BEFORE_DISABLING_THROTTLING {
        Some(Duration::from_secs_f64(
            1f64 / (rate as f64 / batch_size as f64).ceil(),
        ))
    } else {
        None
    };

    loop {
        let mut buffer = String::with_capacity(batch_size * 128);
        for _ in 0..batch_size {
            let log_line = match format {
                LogFormat::Apache => generate_apache_log(&mut rng),
                LogFormat::Json => generate_json_log(&mut rng),
            };
            buffer.push_str(&log_line);
            buffer.push('\n');
        }
        client.publish(subject.to_string(), buffer.into()).await?;

        if let Some(d) = delay {
            sleep(d).await;
        }
    }
}
