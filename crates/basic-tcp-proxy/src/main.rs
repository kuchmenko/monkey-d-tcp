use basic_tcp_proxy::{Config, Proxy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("proxy.toml").unwrap_or_default();

    let (mut proxy, _) = Proxy::new(config).await?;
    proxy.run().await?;

    Ok(())
}
