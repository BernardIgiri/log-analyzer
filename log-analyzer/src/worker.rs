use std::sync::LazyLock;

use crate::models::LogEntry;
use chrono::{DateTime, Utc};
use regex::Regex;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Duration, sleep};

#[derive(Debug)]
pub enum Metric {
    Event(u16),
    Path(String),
    Host(String),
    Hit(DateTime<Utc>),
    HostBytes {
        host: String,
        timestamp: DateTime<Utc>,
        bytes: u64,
    },
}

const FLUSH_INTERVAL: Duration = Duration::from_secs(3);
static BUFFER_SIZE: usize = 500;

static LOG_RX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(?P<host>\S+) \S+ \S+ \[(?P<timestamp>[^\]]+)\] "(?P<method>\S+)\s(?P<path>\S+)[^"]*" (?P<status>\d{3}) (?P<bytes>\d+|-)"#
    ).unwrap()
});

pub async fn worker_loop(tx: Sender<Vec<Metric>>, mut rx: Receiver<String>) {
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    loop {
        tokio::select! {
            maybe_line = rx.recv() => {
                match maybe_line {
                    Some(line) => {
                        if let Some(LogEntry { status, host, timestamp, path, bytes }) = parse_log_line(&line) {
                            buffer.push(Metric::Event(status));
                            buffer.push(Metric::Path(path));
                            buffer.push(Metric::Host(host.clone()));
                            buffer.push(Metric::Hit(timestamp));
                            buffer.push(Metric::HostBytes {
                                host,
                                timestamp,
                                bytes,
                            });
                            if buffer.len() >= BUFFER_SIZE {
                                tx.send(buffer.split_off(0)).await.ok();
                            }
                        }
                    }
                    None => {
                        if !buffer.is_empty() {
                            tx.send(buffer).await.ok();
                        }
                        break;
                    }
                }
            }
            _ = sleep(FLUSH_INTERVAL) => {
                if !buffer.is_empty() {
                    tx.send(buffer.split_off(0)).await.ok();
                }
            }
        }
    }
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let caps = LOG_RX.captures(line)?;

    let host = caps.name("host")?.as_str().to_string();
    let timestamp_str = caps.name("timestamp")?.as_str();
    let path = caps.name("path")?.as_str().to_string();
    let status = caps.name("status")?.as_str().parse().ok()?;
    let bytes_str = caps.name("bytes")?.as_str();
    let bytes = if bytes_str == "-" {
        0
    } else {
        bytes_str.parse().ok()?
    };

    let timestamp = DateTime::parse_from_str(timestamp_str, "%d/%b/%Y:%H:%M:%S %z")
        .ok()
        .map(|dt| dt.with_timezone(&Utc))?;

    Some(LogEntry {
        host,
        timestamp,
        path,
        status,
        bytes,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use asserting::{expectations::IsEqualTo, prelude::*};
    use chrono::prelude::*;

    #[test]
    fn parse_log_line_valid() {
        let line =
            r#"202.32.92.47 - - [01/Jun/1995:00:00:59 -0600] "GET /~scottp/publish.html" 200 271"#;
        assert_that!(parse_log_line(line))
            .is_some()
            .mapping(|o| o.unwrap())
            .expecting(IsEqualTo {
                expected: LogEntry {
                    host: "202.32.92.47".into(),
                    timestamp: FixedOffset::west_opt(6 * 3600)
                        .unwrap()
                        .with_ymd_and_hms(1995, 6, 1, 0, 0, 59)
                        .unwrap()
                        .with_timezone(&Utc),
                    path: "/~scottp/publish.html".into(),
                    status: 200,
                    bytes: 271,
                },
            });
    }
}
