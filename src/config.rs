use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_address: String,
    pub port: u16,
    pub base_url: String,
    pub static_dir: String,
    pub template_dir: String,
    pub max_upload_size: usize,
    pub bugzilla_url: String,
    pub email: EmailConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL"))?,
            bind_address: env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            base_url: env::var("BASE_URL").map_err(|_| ConfigError::MissingEnvVar("BASE_URL"))?,
            static_dir: env::var("STATIC_DIR").unwrap_or_else(|_| "./static".to_string()),
            template_dir: env::var("TEMPLATE_DIR").unwrap_or_else(|_| "./templates".to_string()),
            max_upload_size: env::var("MAX_UPLOAD_SIZE")
                .unwrap_or_else(|_| "10485760".to_string()) // 10MB default
                .parse()
                .map_err(|_| ConfigError::InvalidUploadSize)?,
            bugzilla_url: env::var("BUGZILLA_URL")
                .unwrap_or_else(|_| "https://bugzilla.yoctoproject.org".to_string()),
            email: EmailConfig {
                host: env::var("EMAIL_HOST").unwrap_or_else(|_| "localhost".to_string()),
                port: env::var("EMAIL_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .map_err(|_| ConfigError::InvalidEmailPort)?,
                username: env::var("EMAIL_USERNAME").ok(),
                password: env::var("EMAIL_PASSWORD").ok(),
                from_address: env::var("EMAIL_FROM")
                    .map_err(|_| ConfigError::MissingEnvVar("EMAIL_FROM"))?,
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(&'static str),
    #[error("Invalid port number")]
    InvalidPort,
    #[error("Invalid email port number")]
    InvalidEmailPort,
    #[error("Invalid upload size")]
    InvalidUploadSize,
}
