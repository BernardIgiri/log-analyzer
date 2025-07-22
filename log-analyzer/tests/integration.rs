use asserting::prelude::*;
use std::time::Duration;
use tokio::{process::Command, time::sleep};

const SERVER_URL: &str = "http://localhost:8080";

#[tokio::test]
async fn metrics_endpoint_returns_200() {
    // Start the binary
    let mut child = Command::new(env!("CARGO_BIN_EXE_log-analyzer"))
        .args(["--folder", "logs"])
        .spawn()
        .expect("failed to start server");

    // Wait until /up responds successfully
    let client = reqwest::Client::new();
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 30;

    loop {
        if attempts >= MAX_ATTEMPTS {
            panic!("Server did not become ready in time");
        }
        match client.get(format!("{SERVER_URL}/up")).send().await {
            Ok(resp) if resp.status().is_success() => break,
            _ => {
                attempts += 1;
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    // Make the actual request
    let res = client
        .get(format!("{SERVER_URL}/metrics"))
        .send()
        .await
        .expect("metrics request failed");

    assert_that!(res.status()).satisfies(|s| s.is_success());
    let body = res.text().await.unwrap();
    assert_that!(body).contains("# HELP");

    // Clean up child process
    child.kill().await.unwrap();
}
