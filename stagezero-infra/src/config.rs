use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use stagezero_core::entities::{Applicant, Attachment, DEFAULT_MAX_HIGHLIGHTS, EmailTemplate};
use stagezero_core::personalization::Highlight;

use crate::errors::InfraError;

pub const DEFAULT_CONFIG_PATH: &str = "stagezero.toml";
pub const EXAMPLE_CONFIG: &str = include_str!("../../stagezero.example.toml");
pub const RESEND_API_KEY_ENV: &str = "RESEND_API_KEY";
pub const CLOUDFLARE_API_TOKEN_ENV: &str = "CLOUDFLARE_API_TOKEN";
pub const CLOUDFLARE_ACCOUNT_ID_ENV: &str = "CLOUDFLARE_ACCOUNT_ID";

#[derive(Debug, Clone, Deserialize)]
pub struct StageZeroConfig {
    pub email: EmailConfig,
    pub applicant: ApplicantConfig,
    pub template: EmailTemplate,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub outreach: OutreachConfig,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailProvider {
    #[default]
    Cloudflare,
    Resend,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    #[serde(default)]
    pub provider: EmailProvider,
    pub from: String,
    #[serde(default)]
    pub from_name: Option<String>,
    #[serde(default)]
    pub cloudflare_account_id: Option<String>,
    #[serde(default)]
    pub api_token: Option<String>,
}

fn non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

impl EmailConfig {
    pub fn api_token(&self) -> Option<String> {
        let variable = match self.provider {
            EmailProvider::Cloudflare => CLOUDFLARE_API_TOKEN_ENV,
            EmailProvider::Resend => RESEND_API_KEY_ENV,
        };
        non_empty(std::env::var(variable).ok()).or_else(|| non_empty(self.api_token.clone()))
    }

    pub fn cloudflare_account_id(&self) -> Option<String> {
        non_empty(std::env::var(CLOUDFLARE_ACCOUNT_ID_ENV).ok())
            .or_else(|| non_empty(self.cloudflare_account_id.clone()))
    }

    pub fn formatted_sender(&self) -> String {
        match non_empty(self.from_name.clone()) {
            Some(name) => format!("{name} <{}>", self.from),
            None => self.from.clone(),
        }
    }

    pub fn token_variable(&self) -> &'static str {
        match self.provider {
            EmailProvider::Cloudflare => CLOUDFLARE_API_TOKEN_ENV,
            EmailProvider::Resend => RESEND_API_KEY_ENV,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicantConfig {
    pub name: String,
    pub email: String,
    pub resume_path: PathBuf,
    #[serde(default)]
    pub portfolio: Option<String>,
    #[serde(default = "default_max_highlights")]
    pub max_highlights: usize,
    #[serde(default)]
    pub highlights: Vec<Highlight>,
}

fn default_max_highlights() -> usize {
    DEFAULT_MAX_HIGHLIGHTS
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_database_path")]
    pub database_path: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: default_database_path(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutreachConfig {
    #[serde(default = "default_min_delay_secs")]
    pub min_delay_secs: u64,
    #[serde(default = "default_max_delay_secs")]
    pub max_delay_secs: u64,
    #[serde(default = "default_daily_limit")]
    pub daily_limit: usize,
}

impl Default for OutreachConfig {
    fn default() -> Self {
        Self {
            min_delay_secs: default_min_delay_secs(),
            max_delay_secs: default_max_delay_secs(),
            daily_limit: default_daily_limit(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct StorageOnlyConfig {
    #[serde(default)]
    storage: StorageConfig,
}

fn default_database_path() -> PathBuf {
    PathBuf::from("stagezero.db")
}

fn default_min_delay_secs() -> u64 {
    2
}

fn default_max_delay_secs() -> u64 {
    5
}

fn default_daily_limit() -> usize {
    20
}

impl StageZeroConfig {
    pub fn load(path: &Path) -> Result<Self, InfraError> {
        let raw = std::fs::read_to_string(path).map_err(|error| {
            InfraError::Config(format!(
                "cannot read `{}` ({error}); run `stagezero init` to create one",
                path.display()
            ))
        })?;
        Self::parse(&raw)
    }

    pub fn parse(raw: &str) -> Result<Self, InfraError> {
        Ok(toml::from_str(raw)?)
    }

    pub fn applicant(&self) -> Applicant {
        Applicant {
            name: self.applicant.name.clone(),
            email: self.applicant.email.clone(),
            portfolio: self.applicant.portfolio.clone(),
            highlights: self.applicant.highlights.clone(),
            max_highlights: self.applicant.max_highlights,
        }
    }

    pub fn load_resume(&self) -> Result<Option<Attachment>, InfraError> {
        let path = &self.applicant.resume_path;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read(path)?;
        let filename = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "resume.pdf".to_string());
        Ok(Some(Attachment {
            filename,
            content: Arc::from(content),
        }))
    }
}

pub fn load_storage_config(path: &Path) -> Result<StorageConfig, InfraError> {
    if !path.exists() {
        return Ok(StorageConfig::default());
    }
    let raw = std::fs::read_to_string(path)?;
    let parsed: StorageOnlyConfig = toml::from_str(&raw)?;
    Ok(parsed.storage)
}
