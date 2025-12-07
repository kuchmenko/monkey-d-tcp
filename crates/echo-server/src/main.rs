use std::sync::Arc;

use tokio::select;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:8080";
    let (server, addr) = echo_server::EchoServer::bind(addr).await?;
    let server = Arc::new(server);
    let main_server = server.clone();
    println!("Listening on {}", addr);
    select! {
        _ = main_server.run() => {}
        _ = tokio::signal::ctrl_c() => {
            println!("Stopped echo server");
            server.shutdown();
        }
    }
    Ok(())
}
