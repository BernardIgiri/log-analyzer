use std::{path::Path, time::Duration};
use tokio::{process::Command, time::sleep};

const SERVER_URL: &str = "http://localhost:8080";

#[tokio::test]
async fn metrics_server_starts() {
    // Start the binary
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("logs");
    let mut child = Command::new(env!("CARGO_BIN_EXE_log-analyzer"))
        .args(["--folder", path.to_string_lossy().to_string().as_str()])
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
    // Clean up child process
    child.kill().await.unwrap();
    child.wait().await.unwrap();
}
