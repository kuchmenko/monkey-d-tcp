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
    let metrics_rx = proxy_server.metrics();

    let proxy_server_handle = tokio::spawn(async move {
        proxy_server.run().await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut proxy_stream = tokio::net::TcpStream::connect(proxy_addr).await.unwrap();
    let test_data = b"test data";

    proxy_stream.write_all(test_data).await.unwrap();
    let mut response = vec![0u8; test_data.len()];

    proxy_stream.read_exact(&mut response).await.unwrap();

    assert_eq!(Vec::from(test_data), response);

    proxy_stream.shutdown().await.unwrap();
    echo_server_handle.abort();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let snapshot = metrics_rx.borrow();
    println!("Proxy metrics: {:?}", *snapshot);

    assert_eq!(snapshot.total_connections, 1);
    assert_eq!(snapshot.bytes_upstream, test_data.len() as u64);
    assert_eq!(snapshot.bytes_downstream, test_data.len() as u64);

    proxy_server_handle.abort();
}
