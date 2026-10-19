use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::errors::DomainError;
use crate::personalization::{Highlight, HighlightMatch, render_bullets, select_highlights};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Startup {
    pub slug: String,
    pub name: String,
    pub website: Option<String>,
    pub batch: String,
    pub team_size: Option<u32>,
    pub industries: Vec<String>,
    pub one_liner: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: String,
}

impl Startup {
    pub fn primary_industry(&self) -> Option<&str> {
        self.industries.first().map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Founder {
    pub full_name: String,
    pub title: Option<String>,
}

impl Founder {
    pub fn first_name(&self) -> &str {
        self.full_name.split_whitespace().next().unwrap_or_default()
    }

    pub fn last_name(&self) -> Option<&str> {
        let mut parts = self.full_name.split_whitespace();
        parts.next()?;
        parts.last()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContactOrigin {
    Scraped,
    Guessed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contact {
    pub email: String,
    pub founder: Option<Founder>,
    pub origin: ContactOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApplicationStatus {
    Applied,
    Skipped,
    FailedNoContact,
}

impl ApplicationStatus {
    pub const ALL: [ApplicationStatus; 3] = [
        ApplicationStatus::Applied,
        ApplicationStatus::Skipped,
        ApplicationStatus::FailedNoContact,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ApplicationStatus::Applied => "APPLIED",
            ApplicationStatus::Skipped => "SKIPPED",
            ApplicationStatus::FailedNoContact => "FAILED_NO_CONTACT",
        }
    }
}

impl fmt::Display for ApplicationStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ApplicationStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        ApplicationStatus::ALL
            .into_iter()
            .find(|status| status.as_str().eq_ignore_ascii_case(value.trim()))
            .ok_or_else(|| {
                DomainError::InvalidInput(format!("unknown application status `{value}`"))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Application {
    pub id: i64,
    pub yc_slug: String,
    pub company_name: String,
    pub website: Option<String>,
    pub contact_email: Option<String>,
    pub status: ApplicationStatus,
    pub applied_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewApplication {
    pub yc_slug: String,
    pub company_name: String,
    pub website: Option<String>,
    pub contact_email: Option<String>,
    pub status: ApplicationStatus,
}

impl NewApplication {
    pub fn for_startup(
        startup: &Startup,
        contact_email: Option<String>,
        status: ApplicationStatus,
    ) -> Self {
        Self {
            yc_slug: startup.slug.clone(),
            company_name: startup.name.clone(),
            website: startup.website.clone(),
            contact_email,
            status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamSizeRange {
    pub min: u32,
    pub max: u32,
}

impl TeamSizeRange {
    pub fn contains(&self, size: u32) -> bool {
        (self.min..=self.max).contains(&size)
    }
}

impl Default for TeamSizeRange {
    fn default() -> Self {
        Self { min: 1, max: 10 }
    }
}

impl fmt::Display for TeamSizeRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{}", self.min, self.max)
    }
}

impl FromStr for TeamSizeRange {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let invalid =
            || DomainError::InvalidInput(format!("team size must look like `1-10`, got `{value}`"));
        let (min, max) = match value.trim().split_once('-') {
            Some((min, max)) => (min.trim(), max.trim()),
            None => (value.trim(), value.trim()),
        };
        let min: u32 = min.parse().map_err(|_| invalid())?;
        let max: u32 = max.parse().map_err(|_| invalid())?;
        if min > max {
            return Err(invalid());
        }
        Ok(Self { min, max })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntCriteria {
    pub batch: String,
    pub team_size: TeamSizeRange,
    pub industry: Option<String>,
    pub limit: Option<usize>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Applicant {
    pub name: String,
    pub email: String,
    pub portfolio: Option<String>,
    #[serde(default)]
    pub highlights: Vec<Highlight>,
    #[serde(default = "default_max_highlights")]
    pub max_highlights: usize,
}

pub const DEFAULT_MAX_HIGHLIGHTS: usize = 3;

fn default_max_highlights() -> usize {
    DEFAULT_MAX_HIGHLIGHTS
}

impl Applicant {
    pub fn highlights_for(&self, startup: &Startup) -> Vec<HighlightMatch> {
        select_highlights(&self.highlights, startup, self.max_highlights)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub filename: String,
    pub content: Arc<[u8]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingEmail {
    pub to: String,
    pub reply_to: Option<String>,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailPreview {
    pub to: String,
    pub subject: String,
    pub body: String,
}

impl From<&OutgoingEmail> for EmailPreview {
    fn from(email: &OutgoingEmail) -> Self {
        Self {
            to: email.to.clone(),
            subject: email.subject.clone(),
            body: email.body.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateContext<'a> {
    pub startup: &'a Startup,
    pub founder: Option<&'a Founder>,
    pub applicant: &'a Applicant,
}

impl TemplateContext<'_> {
    fn one_liner_clause(&self) -> String {
        let one_liner = self
            .startup
            .one_liner
            .trim()
            .trim_end_matches(['.', '!'])
            .trim();
        match one_liner.is_empty() {
            true => String::new(),
            false => format!(" (\"{one_liner}\")"),
        }
    }

    fn variables(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "founder_first_name",
                self.founder
                    .map(Founder::first_name)
                    .filter(|name| !name.is_empty())
                    .unwrap_or("there")
                    .to_string(),
            ),
            (
                "founder_name",
                self.founder
                    .map(|founder| founder.full_name.clone())
                    .unwrap_or_else(|| format!("{} team", self.startup.name)),
            ),
            ("company_name", self.startup.name.clone()),
            (
                "industry",
                self.startup
                    .primary_industry()
                    .unwrap_or("your space")
                    .to_string(),
            ),
            ("one_liner", self.startup.one_liner.trim().to_string()),
            ("one_liner_clause", self.one_liner_clause()),
            (
                "highlights",
                render_bullets(&self.applicant.highlights_for(self.startup)),
            ),
            ("batch", self.startup.batch.clone()),
            ("applicant_name", self.applicant.name.clone()),
            (
                "applicant_first_name",
                self.applicant
                    .name
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
            ),
            ("applicant_email", self.applicant.email.clone()),
            (
                "portfolio",
                self.applicant.portfolio.clone().unwrap_or_default(),
            ),
        ]
    }
}

impl EmailTemplate {
    pub fn render(&self, context: &TemplateContext<'_>) -> (String, String) {
        let variables = context.variables();
        let substitute = |text: &str| {
            variables
                .iter()
                .fold(text.to_string(), |rendered, (key, value)| {
                    rendered
                        .replace(&format!("{{{{{key}}}}}"), value)
                        .replace(&format!("{{{{ {key} }}}}"), value)
                })
        };
        (substitute(&self.subject), substitute(&self.body))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum HuntOutcome {
    AlreadyProcessed,
    SkippedNoWebsite,
    NoContact,
    Previewed,
    Applied,
    SendFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntEvent {
    pub slug: String,
    pub company_name: String,
    pub outcome: HuntOutcome,
    pub contact: Option<Contact>,
    pub email: Option<EmailPreview>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntReport {
    pub discovered: usize,
    pub already_processed: usize,
    pub skipped_no_website: usize,
    pub no_contact: usize,
    pub previewed: usize,
    pub applied: usize,
    pub send_failed: usize,
    pub daily_limit_reached: bool,
}

impl HuntReport {
    pub fn register(&mut self, outcome: &HuntOutcome) {
        match outcome {
            HuntOutcome::AlreadyProcessed => self.already_processed += 1,
            HuntOutcome::SkippedNoWebsite => self.skipped_no_website += 1,
            HuntOutcome::NoContact => self.no_contact += 1,
            HuntOutcome::Previewed => self.previewed += 1,
            HuntOutcome::Applied => self.applied += 1,
            HuntOutcome::SendFailed(_) => self.send_failed += 1,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusSummary {
    pub total: usize,
    pub applied: usize,
    pub skipped: usize,
    pub failed_no_contact: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchInfo {
    pub slug: String,
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub startup: Startup,
    pub status: Option<ApplicationStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupInsight {
    pub startup: Startup,
    pub highlights: Vec<HighlightMatch>,
    pub founders: Vec<Founder>,
    pub contact: Option<Contact>,
    pub email: EmailPreview,
    pub status: Option<ApplicationStatus>,
}
