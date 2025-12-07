use std::{
    net::SocketAddr,
    sync::{Arc, atomic::Ordering},
};

use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::{net::TcpListener, select};
use tokio_util::sync::CancellationToken;

use crate::{AppError, Metrics};

#[derive(Serialize)]
pub struct MetricsResponse {
    pub active_connections: u64,
    pub total_connections: u64,
    pub bytes_upstream: u64,
    pub bytes_downstream: u64,
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

pub async fn http_server(
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
