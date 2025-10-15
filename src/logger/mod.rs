use anyhow::Context;
use std::{env, fs::OpenOptions, sync::OnceLock};
use tracing_appender::non_blocking;
use tracing_subscriber::{
    Registry, EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

static FILE_GUARD: OnceLock<non_blocking::WorkerGuard> = OnceLock::new();

#[derive(Debug, PartialEq, Eq)]
struct Config {
    log_file: bool,
}

fn build_config() -> anyhow::Result<Config> {
    let config = Config {
        log_file: env::var("LOG_FILE").is_ok(),
    };

    Ok(config)
}

pub fn init_logging() -> anyhow::Result<()> {
    let config = build_config()?;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = fmt::layer()
        .json()
        .flatten_event(false)
        .with_current_span(false)
        .with_span_list(false)
        .with_target(true)
        .with_level(true);

    let file_layer: Option<_> = if config.log_file {
        let path = "app.log";
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("failed to open {}", path))?;

        let (nb, guard) = non_blocking(file);
        let _ = FILE_GUARD.set(guard);

        Some(
            fmt::layer()
                .json()
                .flatten_event(false)
                .with_current_span(false)
                .with_span_list(false)
                .with_target(true)
                .with_level(true)
                .with_writer(nb)
        )
    } else {
        None
    };

    let subscriber = Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer);

    subscriber
        .try_init()
        .map_err(|e| anyhow::anyhow!("failed to initialize subscriber: {e}"))?;

    tracing::info!("logging to file: {:?}", config.log_file);
    Ok(())
}
