use reqwest::Client;
use reqwest::header::USER_AGENT;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;
use stagezero_core::DomainError;
use stagezero_core::entities::{BatchInfo, Founder, HuntCriteria, Startup};
use stagezero_core::traits::{FounderSource, StartupDirectory};

use crate::http::random_user_agent;

pub const YC_OSS_BASE_URL: &str = "https://yc-oss.github.io/api";
pub const YC_COMPANIES_BASE_URL: &str = "https://www.ycombinator.com/companies";

#[derive(Debug, Deserialize)]
struct YcOssCompany {
    slug: String,
    name: String,
    #[serde(default)]
    website: Option<String>,
    #[serde(default)]
    batch: String,
    #[serde(default)]
    team_size: Option<i64>,
    #[serde(default)]
    industry: Option<String>,
    #[serde(default)]
    subindustry: Option<String>,
    #[serde(default)]
    industries: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    one_liner: Option<String>,
    #[serde(default)]
    long_description: Option<String>,
}

impl YcOssCompany {
    fn known_team_size(&self) -> Option<u32> {
        self.team_size
            .filter(|size| *size > 0)
            .and_then(|size| u32::try_from(size).ok())
    }

    fn is_active(&self) -> bool {
        self.status
            .as_deref()
            .is_none_or(|status| status.eq_ignore_ascii_case("active"))
    }

    fn matches_industry(&self, industry: &str) -> bool {
        let haystack = self
            .industry
            .iter()
            .chain(self.subindustry.iter())
            .chain(self.industries.iter())
            .chain(self.tags.iter())
            .map(|value| value.to_lowercase())
            .collect::<Vec<_>>()
            .join(" | ");
        industry
            .split_whitespace()
            .map(str::to_lowercase)
            .all(|token| haystack.contains(&token))
    }

    fn into_startup(self) -> Startup {
        let team_size = self.known_team_size();
        let mut industries = self.industries;
        if industries.is_empty() {
            industries.extend(self.industry);
        }
        Startup {
            slug: self.slug,
            name: self.name,
            website: self.website.filter(|website| !website.trim().is_empty()),
            batch: self.batch,
            team_size,
            industries,
            one_liner: self.one_liner.unwrap_or_default(),
            tags: self.tags,
            description: self.long_description.unwrap_or_default().trim().to_string(),
        }
    }
}

pub fn batch_slug(batch: &str) -> Result<String, DomainError> {
    let normalized = batch.trim().to_lowercase().replace([' ', '_'], "-");
    let invalid = || {
        DomainError::InvalidInput(format!(
            "unrecognized batch `{batch}`; use codes like W24, S24, F24, X25 or names like winter-2024"
        ))
    };

    let season_of = |prefix: &str| match prefix {
        "w" | "winter" => Some("winter"),
        "s" | "summer" => Some("summer"),
        "f" | "fall" => Some("fall"),
        "x" | "sp" | "spring" => Some("spring"),
        _ => None,
    };

    if let Some((season, year)) = normalized.split_once('-') {
        let season = season_of(season).ok_or_else(invalid)?;
        let year: u32 = year.parse().map_err(|_| invalid())?;
        let year = if year < 100 { 2000 + year } else { year };
        return Ok(format!("{season}-{year}"));
    }

    let digits_start = normalized
        .find(|character: char| character.is_ascii_digit())
        .ok_or_else(invalid)?;
    let (prefix, digits) = normalized.split_at(digits_start);
    let season = season_of(prefix).ok_or_else(invalid)?;
    let year: u32 = digits.parse().map_err(|_| invalid())?;
    let year = match digits.len() {
        2 => 2000 + year,
        4 => year,
        _ => return Err(invalid()),
    };
    Ok(format!("{season}-{year}"))
}

pub fn filter_companies(
    raw_json: &str,
    criteria: &HuntCriteria,
) -> Result<Vec<Startup>, DomainError> {
    let entries: Vec<Value> = serde_json::from_str(raw_json).map_err(|error| {
        DomainError::Directory(format!("unexpected YC directory payload: {error}"))
    })?;
    let companies = entries
        .into_iter()
        .filter_map(
            |entry| match serde_json::from_value::<YcOssCompany>(entry) {
                Ok(company) => Some(company),
                Err(error) => {
                    tracing::warn!("skipping malformed YC directory entry: {error}");
                    None
                }
            },
        )
        .collect::<Vec<_>>();
    Ok(companies
        .into_iter()
        .filter(YcOssCompany::is_active)
        .filter(|company| {
            company
                .known_team_size()
                .is_some_and(|size| criteria.team_size.contains(size))
        })
        .filter(|company| {
            criteria
                .industry
                .as_deref()
                .filter(|industry| !industry.trim().is_empty())
                .is_none_or(|industry| company.matches_industry(industry))
        })
        .map(YcOssCompany::into_startup)
        .collect())
}

#[derive(Clone)]
pub struct YcOssDirectory {
    client: Client,
    base_url: String,
}

impl YcOssDirectory {
    pub fn new(client: Client) -> Self {
        Self::with_base_url(client, YC_OSS_BASE_URL)
    }

    pub fn with_base_url(client: Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct YcOssMeta {
    #[serde(default)]
    batches: std::collections::BTreeMap<String, YcOssBatch>,
}

#[derive(Debug, Deserialize)]
struct YcOssBatch {
    name: String,
    #[serde(default)]
    count: usize,
}

fn batch_recency(slug: &str) -> (u32, u32) {
    let Some((season, year)) = slug.split_once('-') else {
        return (0, 0);
    };
    let season_rank = match season {
        "winter" => 1,
        "spring" => 2,
        "summer" => 3,
        "fall" => 4,
        _ => 0,
    };
    (year.parse().unwrap_or_default(), season_rank)
}

pub fn parse_batches(raw_json: &str) -> Result<Vec<BatchInfo>, DomainError> {
    let meta: YcOssMeta = serde_json::from_str(raw_json)
        .map_err(|error| DomainError::Directory(format!("unexpected YC meta payload: {error}")))?;
    let mut batches = meta
        .batches
        .into_iter()
        .filter(|(slug, _)| batch_recency(slug).0 > 0)
        .map(|(slug, batch)| BatchInfo {
            slug,
            name: batch.name,
            count: batch.count,
        })
        .collect::<Vec<_>>();
    batches.sort_by_key(|batch| std::cmp::Reverse(batch_recency(&batch.slug)));
    Ok(batches)
}

impl YcOssDirectory {
    async fn get_text(&self, url: &str) -> Result<String, DomainError> {
        let response = self
            .client
            .get(url)
            .header(USER_AGENT, random_user_agent())
            .send()
            .await
            .map_err(|error| DomainError::Directory(format!("cannot reach {url}: {error}")))?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(DomainError::Directory(format!("{url} not found")));
        }
        response
            .error_for_status()
            .map_err(|error| DomainError::Directory(error.to_string()))?
            .text()
            .await
            .map_err(|error| DomainError::Directory(error.to_string()))
    }
}

impl StartupDirectory for YcOssDirectory {
    async fn batches(&self) -> Result<Vec<BatchInfo>, DomainError> {
        let body = self
            .get_text(&format!("{}/meta.json", self.base_url))
            .await?;
        parse_batches(&body)
    }

    async fn fetch(&self, criteria: &HuntCriteria) -> Result<Vec<Startup>, DomainError> {
        let slug = batch_slug(&criteria.batch)?;
        let body = self
            .get_text(&format!("{}/batches/{slug}.json", self.base_url))
            .await
            .map_err(|error| match error {
                DomainError::Directory(message) if message.ends_with("not found") => {
                    DomainError::Directory(format!("batch `{}` not found", criteria.batch))
                }
                other => other,
            })?;
        filter_companies(&body, criteria)
    }
}

pub fn extract_founders(html: &str) -> Result<Vec<Founder>, DomainError> {
    let document = Html::parse_document(html);
    let selector =
        Selector::parse("[data-page]").map_err(|error| DomainError::Scraping(error.to_string()))?;
    let payload = document
        .select(&selector)
        .find_map(|element| element.value().attr("data-page"))
        .ok_or_else(|| DomainError::Scraping("YC company page has no data-page payload".into()))?;
    let page: Value = serde_json::from_str(payload)
        .map_err(|error| DomainError::Scraping(format!("invalid data-page json: {error}")))?;
    let founders = page
        .pointer("/props/company/founders")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    Ok(founders
        .iter()
        .filter(|founder| {
            founder
                .get("is_active")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        })
        .filter_map(|founder| {
            let full_name = founder.get("full_name")?.as_str()?.trim().to_string();
            let title = founder
                .get("title")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .map(str::to_string);
            (!full_name.is_empty()).then_some(Founder { full_name, title })
        })
        .collect())
}

#[derive(Clone)]
pub struct YcFounderSource {
    client: Client,
    base_url: String,
}

impl YcFounderSource {
    pub fn new(client: Client) -> Self {
        Self::with_base_url(client, YC_COMPANIES_BASE_URL)
    }

    pub fn with_base_url(client: Client, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }
}

impl FounderSource for YcFounderSource {
    async fn founders(&self, startup: &Startup) -> Result<Vec<Founder>, DomainError> {
        let url = format!("{}/{}", self.base_url, startup.slug);
        let html = self
            .client
            .get(&url)
            .header(USER_AGENT, random_user_agent())
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|error| DomainError::Scraping(format!("cannot load {url}: {error}")))?
            .text()
            .await
            .map_err(|error| DomainError::Scraping(error.to_string()))?;
        extract_founders(&html)
    }
}
