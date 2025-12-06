use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
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

    let input_listener = TcpListener::bind(src_ip).await?;

    loop {
        let (mut stream_a, _) = input_listener.accept().await?;
        let stream_b = TcpStream::connect(output_ip).await?;

        let (mut a_read, mut a_write) = stream_a.into_split();
        let (mut b_read, mut b_write) = stream_b.into_split();

        tokio::spawn(async move {
            loop {
                let mut buf = [0; 1024];
                let n = a_read.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                b_write.write_all(&buf[..n]).await.unwrap();
            }
        });

        tokio::spawn(async move {
            loop {
                let mut buf = [0; 1024];
                let n = b_read.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
                a_write.write_all(&buf[..n]).await.unwrap();
            }
        });
    }
}
