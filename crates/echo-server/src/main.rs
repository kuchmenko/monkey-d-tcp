use echo_server::{Config, EchoServer};
use tokio::select;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("echo.toml").unwrap_or_default();
    let (server, _) = EchoServer::bind(&config.listen_addr).await?;

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
