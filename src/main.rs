use anyhow::Result;
use haxmail::smtp;
use std::env;
use tokio::net::TcpListener;
#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:2525".to_string());
    let domain = &env::args()
        .nth(2)
        .unwrap_or_else(|| "smtp.haxmail.buzz".to_string());

    tracing::info!("Haxmail server started for {domain}");
    let listener = TcpListener::bind(&addr).await?;

    loop {
        let (stream, addr) = listener.accept().await?;
        tracing::info!("Accepted connection from {}", addr);

        tokio::task::LocalSet::new()
            .run_until(async move {
                let smtp = smtp::Server::new(domain, stream).await?;
                smtp.serve().await
            })
            .await
            .ok();
    }
}
