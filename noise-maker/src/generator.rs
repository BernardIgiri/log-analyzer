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

pub fn generate_apache_log<R: Rng + ?Sized>(id: usize, rng: &mut R) -> String {
    let ip = format!("192.168.{}.{}", id % 256, rng.random_range(0..256));
    let timestamp = Local::now().format("%d/%b/%Y:%H:%M:%S %z");
    let method = METHODS.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let path = PATHS.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let status = STATUS.choose_weighted(rng, |(_, w)| *w).unwrap().0;
    let size = rng.random_range(100..2000);

    format!("{ip} - - [{timestamp}] \"{method} {path} HTTP/1.1\" {status} {size}")
}

pub fn generate_json_log<R: Rng + ?Sized>(id: usize, rng: &mut R) -> String {
    let ts = Local::now().to_rfc3339();
    let service = ["auth", "api", "frontend", "db"][id % 4];
    let level = ["INFO", "WARN", "ERROR"][rng.random_range(0..3)];
    let msg = [
        "User logged in",
        "DB query executed",
        "Cache miss",
        "Permission denied",
        "Token refreshed",
    ][rng.random_range(0..5)];

    format!("{{\"ts\":\"{ts}\",\"service\":\"{service}\",\"level\":\"{level}\",\"msg\":\"{msg}\"}}")
}
