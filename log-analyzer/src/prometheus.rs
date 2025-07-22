use prometheus::{IntCounterVec, IntGaugeVec, Registry, opts, register_int_counter_vec};

pub struct PromMetrics {
    pub event_counts: IntCounterVec,
    pub path_hits: IntCounterVec,
    pub host_hits: IntCounterVec,
    pub bytes_per_hour_per_host: IntGaugeVec,
    pub registry: Registry,
}

impl PromMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let event_counts = register_int_counter_vec!(
            opts!("event_count", "Number of HTTP status code events"),
            &["status"]
        )
        .unwrap();

        let path_hits =
            register_int_counter_vec!(opts!("path_hits", "Hits per path"), &["path"]).unwrap();

        let host_hits =
            register_int_counter_vec!(opts!("host_hits", "Hits per host"), &["host"]).unwrap();

        let bytes_per_hour_per_host = IntGaugeVec::new(
            opts!("host_hour_bytes", "Bytes served per hour per host"),
            &["host", "hour"],
        )
        .unwrap();

        registry
            .register(Box::new(bytes_per_hour_per_host.clone()))
            .unwrap();
        registry.register(Box::new(event_counts.clone())).unwrap();
        registry.register(Box::new(path_hits.clone())).unwrap();
        registry.register(Box::new(host_hits.clone())).unwrap();

        Self {
            registry,
            event_counts,
            path_hits,
            host_hits,
            bytes_per_hour_per_host,
        }
    }
}
