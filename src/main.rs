use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
    time::sleep,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Socket error: {0}")]
    SocketError(#[from] std::io::Error),
}

async fn run_server(
    output_ip: &str,
    src_ip: &str,
    graceful_token: &CancellationToken,
) -> Result<(), AppError> {
    let input_listener = TcpListener::bind(src_ip).await?;

    loop {
        if graceful_token.is_cancelled() {
            println!("Stopped accepting connections");
            return Ok(());
        }

        let (mut stream_a, _) = input_listener.accept().await?;
        let stream_b = TcpStream::connect(output_ip).await?;

        let (mut a_read, mut a_write) = stream_a.into_split();
        let (mut b_read, mut b_write) = stream_b.into_split();
        let token = CancellationToken::new();
        let token_clone = token.clone();

        let read_graceful_token = graceful_token.clone();
        let write_graceful_token = graceful_token.clone();

        tokio::spawn(async move {
            loop {
                let mut buf = [0u8; 1024];
                select! {
                    result = a_read.read(&mut buf) => {
                        if let Ok(n) = result {
                            if n == 0 {
                                token.cancel();
                                break;
                            }
                            b_write.write_all(&buf[..n]).await.unwrap();
                        }
                    },
                    _ = read_graceful_token.cancelled() => {
                            println!("Gracefully stopping read for client {}", a_read.peer_addr().unwrap());
                            break;
                        }
                    _ = token.cancelled() => {
                        println!("Write stream is closed for client {}", a_read.peer_addr().unwrap());
                        break;
                    }
                };
            }
        });

        tokio::spawn(async move {
            loop {
                let mut buf = [0u8; 1024];
                select! {
                    result = b_read.read(&mut buf) => {
                        if let Ok(n) = result {
                            if n == 0 {
                                token_clone.cancel();
                                break;
                            }
                            a_write.write_all(&buf[..n]).await.unwrap();
                        }
                    },
                    _ = write_graceful_token.cancelled() => {
                        println!("Gracefully stopping write for dest client {}", b_read.peer_addr().unwrap());
                        break;
                    }
                    _ = token_clone.cancelled() => {
                        println!("Read stream is closed for dest client {}", b_read.peer_addr().unwrap());
                        break;
                    }
                };
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_ip = "127.0.0.1:8080";
    let src_ip = "127.0.0.1:8081";
    let graceful_token = CancellationToken::new();

    select! {
        _ = run_server(output_ip, src_ip, &graceful_token) => {},
        _ = tokio::signal::ctrl_c() => {
            graceful_token.cancel();
            let graceful_period = Duration::from_secs(1);

            sleep(graceful_period).await;
            println!("Exiting...");
            std::process::exit(0);
        },
    };

    Ok(())
}
