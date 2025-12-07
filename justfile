# Default recipe
default:
    @just --list

# Run proxy
proxy:
    cargo run -p basic-tcp-proxy

# Run echo server
echo:
    cargo run -p echo-server

# Run all tests
test:
    cargo test --workspace

# Format code
fmt:
    cargo fmt --all

# Check formatting without applying
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
clippy:
    cargo clippy --workspace --all-targets -- -D warnings

# Run clippy and fix automatically
clippy-fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty

# Check code compiles
check:
    cargo check --workspace

# Build release
build:
    cargo build --workspace --release

# Run all quality checks (format check + clippy + test)
quality:
    @echo "Checking formatting..."
    cargo fmt --all -- --check
    @echo "Running clippy..."
    cargo clippy --workspace --all-targets -- -D warnings
    @echo "Running tests..."
    cargo test --workspace
    @echo "All checks passed!"

# Fix all auto-fixable issues (format + clippy fix)
fix:
    cargo fmt --all
    cargo clippy --workspace --all-targets --fix --allow-dirty

# Clean build artifacts
clean:
    cargo clean
