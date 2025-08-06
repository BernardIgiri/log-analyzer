use crate::models::LogEntry;
use chrono::{DateTime, Utc};
use time::{OffsetDateTime, macros::format_description};
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
const BUFFER_SIZE: usize = 1_000_000;

// Timestamp format for log entries: [01/Jun/1995:00:00:59 -0600]
static TS_FORMAT: &[time::format_description::FormatItem<'static>] = format_description!(
    "[day]/[month repr:short case_sensitive:false]/[year]:[hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]"
);

pub async fn worker_loop(tx: Sender<Vec<Metric>>, mut rx: Receiver<String>) {
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    loop {
        tokio::select! {
            maybe_chunk = rx.recv() => {
                match maybe_chunk {
                    Some(chunk) => {
                        for line in chunk.split('\n').filter(|l| !l.is_empty()) {
                            if let Some(LogEntry { status, host, timestamp, path, bytes }) = parse_log_line(line) {
                                buffer.push(Metric::Event(status));
                                buffer.push(Metric::Path(path));
                                buffer.push(Metric::Host(host.clone()));
                                buffer.push(Metric::Hit(timestamp));
                                buffer.push(Metric::HostBytes { host, timestamp, bytes });
                                if buffer.len() >= BUFFER_SIZE {
                                    tx.send(buffer.split_off(0)).await.ok();
                                }
                            }
                        }
                    }
                    None => break,
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
    let mut parts = line.split_whitespace();
    let host = parts.next()?.to_string();
    parts.next()?; // skip '-'
    parts.next()?; // skip '-'
    // Timestamp spans two tokens
    let ts_part1 = parts.next()?;
    let ts_part2 = parts.next()?;
    let ts_full = format!("{ts_part1} {ts_part2}");
    let ts = ts_full.strip_prefix('[')?.strip_suffix(']')?;
    // Skip method (it has a leading quote)
    let _method = parts.next()?;
    let mut path = parts.next()?.to_string();
    if path.ends_with('"') {
        path.pop();
    }
    let status: u16 = parts.next()?.parse().ok()?;
    let bytes_str = parts.next()?;
    let bytes = if bytes_str == "-" {
        0
    } else {
        bytes_str.parse().ok()?
    };
    let offset_dt = OffsetDateTime::parse(ts, &TS_FORMAT).ok()?;
    let timestamp = chrono::DateTime::<Utc>::from_timestamp(offset_dt.unix_timestamp(), 0)?;
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
