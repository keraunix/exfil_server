#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub log_level: String,
    pub log_path: String,
    pub log_to_file: bool,
    pub port: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            log_path: std::env::var("LOG_PATH").unwrap_or_else(|_| "exfil_server.log".to_string()),
            log_to_file: std::env::var("LOG_TO_FILE").is_ok(),
            port: std::env::var("PORT").unwrap_or_else(|_| "8080".to_string()),
        }
    }

    #[cfg(test)]
    pub fn test_default() -> Self {
        Self {
            log_level: "debug".to_string(),
            log_path: "test.log".to_string(),
            log_to_file: true,
            port: "7878".to_string(),
        }
    }
}
