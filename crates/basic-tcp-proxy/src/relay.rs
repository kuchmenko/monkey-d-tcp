use std::sync::{Arc, atomic::Ordering};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{AppError, Metrics};

async fn accept_connection(
    src_listener: &TcpListener,
    output_ip: &str,
    graceful_token: &CancellationToken,
    tasks_set: &mut JoinSet<()>,
    metrics: Arc<Metrics>,
) -> Result<(), AppError> {
    let (stream_a, client_addr) = src_listener.accept().await?;
    println!("[CONN] New connection from {}", client_addr);
    let stream_b = TcpStream::connect(output_ip).await?;
    println!("[CONN] Connected to destination {}", output_ip);

    metrics.active_connections.fetch_add(1, Ordering::Relaxed);
    let conn_id = metrics.total_connections.fetch_add(1, Ordering::Relaxed);
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
                            token.cancel();
                    }
                    _ = token.cancelled() => {
                         let active = task_tx.active_connections.fetch_sub(1,Ordering::Relaxed);
                        println!("[SHUTDOWN] Closing upstream relay | Active: {}", active);

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
                            token_clone.cancel();
                    }
                    _ = token_clone.cancelled() => {
                                                     let active = task_tx2.active_connections.fetch_sub(1,Ordering::Relaxed);
                        println!("[SHUTDOWN] Closing downstream relay | Active: {}", active);

                        break;
                    }
                };
            }
        });
    Ok(())
}

pub async fn run_server(
    src_listener: &TcpListener,
    output_ip: &str,
    graceful_token: &CancellationToken,
    tasks_set: &mut JoinSet<()>,
    metrics: Arc<Metrics>,
) -> Result<(), AppError> {
    loop {
        select! {
            _ = accept_connection(
                src_listener,
                output_ip,
                graceful_token,
                tasks_set,
                metrics.clone(),
        ) => {}
        _ = graceful_token.cancelled() => {
                println!("Stopped accepting connections");
                return Ok(());
            },
        };
    }
}
