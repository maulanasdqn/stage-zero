use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfraError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("failed to parse configuration: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}
