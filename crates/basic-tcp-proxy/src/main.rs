use basic_tcp_proxy::run_proxy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src_ip = "127.0.0.1:8080";
    let output_ip = "127.0.0.1:8081";

    run_proxy(src_ip, output_ip).await?;

    Ok(())
}
