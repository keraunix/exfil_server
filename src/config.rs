#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub port: String,
    pub log_file: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            log_file: std::env::var("LOG_FILE").is_ok(),
        }
    }
}
