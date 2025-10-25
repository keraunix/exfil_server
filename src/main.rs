use exfil_server::{config, logger, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let config = config::Config::from_env();
    let _ = dotenvy::dotenv();
    logger::init_logging(&config)?;
    tracing::debug!("config: {:?}", config);
    server::init_server(&config.port).await?;
    Ok(())
}
