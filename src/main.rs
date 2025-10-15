mod config;
use exfil_server::{logger, server};
use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();
    let _ = dotenvy::dotenv();
    logger::init_logging(config.log_file)?;
    server::init_server(&config.port).await?;
    Ok(())
}
