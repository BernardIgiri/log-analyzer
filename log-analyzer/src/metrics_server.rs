use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderValue, Response, header},
    response::IntoResponse,
    routing::get,
};
use prometheus::TextEncoder;
use tokio::{net::TcpListener, task::JoinHandle};

use crate::{analytics::Analytics, prometheus::PromMetrics};

#[derive(Clone)]
struct Metrics(Arc<Analytics>, Arc<PromMetrics>);

pub fn start(analytics: Arc<Analytics>) -> JoinHandle<()> {
    tokio::spawn(async {
        let pro_metrics = Arc::new(PromMetrics::new());
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, router(Metrics(analytics, pro_metrics)))
            .await
            .unwrap();
    })
}
fn router(metrics: Metrics) -> Router {
    Router::new()
        .route("/up", get(up))
        .route("/metrics", get(handler))
        .with_state(metrics)
}
async fn handler(State(Metrics(analytics, pro_metrics)): State<Metrics>) -> Response<Body> {
    analytics.export_to_prometheus(&pro_metrics);
    let metric_families = pro_metrics.registry.gather();
    let mut buffer = String::new();
    let encoder = TextEncoder::new();
    encoder.encode_utf8(&metric_families, &mut buffer).unwrap();
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; version=0.0.4"),
        )],
        buffer,
    )
        .into_response()
}
async fn up() -> Response<Body> {
    ().into_response()
}
