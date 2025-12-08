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
| [load-tester](crates/load-tester)         | WIP    | Benchmarking and load testing  |
| [load-balancer](crates/load-balancer)     | WIP    | Multi-backend load balancer    |

## Quick Start

```bash
# Run the proxy (forwards localhost:8080 → localhost:8081)
just proxy

# Run echo server for testing
just echo

# Run tests
just test

# Run all quality checks
just quality
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
