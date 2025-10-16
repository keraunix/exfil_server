use anyhow::Context;
use std::{fs::OpenOptions, sync::OnceLock};
use tracing_appender::non_blocking;
use tracing_subscriber::{EnvFilter, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt};

static FILE_GUARD: OnceLock<non_blocking::WorkerGuard> = OnceLock::new();

pub fn init_logging(log_to_file: bool) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = fmt::layer()
        .json()
        .flatten_event(false)
        .with_current_span(false)
        .with_span_list(false)
        .with_target(true)
        .with_level(true);

    let file_layer: Option<_> = if log_to_file {
        let path = "exfil_server.log";
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
                .with_writer(nb),
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

    tracing::info!("logging to file: {:?}", log_file);
    Ok(())
}
