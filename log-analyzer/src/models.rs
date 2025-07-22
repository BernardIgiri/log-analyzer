use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub host: String,
    pub timestamp: DateTime<Utc>,
    pub path: String,
    pub status: u16,
    pub bytes: u64,
}
