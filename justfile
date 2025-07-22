test:
    cargo test --all-targets --all-features

build:
    cargo build --release --target x86_64-unknown-linux-musl

start-docker:
    docker-compose up --build

stop:
    docker-compose down

clean-rs:
    cargo clean

start: test build start-docker
    
restart: stop start

clean: stop clean-rs
