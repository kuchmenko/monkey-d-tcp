use std::{net::SocketAddr, str::FromStr};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
pub enum EchoServerError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid address: {0}")]
    InvalidAddress(#[from] std::net::AddrParseError),
}

#[derive(Debug)]
pub struct EchoServer {
    listener: tokio::net::TcpListener,
    shutdown: CancellationToken,
}

impl EchoServer {
    pub async fn bind(addr: &str) -> Result<(Self, SocketAddr), EchoServerError> {
        let addr = SocketAddr::from_str(addr)?;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let shutdown = CancellationToken::new();
        let local_addr = listener.local_addr()?;
        Ok((EchoServer { listener, shutdown }, local_addr))
    }

    pub async fn run(&self) -> Result<(), EchoServerError> {
        let mut task_join_set = tokio::task::JoinSet::new();

        loop {
            select! {
                result = self.listener.accept() => {
                    if self.shutdown.is_cancelled() {
                        return Ok(());
                    }
                    let (stream, _) = result?;
                    task_join_set.spawn(async move {
                        let mut stream = stream;
                        let mut buf = [0u8; 1024];

                        loop {
                            let n = stream.read(&mut buf).await.unwrap();
                            if n == 0 {
                                break;
                            }

                            stream.write_all(&buf[..n]).await.unwrap();
                        }
                    });
                }
                _ = self.shutdown.cancelled() => {
                    println!("Stopped accepting connections");
                    task_join_set.join_all().await;
                    println!("Stopped echo server");
                    return Ok(());
                }
            }
        }
    }

    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }

    pub fn get_addr(&self) -> Result<SocketAddr, EchoServerError> {
        self.listener.local_addr().map_err(EchoServerError::IoError)
    }
}
