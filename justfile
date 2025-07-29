# Run tests
test:
    cargo test --no-default-features

# Build Rust binaries for musl target
build-rs:
    cargo build --release --target x86_64-unknown-linux-musl

# Build podman containers
build-pod:
    podman-compose build --no-cache

# Builds everything
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

# Restart services
restart: stop start

# Full clean
clean: stop clean-rs


## KUBERNETES STUFF

# Namespace for Kubernetes resources
namespace := "log-metrics"

# Build images using Podman
k8s-build-load:
    podman build -t localhost/log-analyzer:latest -f Dockerfile.log-analyzer .
    podman tag localhost/log-analyzer:latest log-analyzer:latest
    rm -f log-analyzer.tar
    podman save --format docker-archive -o log-analyzer.tar log-analyzer:latest
    minikube image load log-analyzer.tar
    podman build -t localhost/noise-maker:latest -f Dockerfile.noise-maker .
    podman tag localhost/noise-maker:latest noise-maker:latest
    rm -f noise-maker.tar
    podman save --format docker-archive -o noise-maker.tar noise-maker:latest
    minikube image load noise-maker.tar

# Apply all Kubernetes manifests
k8s-apply:
    kubectl apply -f k8s/namespace.yaml
    kubectl apply -f k8s/nats.yaml
    kubectl apply -f k8s/log-analyzer.yaml
    kubectl apply -f k8s/noise-maker.yaml
    kubectl apply -f k8s/prometheus.yaml
    kubectl apply -f k8s/grafana.yaml

# Delete all resources
k8s-delete:
    kubectl delete -f k8s/grafana.yaml --ignore-not-found
    kubectl delete -f k8s/prometheus.yaml --ignore-not-found
    kubectl delete -f k8s/noise-maker.yaml --ignore-not-found
    kubectl delete -f k8s/log-analyzer.yaml --ignore-not-found
    kubectl delete -f k8s/nats.yaml --ignore-not-found
    kubectl delete -f k8s/namespace.yaml --ignore-not-found

# Full deploy: build, load, and apply
k8s-deploy: k8s-build-load k8s-apply

# View all pods in the namespace
k8s-status:
    kubectl get pods -n {{namespace}}

# Port-forward Grafana (localhost:3000)
k8s-grafana:
    kubectl port-forward svc/grafana 3000:3000 -n {{namespace}}

# Port-forward Prometheus (localhost:9090)
k8s-prometheus:
    kubectl port-forward svc/prometheus 9090:9090 -n {{namespace}}

# Start Minikube with enough resources for your stack
k8s-start:
    minikube start --cpus=4 --memory=4096 --addons=metrics-server

# Stop Minikube
k8s-stop:
    minikube stop

# Delete Minikube cluster
k8s-reset:
    minikube delete
