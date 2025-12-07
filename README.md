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

[![CI](https://github.com/user/monkey-d-tcp/actions/workflows/ci.yml/badge.svg)](https://github.com/user/monkey-d-tcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

> _"I'm gonna be King of the Proxies!"_ — Monkey D. TCP

A high-performance async TCP proxy built with Tokio. Features real-time metrics, graceful shutdown, and zero-copy forwarding.

## Features

- **Async I/O** — Built on Tokio for maximum concurrency
- **Bidirectional Relay** — Full-duplex TCP forwarding
- **Real-time Metrics** — Connection tracking, bytes transferred, per-client stats
- **HTTP Metrics Endpoint** — Prometheus-compatible `/metrics` endpoint
- **Graceful Shutdown** — Ctrl+C handling with configurable grace period
- **Channel-based Architecture** — mpsc for events, watch for state broadcasting

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                           Proxy                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Client ──┬── Upstream ──► Target Server                       │
│            │                     │                               │
│            └── Downstream ◄──────┘                               │
│                    │                                             │
│                    ▼                                             │
│   ┌────────────────────────────────────┐                        │
│   │         MetricEvent (mpsc)         │                        │
│   │  • ConnectionOpened(addr)          │                        │
│   │  • ConnectionClosed(addr)          │                        │
│   │  • BytesUpstream(addr, n)          │                        │
│   │  • BytesDownstream(addr, n)        │                        │
│   └──────────────┬─────────────────────┘                        │
│                  ▼                                               │
│   ┌────────────────────────────────────┐                        │
│   │       MetricsCollector             │                        │
│   │  • Aggregates state                │                        │
│   │  • Logs events                     │                        │
│   │  • Periodic summaries              │                        │
│   └──────────────┬─────────────────────┘                        │
│                  ▼                                               │
│   ┌────────────────────────────────────┐                        │
│   │    MetricsSnapshot (watch)         │──────► HTTP Server     │
│   │  • active_connections              │        /metrics        │
│   │  • total_connections               │                        │
│   │  • bytes_upstream                  │                        │
│   │  • bytes_downstream                │                        │
│   └────────────────────────────────────┘                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

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

## Usage

```rust
use basic_tcp_proxy::Proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut proxy, addr) = Proxy::bind("127.0.0.1:8080", "example.com:80").await?;
    println!("Proxy listening on {}", addr);
    proxy.run().await?;
    Ok(())
}
```

## Metrics

### HTTP Endpoint

```bash
# Plain text (default)
curl http://localhost:8998/metrics

# JSON
curl http://localhost:8998/metrics?format=json
```

**Plain text output:**

```
connections_active 3
connections_total 42
bytes_upstream 1048576
bytes_downstream 2097152
```

**JSON output:**

```json
{
  "active_connections": 3,
  "total_connections": 42,
  "bytes_upstream": 1048576,
  "bytes_downstream": 2097152
}
```

### Console Logging

**Event logging:**

```
[METRICS] ConnectionOpened 127.0.0.1:54321 | active: 1, total: 1
[METRICS] 127.0.0.1:54321 ↑ 1024 | total up: 1.0KB
[METRICS] 127.0.0.1:54321 ↓ 512 | total down: 512B
[METRICS] ConnectionClosed 127.0.0.1:54321 | active: 0
```

**Periodic summary (every 10s):**

```
[METRICS] active: 3 | total: 15 | up: 1.5MB | down: 800.0KB
```

## Project Structure

```
crates/
├── basic-tcp-proxy/     # Main proxy implementation
│   ├── src/
│   │   ├── proxy.rs     # Proxy struct and lifecycle
│   │   ├── relay.rs     # Bidirectional TCP relay
│   │   ├── metrics.rs   # MetricsCollector and events
│   │   └── http_server.rs
│   └── tests/
│       └── proxy_test.rs
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

- Rust 1.75+
- Tokio runtime

## License

MIT
