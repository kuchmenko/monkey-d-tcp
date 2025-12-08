# Monkey D. TCP

```
    ⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣴⣶⣾⣿⣿⣿⣿⣷⣶⣦⣤⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⠀⠀⣠⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⣄⠀⠀⠀⠀⠀⠀
    ⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀⠀
    ⠀⠀⢀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡀⠀⠀
    ⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄⠀
    ⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄
    ⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
              STRETCHING TCP SINCE 2025
```

[![CI](https://github.com/kuchmenko/monkey-d-tcp/actions/workflows/ci.yml/badge.svg)](https://github.com/kuchmenko/monkey-d-tcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)

> _"I'm gonna be King of the Networking!"_ — Monkey D. TCP

Network programming playground. The Grand Line where packets learn to fight.

## The Crew

| Crate                                     | Status | Description                    |
| ----------------------------------------- | ------ | ------------------------------ |
| [basic-tcp-proxy](crates/basic-tcp-proxy) | Ready  | Async TCP proxy with metrics   |
| [echo-server](crates/echo-server)         | Ready  | Simple echo server for testing |
| [load-tester](crates/load-tester)         | Ready  | Benchmarking and load testing  |
| [load-balancer](crates/load-balancer)     | WIP    | Multi-backend load balancer    |

## Quick Start

```bash
# Run the proxy (forwards localhost:3000 → localhost:8081)
just proxy

# Run echo server for testing
just echo

# Run tests
just test

# Run all quality checks
just quality
```

## Performance

The proxy sustains **~90k requests/sec** with sub-millisecond latency.

| Metric | Value |
|--------|-------|
| Peak RPS | 91,599 |
| p50 Latency | 110μs |
| p99 Latency | 294μs |
| Throughput | 90 MB/s |
| Max Connections | 500+ |

See [basic-tcp-proxy README](crates/basic-tcp-proxy/README.md#benchmarks) for full benchmark matrix.

## Load Testing

Run the built-in load tester to benchmark the proxy:

```bash
# Terminal 1: Start echo server
just echo

# Terminal 2: Start proxy
just proxy

# Terminal 3: Run load test
just load-test
```

Configure test scenarios in `load_test.toml`:

```toml
target_addr = "127.0.0.1:3000"

[[scenarios]]
name = "baseline"
connections = 10
duration_secs = 5
message_size = 1024

[[scenarios]]
name = "stress-test"
connections = 500
duration_secs = 5
message_size = 1024
```

## Project Structure

```
crates/
├── basic-tcp-proxy/     # Async TCP proxy with metrics
├── echo-server/         # Simple echo server for testing
├── load-balancer/       # (WIP)
└── load-tester/         # (WIP)
```

## Development

```bash
just fmt          # Format code
just clippy       # Run lints
just test         # Run tests
just quality      # All checks
just fix          # Auto-fix issues
```

## Requirements

- Rust 1.85+ (edition 2024)
- Tokio runtime

## License

MIT
