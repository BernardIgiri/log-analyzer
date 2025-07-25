# Run tests
test:
    cargo test --all-targets --all-features

# Build Rust binaries for musl target
build-rs:
    cargo build --release --target x86_64-unknown-linux-musl

# Build podman containers
build-pod:
    podman-compose build

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
