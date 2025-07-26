use futures_util::StreamExt;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::nats;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn publishes_and_receives_logs() {
    // Start NATS container using testcontainers-modules
    let container = nats::Nats::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(4222).await.unwrap();
    let nats_url = format!("nats://127.0.0.1:{port}");

    // Connect to NATS
    let client = async_nats::connect(&nats_url)
        .await
        .expect("Failed to connect to NATS");

    // Subscribe to a subject
    let subject = "logs";
    let mut subscription = client.subscribe(subject.to_string()).await.unwrap();

    // Publish a message
    let test_message = "integration-test-log";
    client
        .publish(subject.to_string(), test_message.into())
        .await
        .unwrap();

    // Check if message is received
    if let Some(message) = subscription.next().await {
        let data = String::from_utf8(message.payload.to_vec()).unwrap();
        assert_eq!(data, test_message);
    } else {
        panic!("Did not receive message from NATS");
    }
}
