# basic-tcp-proxy

A minimal asynchronous TCP proxy with metrics, built in Rust using Tokio.

## Features

- **Async TCP Proxy** - High-performance async I/O with Tokio
- **Bidirectional Relay** - Full duplex data transfer
- **Graceful Shutdown** - SIGINT handling with 60s grace period
- **Metrics Endpoint** - Real-time JSON metrics via HTTP
- **Concurrent Connections** - Each connection in separate async task

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

## Configuration

| Setting     | Value            | Description               |
| ----------- | ---------------- | ------------------------- |
| Listen      | `127.0.0.1:8081` | Proxy accepts connections |
| Destination | `127.0.0.1:8080` | Forward traffic to        |
| Metrics     | `127.0.0.1:8998` | HTTP `/metrics` endpoint  |

## Metrics Response

```json
{
  "active_connections": 1,
  "total_connections": 5,
  "bytes_upstream": 1234,
  "bytes_downstream": 5678
}
```

## Architecture

```
Client :8081 ──▶ [Proxy] ──▶ Destination :8080
                   │
                   ▼
            HTTP :8998/metrics
```

## Graceful Shutdown

1. Press `Ctrl+C`
2. Stop accepting new connections
3. Wait for active connections (max 60s)
4. Exit
