# 🧵 Concurrent Log Metrics Tracker

A Rust-based learning project exploring concurrency, file IO, metrics aggregation, and observability using a custom log ingestion pipeline and metrics server. This project consists of:

- `log-analyzer`: A multi-threaded log processor with Prometheus metrics output.
- `noise-maker`: A synthetic log generator for testing the system under load.
- Containerized setup with Prometheus and Grafana dashboards.

---

## 🧠 Purpose

This project was built to explore:

- **Asynchronous Rust** using `tokio`
- **Concurrency control** with channels and worker threads
- **Metric aggregation** with `prometheus`
- **Observability tooling** using Prometheus + Grafana

---

## 📁 Project Structure

```text
.
├── docker-compose.yml
├── grafana/                <-- Grafana config
├── log-analyzer/           <-- Log metrics tracking server in Rust Axum
├── noise-maker/            <-- Log generator
├── prometheus.yml          <-- Prometheus config
├── Dockerfile.log-analyzer <-- Dockerfile for log-analizer
├── Dockerfile.noise-maker  <-- Docker file for noise-maker
├── justfile                <-- justifle
├── Cargo.toml              <-- Workspace Cargo.toml
```

---

## 🚀 Usage

### Prerequisites

- Rust ≥ 1.88
  - Rust toolchain target `x86_64-unknown-linux-musl`

- Podman and Podman-Compose
- Just

### Build all workspace binaries

```bash
just build
```

### Run locally with Podman

```bash
just start
```

You can access grafana at [http://localhost:3000](http://localhost:3000) with default user name `admin` and default password`admin`.

### Stop with

```bash
just stop
```

---

## 🔧 Configuration

- **Prometheus**: [`prometheus.yml`](./prometheus.yml)
- **Grafana Dashboards**: [`grafana/dashboards`](./grafana/dashboards)
- **Log Ingestion Format**: defined in `log-analyzer/src/ingest.rs`
- **Metrics Export**: served on `http://localhost:8080/metrics`

---

## 🧪 Testing

```bash
just test
```

---

## 📊 Metrics Examples

- `log_lines_total{file="..."}`
- `log_errors_total{reason="parse_failure"}`
- `http_status_count{status="200"}`
- `lines_per_second{source="noise-maker"}`

---

## 📉 Observability Stack

| Tool       | URL                   | Notes                                                        |
| ---------- | --------------------- | ------------------------------------------------------------ |
| Prometheus | http://localhost:9090 | Scrapes metrics every 5s                                     |
| Grafana    | http://localhost:3000 | Default login: admin / admin                                 |
| Log Source | noise-maker           | Simulates Apache server traffic through via appended log files |

---

## ✨ Example Dashboard

A custom dashboard (`grafana/dashboards/log_analyzer.json`) is preloaded via provisioning.

- Track logs per second
- Visualize error rates
- View status code counts
- Latency graphs (optional extension)

---

## 🔮 Future Explorations

- [ ] Process to empty log files before they consume too much disk space when left running.
- [ ] Distributed log ingest
  - [ ] kafka messaging?
- [ ] CPU and memory instrumentation
  - [ ] Performance optimizations
- [ ] Kubernetes orchestration
