use std::sync::Arc;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{RwLock, mpsc},
};

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Socket error: {0}")]
    SocketError(std::io::Error),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_ip = "127.0.0.1:8080";
    let src_ip = "127.0.0.1:8081";

    // let mut dest_stream = TcpListener::connect(dest_ip).await.unwrap();
    let input_listener = TcpListener::bind(src_ip).await?;

    loop {
        let (mut input_stream, _) = input_listener.accept().await?;

        let (tx, mut rx) = mpsc::channel(10);

        let input_stream = Arc::new(RwLock::new(input_stream));
        let read_stream = input_stream.clone();
        let write_stream = input_stream.clone();

        tokio::spawn(async move {
            let input_stream = read_stream.read().await;

            loop {
                let mut buf = [0; 1024];
                let n = input_stream.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                tx.send(buf).await.unwrap();
            }
        });

        tokio::spawn(async move {
            let mut output_stream = TcpStream::connect(output_ip).await.unwrap();
            loop {
                let buf = rx.recv().await.unwrap();
                output_stream.write_all(&buf).await.unwrap();
            }
        });

        tokio::spawn(async move {
            let mut output_listener = TcpListener::bind(output_ip).await.unwrap();
            let mut input_stream = write_stream.write().await;
            loop {
                let (mut output_stream, _) = output_listener.accept().await.unwrap();
                let mut buf = [0; 1024];
                let n = output_stream.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                input_stream.write_all(&buf).await.unwrap();
            }
        });
    }
}
