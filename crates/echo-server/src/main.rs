use tokio::select;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8081";
    let (server, _) = echo_server::EchoServer::bind(addr).await?;

    println!("Listening on {}", server.get_addr()?);
    select! {
        _ = server.run() => {}
        _ = tokio::signal::ctrl_c() => {
            println!("Stopped echo server");
            server.shutdown();
        }
    }
    Ok(())
}
