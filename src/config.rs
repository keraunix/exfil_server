#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub port: String,
    pub log_to_file: bool,
    pub log_path: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
            log_to_file: std::env::var("LOG_TO_FILE").is_ok(),
            log_path: std::env::var("LOG_PATH").unwrap_or_else(|_| "exfil_server.log".to_string()),
        }
    }
}
