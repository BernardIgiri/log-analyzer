use std::time::Duration;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::nats;
use tokio::{process::Command, time::sleep};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn log_analyzer_starts_and_consumes() {
    // Start NATS container
    let nats_container = nats::Nats::default()
        .start()
        .await
        .expect("Failed to start NATS container");
    let nats_port = nats_container
        .get_host_port_ipv4(4222)
        .await
        .expect("Failed to get NATS port");
    let nats_url = format!("nats://127.0.0.1:{nats_port}");

    // Start log-analyzer
    let metrics_port = portpicker::pick_unused_port().expect("No free ports available");
    let metrics_url = format!("http://localhost:{metrics_port}");
    let mut child = Command::new(env!("CARGO_BIN_EXE_log-analyzer"))
        .args([
            "--nats-url",
            &nats_url,
            "--subject",
            "logs",
            "--port",
            &metrics_port.to_string(),
        ])
        .spawn()
        .expect("Failed to start log-analyzer");

    // Wait until /up responds
    let client = reqwest::Client::new();
    for _ in 0..50 {
        if client
            .get(format!("{metrics_url}/up"))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }

    // Publish a log to NATS
    let nats_client = async_nats::connect(&nats_url)
        .await
        .expect("Failed to connect to NATS");
    let apache_log = r#"127.0.0.1 - - [25/Jul/2025:23:59:59 +0000] "GET /api HTTP/1.1" 200 512"#;
    nats_client
        .publish("logs", apache_log.into())
        .await
        .unwrap();
    nats_client.flush().await.unwrap();

    // Poll /metrics for expected content
    let mut metrics_ok = false;
    for _ in 0..50 {
        let resp = client
            .get(format!("{metrics_url}/metrics"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        if resp.contains("event_count") {
            metrics_ok = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }

    assert!(metrics_ok, "Metrics endpoint never returned event_count");

    let _ = child.kill().await;
    let _ = child.wait().await;
}
