use basic_tcp_proxy::Proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_ip = "127.0.0.1:8080";
    let output_ip = "127.0.0.1:8081";

    let (mut proxy, _) = Proxy::bind(src_ip, output_ip).await?;
    proxy.run().await?;

    Ok(())
}
