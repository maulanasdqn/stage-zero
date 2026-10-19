use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use stagezero_core::entities::{Applicant, Attachment, EmailTemplate};

use crate::errors::InfraError;

pub const DEFAULT_CONFIG_PATH: &str = "stagezero.toml";
pub const EXAMPLE_CONFIG: &str = include_str!("../../stagezero.example.toml");
pub const RESEND_API_KEY_ENV: &str = "RESEND_API_KEY";

#[derive(Debug, Clone, Deserialize)]
pub struct StageZeroConfig {
    pub resend: ResendConfig,
    pub applicant: ApplicantConfig,
    pub template: EmailTemplate,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub outreach: OutreachConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResendConfig {
    pub from: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicantConfig {
    pub name: String,
    pub email: String,
    pub resume_path: PathBuf,
    #[serde(default)]
    pub portfolio: Option<String>,
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

    pub fn resend_api_key(&self) -> Option<String> {
        std::env::var(RESEND_API_KEY_ENV)
            .ok()
            .or_else(|| self.resend.api_key.clone())
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty())
    }

    pub fn applicant(&self) -> Applicant {
        Applicant {
            name: self.applicant.name.clone(),
            email: self.applicant.email.clone(),
            portfolio: self.applicant.portfolio.clone(),
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
