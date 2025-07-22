use lru::LruCache;
use parking_lot::RwLock;
use std::{collections::HashMap, num::NonZero, sync::LazyLock};

use crate::{
    invariants::{Endpoint, Hostname, Timestamp},
    prometheus::PromMetrics,
};

static MAX_HOURS: LazyLock<NonZero<usize>> =
    LazyLock::new(|| NonZero::new(6).expect("nonzero const"));
static MAX_PATHS: LazyLock<NonZero<usize>> =
    LazyLock::new(|| NonZero::new(10).expect("nonzero const"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Event {
    Ok,
    NotFound,
    InternalError,
    Redirect,
}

impl Event {
    pub fn try_from_status(status: u16) -> Option<Self> {
        match status {
            200 => Some(Self::Ok),
            301 => Some(Self::Redirect),
            404 => Some(Self::NotFound),
            500 => Some(Self::InternalError),
            _ => None,
        }
    }
    pub fn to_status(self) -> u16 {
        match self {
            Self::Ok => 200u16,
            Self::Redirect => 301u16,
            Self::NotFound => 404u16,
            Self::InternalError => 500u16,
        }
    }
}

#[derive(Debug)]
pub struct Analytics {
    events: RwLock<HashMap<Event, usize>>,
    paths: RwLock<LruCache<Endpoint, usize>>,
    hosts: RwLock<HashMap<Hostname, usize>>,
    by_hour: RwLock<LruCache<Timestamp, usize>>,
    bytes_by_hour_per_host: RwLock<HashMap<Hostname, LruCache<Timestamp, u64>>>,
}

impl Default for Analytics {
    fn default() -> Self {
        Self {
            events: RwLock::default(),
            paths: RwLock::new(LruCache::new(*MAX_PATHS)),
            hosts: RwLock::default(),
            by_hour: RwLock::new(LruCache::new(*MAX_HOURS)),
            bytes_by_hour_per_host: RwLock::default(),
        }
    }
}

impl Analytics {
    pub fn record_event(&self, code: u16) {
        if let Some(e) = Event::try_from_status(code) {
            let mut map = self.events.write();
            *map.entry(e).or_default() += 1;
        }
    }
    pub fn record_path(&self, path: &str) {
        let mut map = self.paths.write();
        *map.get_or_insert_mut(path.parse().unwrap(), || 0) += 1;
    }
    pub fn record_host(&self, host: &str) {
        let mut map = self.hosts.write();
        *map.entry(host.to_string().parse().unwrap()).or_default() += 1;
    }
    pub fn record_hour_hit(&self, hour: Timestamp) {
        let mut map = self.by_hour.write();
        *map.get_or_insert_mut(hour, || 0) += 1;
    }
    pub fn record_host_hour_bytes(&self, host: &str, hour: Timestamp, bytes: u64) {
        let mut outer = self.bytes_by_hour_per_host.write();
        let entry = outer
            .entry(host.parse().unwrap())
            .or_insert_with(|| LruCache::new(*MAX_HOURS));
        *entry.get_or_insert_mut(hour, || 0) += bytes;
    }

    pub fn event_frequency(&self) -> HashMap<u16, usize> {
        self.events
            .read()
            .iter()
            .map(|(k, v)| (k.to_status(), *v))
            .collect()
    }
    pub fn top_path_frequency(&self, n: usize) -> Vec<(String, usize)> {
        let map = self.paths.read();
        let mut entries: Vec<_> = map.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        entries.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n);
        entries
    }
    pub fn bytes_per_hour_per_host(&self) -> Vec<(String, Vec<(Timestamp, u64)>)> {
        let map = self.bytes_by_hour_per_host.read();
        map.iter()
            .map(|(host, by_hour)| {
                let mut sorted: Vec<_> = by_hour.iter().map(|(t, b)| (*t, *b)).collect();
                sorted.sort_unstable_by_key(|(ts, _)| *ts);
                (host.to_string(), sorted)
            })
            .collect()
    }
    pub fn export_to_prometheus(&self, metrics: &PromMetrics) {
        for (event, count) in self.event_frequency().iter() {
            metrics
                .event_counts
                .with_label_values(&[&event.to_string()])
                .inc_by(*count as u64);
        }

        let host_hits_map = self.hosts.read();
        let mut host_hits: Vec<_> = host_hits_map.iter().collect();
        host_hits.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(**count));
        for (host, count) in host_hits.into_iter().take(10) {
            metrics
                .host_hits
                .with_label_values(&[host.as_str()])
                .inc_by(*count as u64);
        }

        let top_paths = self.top_path_frequency(5);
        for (path, count) in top_paths {
            metrics
                .path_hits
                .with_label_values(&[&path])
                .inc_by(count as u64);
        }

        let host_data = self.bytes_per_hour_per_host();
        let top_hosts = host_data
            .iter()
            .map(|(host, data)| {
                let sum: u64 = data.iter().map(|(_, b)| *b).sum();
                (host, sum)
            })
            .collect::<Vec<_>>();

        let top_hosts = {
            let mut v = top_hosts;
            v.sort_by_key(|(_, total)| std::cmp::Reverse(*total));
            v.truncate(5); // only top 5 hosts
            v
        };

        for (host, data) in host_data.iter() {
            if top_hosts.iter().any(|(h, _)| *h == host) {
                for (hour, bytes) in data.iter().rev().take(4) {
                    let ts = hour.into_utc().format("%Y%m%d%H").to_string();
                    metrics
                        .bytes_per_hour_per_host
                        .with_label_values(&[host, &ts])
                        .set(*bytes as i64);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asserting::prelude::*;
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn record_event_counts_correctly() {
        let analytics = Analytics::default();
        analytics.record_event(200);
        analytics.record_event(200);
        analytics.record_event(404);

        let freq = analytics.event_frequency();
        assert_eq!(freq.get(&200), Some(&2));
        assert_eq!(freq.get(&404), Some(&1));
    }

    #[test]
    fn record_path_counts() {
        let analytics = Analytics::default();
        analytics.record_path("/foo");
        analytics.record_path("/bar");
        analytics.record_path("/foo");

        let paths = analytics.top_path_frequency(2);
        assert_eq!(paths[0], ("/foo".into(), 2));
        assert_eq!(paths[1], ("/bar".into(), 1));
    }

    #[test]
    fn record_host_hour_bytes_counts() {
        let analytics = Analytics::default();
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        analytics.record_host_hour_bytes("host1", ts.into(), 100);
        analytics.record_host_hour_bytes("host1", ts.into(), 200);

        let result = analytics.bytes_per_hour_per_host();
        assert!(result.contains(&("host1".into(), vec![(ts.into(), 300)])));
    }
    #[test]
    fn record_host_hour_bytes_doesnt_eat_ram() {
        let analytics = Analytics::default();
        let ts = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        for i in 0..1000 {
            let ts = ts + Duration::hours(i);
            analytics.record_host_hour_bytes("host1", ts.into(), 200);
        }

        let result = &analytics.bytes_per_hour_per_host()[0].1;
        assert_that!(result.len()).is_in_range(0..=MAX_HOURS.get());
    }
}
