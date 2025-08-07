# Use bash with strict mode-ish
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# --------
# Config
# --------
namespace := "log-metrics"
arch      := "x86_64-unknown-linux-musl"

# Run tests (unchanged)
test:
    cargo test --no-default-features # -- --include-ignored

# Build Rust binaries for MUSL
build-rs:
    cargo build --release --target {{arch}}

# Build images and load them into Minikube (was build-pod)
# Requires: podman, minikube
build: build-rs
    # log-analyzer
    podman build -t localhost/log-analyzer:latest -f Dockerfile.log-analyzer .
    podman save --format oci-archive -o log-analyzer.tar localhost/log-analyzer:latest
    minikube image load log-analyzer.tar --overwrite=true
    # noise-maker
    podman build -t localhost/noise-maker:latest -f Dockerfile.noise-maker .
    podman save --format oci-archive -o noise-maker.tar localhost/noise-maker:latest
    minikube image load noise-maker.tar --overwrite=true
    # clean up
    rm -f log-analyzer.tar noise-maker.tar

# Start the local k8s stack end-to-end (tests + build + deploy)
start: test build k8s-start k8s-apply k8s-wait

# Stop the k8s stack (delete manifests, keep cluster running)
stop: k8s-delete

# Restart deployments in-place (no image rebuild)
restart:
    kubectl rollout restart deployment -n {{namespace}} --all
    just k8s-wait

# Clean local artifacts + tear down k8s resources
clean:
    just stop
    cargo clean

# Build log-analyzer with profiling
build-pprof:
    cargo build --profile pprof --features pprof -p log-analyzer
    podman-compose build --no-cache

# Run profiling: Usage: just profile 30 100000 1
# - shutdown_after = seconds to run profiler before shutdown
# - rate = number of log lines per second; after 10,000 throttling is disabled
# - batch_size = number of log lines per NATS message
profile shutdown_after rate batch_size:
    mkdir -p profile logs
    just build-pprof
    trap 'podman-compose down' EXIT
    BATCH_SIZE={{batch_size}} RATE={{rate}} podman-compose up -d --force-recreate nats noise-maker
    cargo run --profile pprof --features pprof -p log-analyzer -- \
      --nats-url nats://127.0.0.1:4222 \
      --shutdown-after {{shutdown_after}} \
      --log-file logs/log-analyzer-profile.log

# Start local cluster (Minikube)
# - Only set CPU/mem on first creation to avoid errors on existing clusters.
k8s-start:
    if minikube status >/dev/null 2>&1; then \
      echo "Minikube exists; starting without resource flags..."; \
      minikube start; \
    else \
      minikube start --cpus=4 --memory=4096 --addons=metrics-server; \
    fi

# Ensure namespace exists and is Active (auto-fixes Terminating if jq is available)
k8s-ensure-namespace:
    if ! kubectl get ns {{namespace}} >/dev/null 2>&1; then \
      kubectl apply -f k8s/namespace.yaml; \
    fi; \
    echo "Waiting for namespace {{namespace}} to be Active..."; \
    for i in $$(seq 1 60); do \
      phase=$$(kubectl get ns {{namespace}} -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound"); \
      if [ "$$phase" = "Active" ]; then echo "Namespace Active"; break; fi; \
      if [ "$$phase" = "Terminating" ] && command -v jq >/dev/null 2>&1 && [ "$$i" -eq 30 ]; then \
        echo "Namespace stuck in Terminating; attempting to clear finalizers..."; \
        kubectl get ns {{namespace}} -o json \
          | jq 'del(.spec

# Apply manifests (namespace first)
k8s-apply:
    kubectl apply -f k8s/namespace.yaml
    kubectl apply -f k8s/nats.yaml -n {{namespace}}
    kubectl apply -f k8s/log-analyzer.yaml -n {{namespace}}
    kubectl apply -f k8s/noise-maker.yaml -n {{namespace}}
    kubectl apply -f k8s/prometheus.yaml -n {{namespace}}
    kubectl apply -f k8s/grafana.yaml -n {{namespace}}

# Wait for all Deployments in the namespace to become ready
k8s-wait:
    # Wait for any Deployments that exist (no-op if none)
    if kubectl get deploy -n {{namespace}} >/dev/null 2>&1; then \
      for d in $$(kubectl get deploy -n {{namespace}} -o name); do \
        kubectl rollout status "$$d" -n {{namespace}} --timeout=120s; \
      done; \
    fi

# Delete manifests (safe with ignore-not-found flags)
k8s-delete:
    kubectl delete -f k8s/grafana.yaml     -n {{namespace}} --ignore-not-found
    kubectl delete -f k8s/prometheus.yaml  -n {{namespace}} --ignore-not-found
    kubectl delete -f k8s/noise-maker.yaml -n {{namespace}} --ignore-not-found
    kubectl delete -f k8s/log-analyzer.yaml -n {{namespace}} --ignore-not-found
    kubectl delete -f k8s/nats.yaml        -n {{namespace}} --ignore-not-found
    kubectl delete -f k8s/namespace.yaml   --ignore-not-found

# Quick status
k8s-status:
    kubectl get pods,svc,deploy -n {{namespace}}

# Port-forwards
k8s-grafana:
    kubectl port-forward svc/grafana 3000:3000 -n {{namespace}}

k8s-prometheus:
    kubectl port-forward svc/prometheus 9090:9090 -n {{namespace}}

# Stop & reset Minikube (optional)
k8s-stop:
    minikube stop

k8s-reset:
    minikube delete
