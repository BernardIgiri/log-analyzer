# 🧵 Concurrent Log Metrics Tracker

**A Rust-based observability playground using NATS, Prometheus, Grafana, and Kubernetes.**

---

## ✅ What is this?

This project demonstrates **real-time log metrics processing** in Rust with:

* **`log-analyzer`**: Multi-threaded log metrics processor exposing Prometheus metrics.
* **`noise-maker`**: Synthetic log generator publishing messages to **NATS**.
* **NATS**: High-performance messaging backbone for decoupled log streaming.
* **Prometheus + Grafana**: Complete observability stack for metrics visualization.
* **Kubernetes Deployment**: Fully containerized and orchestrated stack for testing scalability.

---

## 🏗 Architecture

```
+-------------+       +--------+        +-------------+        +-----------+
| noise-maker | --->  |  NATS  |  --->  | log-analyzer | --->  | Prometheus |
+-------------+       +--------+        +-------------+        +-----------+
                                                              |
                                                              v
                                                           Grafana
```

* **noise-maker** simulates log traffic and sends structured messages over **NATS**.
* **log-analyzer** subscribes to NATS subjects, processes log data, and exposes metrics at `/metrics`.
* **Prometheus** scrapes metrics from `log-analyzer` every 5 seconds.
* **Grafana** visualizes system performance using a pre-provisioned dashboard.

---

## ✨ Features

* Multi-threaded log ingestion using `tokio` + Rust channels.
* Metrics served on `http://localhost:8080/metrics` (Prometheus format).
* Kubernetes manifests for **Minikube** or any K8s cluster.
* Podman/Minikube image loading workflow via `just`.
* Preconfigured Grafana dashboard for real-time insights.

---

## 📦 Project Structure

```text
.
├── Cargo.toml                  <-- Workspace root
├── justfile                    <-- Build & deploy tasks
├── docker-compose.yml          <-- Optional local stack
├── grafana/                    <-- Grafana config & dashboards
├── k8s/                        <-- Kubernetes manifests
├── log-analyzer/               <-- Metrics aggregation service in Rust
├── noise-maker/                <-- Log generator publishing to NATS
├── prometheus.yml              <-- Prometheus configuration
├── Dockerfile.log-analyzer     <-- Container image for log-analyzer
├── Dockerfile.noise-maker      <-- Container image for noise-maker
```

---

## 🚀 Quick Start

### **Local Development (Podman)**

```bash
# Run tests, build images, and start the stack
just start

# Stop services
just stop
```

Access:

* **Grafana** → [http://localhost:3000](http://localhost:3000) (admin / admin)
* **Prometheus** → [http://localhost:9090](http://localhost:9090)
* **Log Analyzer Metrics** → [http://localhost:8080/metrics](http://localhost:8080/metrics)

---

### **Kubernetes Deployment (Minikube)**

```bash
# Start Minikube with enough resources
just k8s-start

# Build and load Podman images into Minikube
just k8s-build-load

# Deploy all manifests
just k8s-apply
```

Port-forward services:

```bash
just k8s-grafana     # Grafana on localhost:3000
just k8s-prometheus  # Prometheus on localhost:9090
```

Check pods:

```bash
just k8s-status
```

Tear down:

```bash
just k8s-delete
just k8s-stop
```

---

## 🧪 Testing

Run all Rust tests:

```bash
just test
```

---

## 📊 Example Metrics

Prometheus metrics exposed by `log-analyzer`:

```
log_messages_total{source="noise-maker"} 12345
log_parse_errors_total{reason="invalid_format"} 12
log_throughput_per_second 320
```

---

## 📉 Observability Stack

| Tool       | URL                                            | Notes                             |
| ---------- | ---------------------------------------------- | --------------------------------- |
| Prometheus | [http://localhost:9090](http://localhost:9090) | Scrapes metrics every 5 seconds   |
| Grafana    | [http://localhost:3000](http://localhost:3000) | Dashboard preloaded (admin/admin) |
| NATS       | Internal                                       | Handles log streaming             |

---

## 🔮 Future Explorations

* [ ] Profiling log-analyzer with `pprof` and `flamegraph`
* [ ] Optimize backpressure handling in NATS subscriber
* [ ] CPU & memory instrumentation
* [ ] Horizontal Pod Autoscaling in K8s
