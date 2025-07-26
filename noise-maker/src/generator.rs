use chrono::Local;
use rand::{Rng, seq::IndexedRandom};

const METHODS: [(&str, u8); 4] = [("GET", 6), ("POST", 2), ("PUT", 1), ("DELETE", 1)];
const PATHS: [(&str, u8); 6] = [
    ("/", 10),
    ("/login", 10),
    ("/api", 50),
    ("/admin", 5),
    ("/splash", 20),
    ("gallery", 10),
];
const STATUS: [(u16, u8); 6] = [
    (200, 50),
    (201, 10),
    (400, 10),
    (401, 20),
    (404, 50),
    (500, 5),
];
const SERVICE: [(&str, u8); 4] = [("auth", 1), ("api", 5), ("frontend", 10), ("db", 10)];
const LEVEL: [(&str, u8); 3] = [("INFO", 30), ("WARN", 5), ("ERROR", 1)];
const MESSAGE: [(&str, u8); 5] = [
    ("User logged in", 5),
    ("DB query executed", 50),
    ("Cache miss", 10),
    ("Permission denied", 10),
    ("Token refreshed", 8),
];

pub fn generate_apache_log<R: Rng + ?Sized>(rng: &mut R) -> String {
    let ip = format!(
        "192.168.{}.{}",
        rng.random_range(0..256),
        rng.random_range(0..256)
    );
    let timestamp = Local::now().format("%d/%b/%Y:%H:%M:%S %z");
    let method = METHODS.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let path = PATHS.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let status = STATUS.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let size = rng.random_range(100..2000);

    format!("{ip} - - [{timestamp}] \"{method} {path} HTTP/1.1\" {status} {size}")
}

pub fn generate_json_log<R: Rng + ?Sized>(rng: &mut R) -> String {
    let ts = Local::now().to_rfc3339();
    let service = SERVICE.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let level = LEVEL.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let msg = MESSAGE.choose_weighted(rng, |(_, w)| *w).unwrap().0;

    format!("{{\"ts\":\"{ts}\",\"service\":\"{service}\",\"level\":\"{level}\",\"msg\":\"{msg}\"}}")
}
