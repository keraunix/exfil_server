use exfil_server::{logger, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    logger::init_logging()?;
    server::init_server().await?;
    Ok(())
}
