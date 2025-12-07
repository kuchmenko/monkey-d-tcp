use basic_tcp_proxy::Proxy;
use echo_server::EchoServer;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_proxy() {
    let (echo_server, echo_addr) = EchoServer::bind("127.0.0.1:0").await.unwrap();
    let echo_server_handle = tokio::spawn(async move {
        echo_server.run().await.unwrap();
    });

    let (mut proxy_server, proxy_addr) = Proxy::bind("127.0.0.1:0", &echo_addr.to_string())
        .await
        .unwrap();
    let proxy_metrics = proxy_server.metrics().clone();

    let proxy_server_handle = tokio::spawn(async move {
        proxy_server.run().await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut proxy_stream = tokio::net::TcpStream::connect(proxy_addr).await.unwrap();
    let test_data = b"test data";

    proxy_stream.write_all(test_data).await.unwrap();
    let mut responce = vec![0u8; test_data.len()];

    proxy_stream.read_exact(&mut responce).await.unwrap();

    assert_eq!(Vec::from(test_data), responce);

    proxy_stream.shutdown().await.unwrap();
    echo_server_handle.abort();

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    println!("Proxy metrics: {:?}", proxy_metrics);

    assert_eq!(
        proxy_metrics
            .total_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        proxy_metrics
            .active_connections
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        proxy_metrics
            .bytes_upstream
            .load(std::sync::atomic::Ordering::Relaxed),
        test_data.len() as u64,
    );
    assert_eq!(
        proxy_metrics
            .bytes_downstream
            .load(std::sync::atomic::Ordering::Relaxed),
        test_data.len() as u64,
    );

    proxy_server_handle.abort();
}
