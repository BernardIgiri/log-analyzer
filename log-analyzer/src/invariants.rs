use std::str::FromStr;

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use derive_more::{AsRef, Debug, Display};

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Hostname(String);

impl Hostname {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Hostname {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash)]
pub struct Endpoint(String);

impl FromStr for Endpoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.chars().take(100).collect()))
    }
}

#[derive(Debug, Display, AsRef, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn into_utc(self) -> DateTime<Utc> {
        self.0
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(value: DateTime<Utc>) -> Self {
        Self(
            Utc.with_ymd_and_hms(value.year(), value.month(), value.day(), value.hour(), 0, 0)
                .unwrap(),
        )
    }
}
