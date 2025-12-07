default:
    @just --list

proxy:
    cargo run -p basic-tcp-proxy

echo:
    cargo run -p echo-server

test:
    cargo test --workspace

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

clippy-fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty

check:
    cargo check --workspace

build:
    cargo build --workspace --release

quality:
    @echo "Checking formatting..."
    cargo fmt --all -- --check
    @echo "Running clippy..."
    cargo clippy --workspace --all-targets -- -D warnings
    @echo "Running tests..."
    cargo test --workspace
    @echo "All checks passed!"

fix:
    cargo fmt --all
    cargo clippy --workspace --all-targets --fix --allow-dirty

clean:
    cargo clean
