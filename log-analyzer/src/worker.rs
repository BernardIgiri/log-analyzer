use crate::models::LogEntry;
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::{Duration, sleep};
use tracing::debug;

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

pub async fn worker_loop(tx: Sender<Vec<Metric>>, mut rx: Receiver<String>) {
    let mut buffer = Vec::with_capacity(BUFFER_SIZE);
    loop {
        tokio::select! {
            maybe_chunk = rx.recv() => {
                match maybe_chunk {
                    Some(chunk) => {
                        debug!("chunk: {chunk}");
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
    let mut parts = line.split_ascii_whitespace();
    let host = parts.next()?;
    parts.next()?; // skip '-'
    parts.next()?; // skip '-'
    let ts1 = parts.next()?.strip_prefix('[')?;
    let ts2 = parts.next()?.strip_suffix(']')?;
    let ts_start = ts1.as_ptr() as usize - line.as_ptr() as usize;
    let ts_end = ts2.as_ptr() as usize - line.as_ptr() as usize + ts2.len();
    let ts_combined = &line[ts_start..ts_end];
    parts.next()?; // skip method
    let mut path = parts.next()?;
    if let Some(stripped) = path.strip_suffix('"') {
        path = stripped;
    }
    let status: u16 = parts.next()?.parse().ok()?;
    let bytes = match parts.next()? {
        "-" => 0,
        s => s.parse().ok()?,
    };
    let dt = parse_apache_timestamp(ts_combined)?;
    Some(LogEntry {
        host: host.to_owned(),
        timestamp: dt,
        path: path.to_owned(),
        status,
        bytes,
    })
}

fn parse_apache_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    let mut parts = ts.splitn(7, ['/', ':', ' ']);
    let day: u32 = parts.next()?.parse().ok()?;
    let month = match parts.next()? {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year: i32 = parts.next()?.parse().ok()?;
    let hour: u32 = parts.next()?.parse().ok()?;
    let min: u32 = parts.next()?.parse().ok()?;
    let sec: u32 = parts.next()?.parse().ok()?;
    let offset: i32 = parts.next().map(|s| s.parse::<i32>().ok())??;
    let offset_hours = offset / 100;
    let offset_min = offset - (offset_hours * 100);
    Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()
        .map(|t| t - TimeDelta::hours(offset_hours.into()) - TimeDelta::minutes(offset_min.into()))
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
