use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    task::JoinSet,
    time::sleep,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Default)]
struct Metrics {
    active_connections: AtomicU64,
    total_connections: AtomicU64,
    bytes_upstream: AtomicU64,
    bytes_downstream: AtomicU64,
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Socket error: {0}")]
    Socket(#[from] std::io::Error),

    #[error("Hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] hyper::http::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
}

async fn run_server(
    output_ip: &str,
    src_ip: &str,
    graceful_token: &CancellationToken,
    tasks_set: &mut JoinSet<()>,
    metrics: Arc<Metrics>,
) -> Result<(), AppError> {
    let input_listener = TcpListener::bind(src_ip).await?;

    loop {
        if graceful_token.is_cancelled() {
            println!("Stopped accepting connections");
            return Ok(());
        }

        let (stream_a, client_addr) = input_listener.accept().await?;
        println!("[CONN] New connection from {}", client_addr);

        let stream_b = TcpStream::connect(output_ip).await?;
        println!("[CONN] Connected to destination {}", output_ip);

        metrics.active_connections.fetch_add(1, Ordering::Relaxed);
        metrics.total_connections.fetch_add(1, Ordering::Relaxed);
        let conn_id = metrics.total_connections.load(Ordering::Relaxed);
        println!(
            "[STATS] Active: {} | Total: {}",
            metrics.active_connections.load(Ordering::Relaxed),
            conn_id
        );

        let (mut a_read, mut a_write) = stream_a.into_split();
        let (mut b_read, mut b_write) = stream_b.into_split();
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let read_graceful_token = graceful_token.clone();
        let write_graceful_token = graceful_token.clone();

        let task_tx = metrics.clone();
        let task_tx2 = metrics.clone();

        tasks_set.spawn(async move {
            loop {
                let mut buf = [0u8; 1024];
                select! {
                    result = a_read.read(&mut buf) => {
                        if let Ok(n) = result {
                            if n == 0 {
                                let active = task_tx.active_connections.fetch_sub(1, Ordering::Relaxed) - 1;
                                println!("[CLOSE] Client disconnected | Active: {}", active);
                                token.cancel();
                                break;
                            }
                            let total_up = task_tx.bytes_upstream.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
                            let total_down = task_tx.bytes_downstream.load(Ordering::Relaxed);
                            println!("[DATA] Upstream: {} bytes | Total: up={} down={}", n, total_up, total_down);
                            b_write.write_all(&buf[..n]).await.unwrap();
                        }
                    },
                    _ = read_graceful_token.cancelled() => {
                        let active = task_tx.active_connections.fetch_sub(1, Ordering::Relaxed) - 1;
                        println!("[SHUTDOWN] Closing upstream relay | Active: {}", active);
                        break;
                    }
                    _ = token.cancelled() => {
                        task_tx.active_connections.fetch_sub(1, Ordering::Relaxed);
                        break;
                    }
                };
            }
        });

        tasks_set.spawn(async move {
            loop {
                let mut buf = [0u8; 1024];
                select! {
                    result = b_read.read(&mut buf) => {
                        if let Ok(n) = result {
                            if n == 0 {
                                println!("[CLOSE] Destination disconnected");
                                token_clone.cancel();
                                break;
                            }
                            let total_down = task_tx2.bytes_downstream.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
                            let total_up = task_tx2.bytes_upstream.load(Ordering::Relaxed);
                            println!("[DATA] Downstream: {} bytes | Total: up={} down={}", n, total_up, total_down);
                            a_write.write_all(&buf[..n]).await.unwrap();
                        }
                    },
                    _ = write_graceful_token.cancelled() => {
                        println!("[SHUTDOWN] Closing downstream relay");
                        break;
                    }
                    _ = token_clone.cancelled() => {
                        break;
                    }
                };
            }
        });
    }
}

#[derive(Serialize)]
struct MetricsResponse {
    active_connections: u64,
    total_connections: u64,
    bytes_upstream: u64,
    bytes_downstream: u64,
}

async fn handle_http_request(
    req: Request<hyper::body::Incoming>,
    metrics: Arc<Metrics>,
) -> Result<Response<Full<Bytes>>, AppError> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("Hello, world!")))?;
            Ok(response)
        }
        (&hyper::Method::GET, "/metrics") => {
            let response = MetricsResponse {
                active_connections: metrics.active_connections.load(Ordering::Relaxed),
                total_connections: metrics.total_connections.load(Ordering::Relaxed),
                bytes_upstream: metrics.bytes_upstream.load(Ordering::Relaxed),
                bytes_downstream: metrics.bytes_downstream.load(Ordering::Relaxed),
            };
            let json = serde_json::to_string(&response)?;
            let body = Full::new(Bytes::from(json));
            let res = Response::builder()
                .header("Content-Type", "application/json")
                .body(body)?;

            Ok(res)
        }
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("Not found")))?;
            Ok(response)
        }
    }
}

async fn http_server(
    metrics: Arc<Metrics>,
    graceful_token: CancellationToken,
) -> Result<(), AppError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8998));
    let listener = TcpListener::bind(addr).await?;

    loop {
        select! {
            result = listener.accept() => {
                let (stream, _) = result?;
                let io = TokioIo::new(stream);

                let metrics = metrics.clone();

                let service = service_fn(move |req| {
                    let metrics = metrics.clone();
                    async move { handle_http_request(req, metrics).await }
                });

            tokio::spawn(async move {
                let _ = http1::Builder::new()
                    .serve_connection(io, service)
                .await;
            });

            }
            _ = graceful_token.cancelled() => {
                println!("Stopping HTTP server");
                return Ok(());
            },

        }
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let output_ip = "127.0.0.1:8080";
    let src_ip = "127.0.0.1:8081";
    let metrics_addr = "127.0.0.1:8998";
    let graceful_token = CancellationToken::new();
    let mut server_tasks_set = JoinSet::new();
    let metrics = Arc::new(Metrics::default());

    println!("========================================");
    println!("       basic-tcp-proxy starting");
    println!("========================================");
    println!("Proxy listening on:    {}", src_ip);
    println!("Forwarding to:         {}", output_ip);
    println!("Metrics endpoint:      http://{}/metrics", metrics_addr);
    println!("----------------------------------------");
    println!("Press Ctrl+C to shutdown gracefully");
    println!("========================================\n");

    let http_server_handle = tokio::spawn(http_server(metrics.clone(), graceful_token.clone()));

    select! {
        _ = run_server(output_ip, src_ip, &graceful_token, &mut server_tasks_set, metrics.clone()) => {},
        _ = tokio::signal::ctrl_c() => {
            println!("\n[SHUTDOWN] Received Ctrl+C, starting graceful shutdown...");
            graceful_token.cancel();

            let active = metrics.active_connections.load(Ordering::Relaxed);
            println!("[SHUTDOWN] Waiting for {} active connection(s)...", active);

            let force_handle = tokio::spawn(async move {
                let graceful_period = Duration::from_secs(60);
                sleep(graceful_period).await;
                println!("[SHUTDOWN] Grace period expired, force exiting...");
                std::process::exit(0);
            });

            server_tasks_set.join_all().await;
            http_server_handle.await??;

            let final_metrics = MetricsResponse {
                active_connections: metrics.active_connections.load(Ordering::Relaxed),
                total_connections: metrics.total_connections.load(Ordering::Relaxed),
                bytes_upstream: metrics.bytes_upstream.load(Ordering::Relaxed),
                bytes_downstream: metrics.bytes_downstream.load(Ordering::Relaxed),
            };

            println!("\n========================================");
            println!("         Shutdown complete");
            println!("========================================");
            println!("Total connections: {}", final_metrics.total_connections);
            println!("Bytes upstream:    {}", final_metrics.bytes_upstream);
            println!("Bytes downstream:  {}", final_metrics.bytes_downstream);
            println!("========================================");

            force_handle.abort();
        },
    };

    Ok(())
}
