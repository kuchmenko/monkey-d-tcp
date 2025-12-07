use std::{
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use tokio::{
    net::TcpListener,
    select,
    task::{JoinHandle, JoinSet},
    time::sleep,
};
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

    #[error("Failded to parse address: {0}")]
    Parse(#[from] std::net::AddrParseError),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

pub struct Proxy {
    src_ip: String,
    output_ip: String,
    src_listener: TcpListener,
    metrics_addr: String,
    shutdown_token: CancellationToken,
    metrics: Arc<Metrics>,
}

impl Proxy {
    pub async fn bind(src_ip: &str, output_ip: &str) -> Result<(Self, SocketAddr), AppError> {
        let src_listener = TcpListener::bind(src_ip.parse::<SocketAddr>()?).await?;
        let local_addr = src_listener.local_addr()?;

        let proxy = Self {
            src_ip: src_ip.parse::<SocketAddr>()?.to_string(),
            output_ip: output_ip.to_string(),
            metrics_addr: "127.0.0.1:8998".to_string(),
            shutdown_token: CancellationToken::new(),
            metrics: Arc::new(Metrics::default()),
            src_listener,
        };

        Ok((proxy, local_addr))
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        println!("========================================");
        println!("       basic-tcp-proxy starting");
        println!("========================================");
        println!("Proxy listening on:    {}", self.src_ip);
        println!("Forwarding to:         {}", self.output_ip);
        println!(
            "Metrics endpoint:      http://{}/metrics",
            self.metrics_addr
        );
        println!("----------------------------------------");
        println!("Press Ctrl+C to shutdown gracefully");
        println!("========================================\n");

        let http_server = tokio::spawn(http_server(
            self.metrics.clone(),
            self.shutdown_token.clone(),
        ));

        let mut tasks_set = JoinSet::new();

        select! {
            _ = run_server(&self.src_listener, &self.output_ip, &self.shutdown_token, &mut tasks_set, self.metrics.clone()) => {},
            _ = self.shutdown_token.cancelled() => {
                println!("\n[SHUTDOWN] Received shutdown signal, starting graceful shutdown...");
                self.graceful_shutdown(tasks_set, http_server).await?;
            },
            _ = tokio::signal::ctrl_c() => {
            println!("\n[SHUTDOWN] Received Ctrl+C, starting graceful shutdown...");
                self.shutdown();
            },

        };

        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.shutdown_token.cancel();
    }

    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    async fn graceful_shutdown(
        &mut self,
        tasks_set: JoinSet<()>,
        http_server: JoinHandle<Result<(), AppError>>,
    ) -> Result<(), AppError> {
        let active = self.metrics.active_connections.load(Ordering::Relaxed);
        println!("[SHUTDOWN] Waiting for {} active connection(s)...", active);

        let force_handle = Self::start_force_timeout_task();

        tasks_set.join_all().await;
        http_server.await??;

        let final_metrics = MetricsResponse {
            active_connections: self.metrics.active_connections.load(Ordering::Relaxed),
            total_connections: self.metrics.total_connections.load(Ordering::Relaxed),
            bytes_upstream: self.metrics.bytes_upstream.load(Ordering::Relaxed),
            bytes_downstream: self.metrics.bytes_downstream.load(Ordering::Relaxed),
        };

        println!("\n========================================");
        println!("         Shutdown complete");
        println!("========================================");
        println!("Total connections: {}", final_metrics.total_connections);
        println!("Bytes upstream:    {}", final_metrics.bytes_upstream);
        println!("Bytes downstream:  {}", final_metrics.bytes_downstream);
        println!("========================================");

        force_handle.abort();

        Ok(())
    }

    fn start_force_timeout_task() -> JoinHandle<()> {
        tokio::spawn(async move {
            let graceful_period = Duration::from_secs(60);
            sleep(graceful_period).await;
            println!("[SHUTDOWN] Grace period expired, force exiting...");
            std::process::exit(0);
        })
    }
}
