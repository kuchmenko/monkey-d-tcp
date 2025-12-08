# basic-tcp-proxy

> High-performance async TCP proxy built with Tokio

A minimal asynchronous TCP proxy with real-time metrics, graceful shutdown, and channel-based architecture.

## Features

- **Async I/O** — Built on Tokio for maximum concurrency
- **Bidirectional Relay** — Full-duplex TCP forwarding
- **Real-time Metrics** — Connection tracking, bytes transferred, per-client stats
- **HTTP Metrics Endpoint** — Prometheus-compatible `/metrics` endpoint
- **Graceful Shutdown** — Ctrl+C handling with configurable grace period
- **Channel-based Architecture** — mpsc for events, watch for state broadcasting

## Benchmarks

Tested on AMD Ryzen 9 7950X3D (16c/32t), 96GB DDR5 6400MHz, Arch Linux.

| Scenario      | Connections | Message Size | RPS        | p50   | p99   | Throughput |
| ------------- | ----------- | ------------ | ---------- | ----- | ----- | ---------- |
| Baseline      | 10          | 1 KB         | **83,394** | 110μs | 294μs | 83 MB/s    |
| Small packets | 10          | 64 B         | **85,709** | 107μs | 278μs | 5.5 MB/s   |
| Medium load   | 50          | 1 KB         | **91,599** | 447μs | 2.4ms | 92 MB/s    |
| Heavy load    | 100         | 1 KB         | **89,580** | 923μs | 3.4ms | 90 MB/s    |
| Stress test   | 500         | 1 KB         | **90,411** | 5.0ms | 8.9ms | 90 MB/s    |
| Large packets | 10          | 8 KB         | 171        | 42ms  | 84ms  | 1.4 MB/s   |
| Big data      | 10          | 64 KB        | 221        | -     | 84ms  | 14 MB/s    |

**Key findings:**

- Sustains **~90k RPS** with sub-millisecond p50 latency
- Scales linearly from 10 to 500 concurrent connections
- Optimized for small-to-medium packets (< 8KB)
- Zero errors under all test conditions

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
# Terminal 1: Start destination server
nc -l 8080

# Terminal 2: Start proxy
cargo run -p basic-tcp-proxy

# Terminal 3: Connect through proxy
nc localhost 8081

# Terminal 4: Check metrics
curl http://localhost:8998/metrics
```

Or using just:

```bash
just echo   # Start echo server on :8081
just proxy  # Start proxy :8080 → :8081
```

## Configuration

Create a `proxy.toml` file:

```toml
# Proxy listen address
listen_addr = "127.0.0.1:3000"

# Target to forward connections to
target_addr = "127.0.0.1:8081"

# HTTP metrics endpoint
metrics_addr = "127.0.0.1:9090"

# Graceful shutdown timeout (seconds)
grace_period_secs = 30

# Metrics logging interval (seconds)
metrics_log_interval_secs = 10

# Channel buffer size for metrics events
channel_buffer_size = 1000
```

## Usage

### From TOML config

```rust
use basic_tcp_proxy::{Config, Proxy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("proxy.toml")?;
    let (mut proxy, addr) = Proxy::new(config).await?;
    println!("Proxy listening on {}", addr);
    proxy.run().await?;
    Ok(())
}
```

### Programmatic config

```rust
use basic_tcp_proxy::{Config, Proxy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config {
        listen_addr: "127.0.0.1:8080".to_string(),
        target_addr: "example.com:80".to_string(),
        ..Config::default()
    };
    let (mut proxy, addr) = Proxy::new(config).await?;
    println!("Proxy listening on {}", addr);
    proxy.run().await?;
    Ok(())
}
```

## Metrics

### HTTP Endpoint

```bash
# Plain text (default)
curl http://localhost:9090/metrics

# JSON
curl http://localhost:9090/metrics?format=json
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

## Graceful Shutdown

1. Press `Ctrl+C`
2. Stop accepting new connections
3. Wait for active connections (configurable grace period)
4. Exit
