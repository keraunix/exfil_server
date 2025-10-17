#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub port: String,
    pub log_to_file: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            log_to_file: std::env::var("LOG_TO_FILE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
        }
    }
}
