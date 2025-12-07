use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use tokio::{select, task::JoinSet, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::{Metrics, MetricsResponse, http_server, run_server};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
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

pub async fn run_proxy(src_ip: &str, output_ip: &str) -> Result<(), AppError> {
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
