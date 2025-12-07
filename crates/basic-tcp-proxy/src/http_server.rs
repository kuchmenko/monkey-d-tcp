use http_body_util::Full;
use hyper::{Request, Response, StatusCode, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, select, sync::watch};
use tokio_util::sync::CancellationToken;

use crate::{AppError, MetricsSnapshot};

fn parse_format_param(uri: &hyper::Uri) -> &str {
    uri.query()
        .and_then(|q| {
            q.split('&').find_map(|pair| {
                let (key, value) = pair.split_once('=')?;
                if key == "format" { Some(value) } else { None }
            })
        })
        .unwrap_or("text")
}

fn handle_http_request(
    req: &Request<hyper::body::Incoming>,
    metrics_rx: &watch::Receiver<MetricsSnapshot>,
) -> Result<Response<Full<Bytes>>, AppError> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from("Hello, world!")))?;
            Ok(response)
        }
        (&hyper::Method::GET, "/metrics") => {
            let snapshot = metrics_rx.borrow().clone();
            let format = parse_format_param(req.uri());

            let (content_type, body) = match format {
                "json" => {
                    let json = serde_json::to_string(&snapshot)?;
                    ("application/json", json)
                }
                _ => ("text/plain", snapshot.to_plain_text()),
            };

            let res = Response::builder()
                .header("Content-Type", content_type)
                .body(Full::new(Bytes::from(body)))?;

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
    listener: TcpListener,
    metrics_rx: watch::Receiver<MetricsSnapshot>,
    graceful_token: CancellationToken,
) -> Result<(), AppError> {
    loop {
        select! {
            result = listener.accept() => {
                let (stream, _) = result?;
                let io = TokioIo::new(stream);

                let metrics_rx = metrics_rx.clone();

                let service = service_fn(move |req| {
                    let metrics_rx = metrics_rx.clone();
                    async move { handle_http_request(&req, &metrics_rx) }
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
            }
        }
    }
}
