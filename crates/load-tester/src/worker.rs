use std::time::{Duration, Instant};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    select,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Default)]
pub struct WorkerStats {
    pub requests: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latencies: Vec<Duration>,
    pub errors: u64,
}

pub async fn run_worker(
    target_addr: String,
    message_size: usize,
    shutdown: CancellationToken,
) -> WorkerStats {
    let mut stats = WorkerStats::default();
    let message = vec![b'x'; message_size];
    let mut recv_buf = vec![0u8; message_size];

    let Ok(mut stream) = TcpStream::connect(&target_addr).await else {
        stats.errors += 1;
        return stats;
    };

    loop {
        select! {
            biased;

            _ = shutdown.cancelled() => {
                break;
            }

            result = send_receive(&mut stream, &message, &mut recv_buf) => {
                if let Ok((sent, received, latency)) = result {
                    stats.requests += 1;
                    stats.bytes_sent += sent as u64;
                    stats.bytes_received += received as u64;
                    stats.latencies.push(latency);
                } else {
                    stats.errors += 1;
                    if let Ok(new_stream) = TcpStream::connect(&target_addr).await {
                        stream = new_stream;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    stats
}

async fn send_receive(
    stream: &mut TcpStream,
    message: &[u8],
    recv_buf: &mut [u8],
) -> Result<(usize, usize, Duration), std::io::Error> {
    let start = Instant::now();

    stream.write_all(message).await?;
    let n = stream.read_exact(recv_buf).await?;

    let latency = start.elapsed();
    Ok((message.len(), n, latency))
}
