use std::sync::Arc;

use askama::Template;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;
use serde_json::json;
use stagezero_core::DomainError;
use stagezero_core::entities::{
    Application, ApplicationStatus, HuntCriteria, StatusSummary, TeamSizeRange,
};

use crate::state::{AppState, HuntRun, SharedRunProgress, lock_run};

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        let status = match error {
            DomainError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self::new(status, error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct StatusFilter {
    pub status: Option<String>,
}

impl StatusFilter {
    fn parse(&self) -> Result<Option<ApplicationStatus>, ApiError> {
        self.status
            .as_deref()
            .filter(|status| !status.trim().is_empty())
            .map(str::parse)
            .transpose()
            .map_err(ApiError::from)
    }
}

#[derive(Debug, Deserialize)]
pub struct ResetQuery {
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HuntRequest {
    pub batch: String,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    pub dry_run: bool,
}

impl HuntRequest {
    fn into_criteria(self) -> Result<HuntCriteria, ApiError> {
        let team_size = match self.size.as_deref() {
            Some(size) => size.parse::<TeamSizeRange>()?,
            None => TeamSizeRange::default(),
        };
        Ok(HuntCriteria {
            batch: self.batch,
            team_size,
            industry: self.industry.filter(|industry| !industry.trim().is_empty()),
            limit: self.limit,
            dry_run: self.dry_run,
        })
    }
}

struct ApplicationRow {
    applied_at: String,
    status: String,
    status_class: String,
    company_name: String,
    yc_url: String,
    website: String,
    contact_email: String,
}

impl From<&Application> for ApplicationRow {
    fn from(application: &Application) -> Self {
        Self {
            applied_at: application.applied_at.clone(),
            status: application.status.to_string(),
            status_class: application.status.as_str().to_lowercase().replace('_', "-"),
            company_name: application.company_name.clone(),
            yc_url: format!(
                "https://www.ycombinator.com/companies/{}",
                application.yc_slug
            ),
            website: application.website.clone().unwrap_or_default(),
            contact_email: application
                .contact_email
                .clone()
                .unwrap_or_else(|| "-".into()),
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    summary: StatusSummary,
    rows: Vec<ApplicationRow>,
    hunt_running: bool,
    mailer_configured: bool,
    resume_loaded: bool,
}

pub async fn index(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    let applications = state.status.list(None).await?;
    let summary = state.status.summary().await?;
    let template = IndexTemplate {
        summary,
        rows: applications.iter().map(ApplicationRow::from).collect(),
        hunt_running: state.current_run().running,
        mailer_configured: state.mailer_configured,
        resume_loaded: state.resume_loaded,
    };
    template
        .render()
        .map(Html)
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn list_applications(
    State(state): State<AppState>,
    Query(filter): Query<StatusFilter>,
) -> Result<Json<Vec<Application>>, ApiError> {
    Ok(Json(state.status.list(filter.parse()?).await?))
}

pub async fn applications_summary(
    State(state): State<AppState>,
) -> Result<Json<StatusSummary>, ApiError> {
    Ok(Json(state.status.summary().await?))
}

pub async fn reset_applications(
    State(state): State<AppState>,
    Query(query): Query<ResetQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if state.current_run().running {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "a hunt is running, try again when it finishes",
        ));
    }
    let removed = state.reset.execute(query.slug).await?;
    Ok(Json(json!({ "removed": removed })))
}

pub async fn hunt_status(State(state): State<AppState>) -> Json<HuntRun> {
    Json(state.current_run().clone())
}

pub async fn start_hunt(
    State(state): State<AppState>,
    Json(request): Json<HuntRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let criteria = request.into_criteria()?;
    if !criteria.dry_run && !state.mailer_configured {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "Resend API key missing; set RESEND_API_KEY or send dry_run: true",
        ));
    }
    if !criteria.dry_run && !state.resume_loaded {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "resume file not found; fix [applicant].resume_path or send dry_run: true",
        ));
    }
    if !state.try_claim_hunt() {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "a hunt is already running",
        ));
    }

    *state.current_run() = HuntRun {
        running: true,
        criteria: Some(criteria.clone()),
        ..HuntRun::default()
    };

    let background_state = state.clone();
    tokio::spawn(async move {
        let progress = SharedRunProgress {
            run: Arc::clone(&background_state.hunt_run),
        };
        let outcome = background_state.hunt.execute(&criteria, &progress).await;
        {
            let mut run = lock_run(&background_state.hunt_run);
            run.running = false;
            match outcome {
                Ok(report) => run.report = Some(report),
                Err(error) => run.error = Some(error.to_string()),
            }
        }
        background_state.release_hunt();
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "started": true, "poll": "/hunt" })),
    ))
}
