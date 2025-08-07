# Run tests
test:
    cargo test --no-default-features -- --include-ignored

# Build Rust binaries for musl target
build-rs:
    cargo build --release --target x86_64-unknown-linux-musl

# Build podman containers
build-pod:
    podman-compose build --no-cache

# Full build: Rust + podman
build: build-rs build-pod

# Start services using Podman Compose
start-podman:
    podman-compose up

# Stop services
stop:
    podman-compose down

# Clean Rust build artifacts
clean-rs:
    cargo clean

# Start everything (tests + build + podman-compose up)
start: test build start-podman

# Restart all services
restart: stop start

# Full clean
clean: stop clean-rs

# Build log-analyzer with profiling
build-pprof:
    cargo build --profile pprof --features pprof -p log-analyzer
    podman-compose build --no-cache

# Run profiling: Usage: just profile 30 100000 1
profile shutdown_after rate batch_size:
    mkdir -p profile logs
    just build-pprof
    BATCH_SIZE={{batch_size}} RATE={{rate}} podman-compose up -d --force-recreate nats noise-maker
    cargo run --profile pprof --features pprof -p log-analyzer -- \
      --nats-url nats://127.0.0.1:4222 \
      --shutdown-after {{shutdown_after}} \
      --log-file logs/log-analyzer-profile.log
    podman-compose down

# View the flamegraph after profiling
view-flamegraph:
    xdg-open profile/flamegraph.svg

# View Prometheus & Grafana in Kubernetes

## KUBERNETES

namespace := "log-metrics"

# Build images for Minikube and load
k8s-build-load:
    podman build -t localhost/log-analyzer:latest -f Dockerfile.log-analyzer .
    podman tag localhost/log-analyzer:latest log-analyzer:latest
    podman save --format docker-archive -o log-analyzer.tar log-analyzer:latest
    minikube image load log-analyzer.tar
    podman build -t localhost/noise-maker:latest -f Dockerfile.noise-maker .
    podman tag localhost/noise-maker:latest noise-maker:latest
    podman save --format docker-archive -o noise-maker.tar noise-maker:latest
    minikube image load noise-maker.tar
    rm -f noise-maker.tar log-analyzer.tar

# Apply manifests
k8s-apply:
    kubectl apply -f k8s/namespace.yaml
    kubectl apply -f k8s/nats.yaml
    kubectl apply -f k8s/log-analyzer.yaml
    kubectl apply -f k8s/noise-maker.yaml
    kubectl apply -f k8s/prometheus.yaml
    kubectl apply -f k8s/grafana.yaml

# Tear down
k8s-delete:
    kubectl delete -f k8s/grafana.yaml --ignore-not-found
    kubectl delete -f k8s/prometheus.yaml --ignore-not-found
    kubectl delete -f k8s/noise-maker.yaml --ignore-not-found
    kubectl delete -f k8s/log-analyzer.yaml --ignore-not-found
    kubectl delete -f k8s/nats.yaml --ignore-not-found
    kubectl delete -f k8s/namespace.yaml --ignore-not-found

# End-to-end deploy
k8s-deploy: k8s-build-load k8s-apply

# Monitor
k8s-status:
    kubectl get pods -n {{namespace}}

# Port-forward Grafana
k8s-grafana:
    kubectl port-forward svc/grafana 3000:3000 -n {{namespace}}

# Port-forward Prometheus
k8s-prometheus:
    kubectl port-forward svc/prometheus 9090:9090 -n {{namespace}}

# Start Kubernetes locally
k8s-start:
    minikube start --cpus=4 --memory=4096 --addons=metrics-server

# Stop Kubernetes
k8s-stop:
    minikube stop

# Reset Minikube
k8s-reset:
    minikube delete
