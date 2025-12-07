use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{AppError, MetricEvent};

async fn accept_connection(
    src_listener: &TcpListener,
    output_ip: &str,
    graceful_token: &CancellationToken,
    tasks_set: &mut JoinSet<()>,
    metrics_tx: mpsc::Sender<MetricEvent>,
) -> Result<(), AppError> {
    let (stream_a, client_addr) = src_listener.accept().await?;
    let stream_b = TcpStream::connect(output_ip).await?;

    let _ = metrics_tx
        .send(MetricEvent::ConnectionOpened(client_addr))
        .await;

    let (mut a_read, mut a_write) = stream_a.into_split();
    let (mut b_read, mut b_write) = stream_b.into_split();

    let conn_token = CancellationToken::new();
    let conn_token_clone = conn_token.clone();

    let upstream_graceful = graceful_token.clone();
    let downstream_graceful = graceful_token.clone();

    let upstream_tx = metrics_tx.clone();
    let downstream_tx = metrics_tx;

    tasks_set.spawn(async move {
        loop {
            let mut buf = [0u8; 1024];
            select! {
                result = a_read.read(&mut buf) => {
                    match result {
                        Ok(0) | Err(_) => {
                            conn_token.cancel();
                            break;
                        }
                        Ok(n) => {
                            let _ = upstream_tx.send(MetricEvent::BytesUpstream(client_addr, n as u64)).await;
                            if b_write.write_all(&buf[..n]).await.is_err() {
                                conn_token.cancel();
                                break;
                            }
                        }
                    }
                }
                _ = upstream_graceful.cancelled() => {
                    conn_token.cancel();
                    break;
                }
                _ = conn_token.cancelled() => {
                    break;
                }
            }
        }
        let _ = upstream_tx.send(MetricEvent::ConnectionClosed(client_addr)).await;
    });

    tasks_set.spawn(async move {
        loop {
            let mut buf = [0u8; 1024];
            select! {
                result = b_read.read(&mut buf) => {
                    match result {
                        Ok(0) | Err(_) => {
                            conn_token_clone.cancel();
                            break;
                        }
                        Ok(n) => {
                            let _ = downstream_tx.send(MetricEvent::BytesDownstream(client_addr, n as u64)).await;
                            if a_write.write_all(&buf[..n]).await.is_err() {
                                conn_token_clone.cancel();
                                break;
                            }
                        }
                    }
                }
                _ = downstream_graceful.cancelled() => {
                    conn_token_clone.cancel();
                    break;
                }
                _ = conn_token_clone.cancelled() => {
                    break;
                }
            }
        }
    });

    Ok(())
}

pub async fn run_server(
    src_listener: &TcpListener,
    output_ip: &str,
    graceful_token: &CancellationToken,
    tasks_set: &mut JoinSet<()>,
    metrics_tx: mpsc::Sender<MetricEvent>,
) -> Result<(), AppError> {
    loop {
        select! {
            result = accept_connection(
                src_listener,
                output_ip,
                graceful_token,
                tasks_set,
                metrics_tx.clone(),
            ) => {
                if let Err(e) = result {
                    eprintln!("[ERROR] Failed to accept connection: {}", e);
                }
            }
            _ = graceful_token.cancelled() => {
                println!("Stopped accepting connections");
                return Ok(());
            }
        }
    }
}
