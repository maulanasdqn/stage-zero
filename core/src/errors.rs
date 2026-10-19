use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("startup directory error: {0}")]
    Directory(String),
    #[error("scraping error: {0}")]
    Scraping(String),
    #[error("repository error: {0}")]
    Repository(String),
    #[error("mailer error: {0}")]
    Mailer(String),
}
