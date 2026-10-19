use std::sync::Arc;

use askama::Template;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use stagezero_core::DomainError;
use stagezero_core::entities::{
    Application, ApplicationStatus, BatchInfo, DirectoryEntry, EmailPreview, HuntCriteria,
    HuntOutcome, Startup, StartupInsight, StatusSummary, TeamSizeRange,
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
            DomainError::Directory(_) | DomainError::Scraping(_) | DomainError::Mailer(_) => {
                StatusCode::BAD_GATEWAY
            }
            DomainError::Repository(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self::new(status, error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

fn parse_team_size(size: Option<&str>) -> Result<TeamSizeRange, ApiError> {
    match size.map(str::trim).filter(|size| !size.is_empty()) {
        Some(size) => Ok(size.parse::<TeamSizeRange>()?),
        None => Ok(TeamSizeRange::default()),
    }
}

fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
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
pub struct ExploreQuery {
    pub batch: String,
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StartupRequest {
    pub startup: Startup,
}

#[derive(Debug, Deserialize)]
pub struct SendRequest {
    pub startup: Startup,
    pub email: EmailPreview,
}

#[derive(Debug, Serialize)]
pub struct SendResponse {
    pub outcome: HuntOutcome,
}

#[derive(Debug, Serialize)]
pub struct Settings {
    pub mailer_provider: &'static str,
    pub mailer_configured: bool,
    pub sender: String,
    pub resume_loaded: bool,
    pub hunt_running: bool,
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
        Ok(HuntCriteria {
            team_size: parse_team_size(self.size.as_deref())?,
            batch: self.batch,
            industry: non_blank(self.industry),
            limit: self.limit,
            dry_run: self.dry_run,
        })
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    mailer_provider: &'static str,
    mailer_configured: bool,
    sender: String,
    resume_loaded: bool,
}

fn settings_of(state: &AppState) -> Settings {
    Settings {
        mailer_provider: state.mailer_provider,
        mailer_configured: state.mailer_configured,
        sender: state.sender.clone(),
        resume_loaded: state.resume_loaded,
        hunt_running: state.current_run().running,
    }
}

fn ensure_can_send(state: &AppState) -> Result<(), ApiError> {
    if !state.mailer_configured {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            format!(
                "{} is not configured; check the [email] section and API token",
                state.mailer_provider
            ),
        ));
    }
    if !state.resume_loaded {
        return Err(ApiError::new(
            StatusCode::BAD_REQUEST,
            "resume file not found; fix [applicant].resume_path",
        ));
    }
    Ok(())
}

pub async fn index(State(state): State<AppState>) -> Result<Html<String>, ApiError> {
    IndexTemplate {
        mailer_provider: state.mailer_provider,
        mailer_configured: state.mailer_configured,
        sender: state.sender.clone(),
        resume_loaded: state.resume_loaded,
    }
    .render()
    .map(Html)
    .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
}

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn settings(State(state): State<AppState>) -> Json<Settings> {
    Json(settings_of(&state))
}

pub async fn list_batches(State(state): State<AppState>) -> Result<Json<Vec<BatchInfo>>, ApiError> {
    Ok(Json(state.hunt.batches().await?))
}

pub async fn explore_startups(
    State(state): State<AppState>,
    Query(query): Query<ExploreQuery>,
) -> Result<Json<Vec<DirectoryEntry>>, ApiError> {
    let criteria = HuntCriteria {
        team_size: parse_team_size(query.size.as_deref())?,
        batch: query.batch,
        industry: non_blank(query.industry),
        limit: None,
        dry_run: true,
    };
    Ok(Json(state.hunt.explore(&criteria).await?))
}

pub async fn inspect_startup(
    State(state): State<AppState>,
    Json(request): Json<StartupRequest>,
) -> Result<Json<StartupInsight>, ApiError> {
    Ok(Json(state.hunt.inspect(&request.startup).await?))
}

pub async fn send_to_startup(
    State(state): State<AppState>,
    Json(request): Json<SendRequest>,
) -> Result<Json<SendResponse>, ApiError> {
    ensure_can_send(&state)?;
    let outcome = state
        .hunt
        .send_reviewed(&request.startup, request.email)
        .await?;
    Ok(Json(SendResponse { outcome }))
}

pub async fn skip_startup(
    State(state): State<AppState>,
    Json(request): Json<StartupRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    state.hunt.skip(&request.startup).await?;
    Ok(Json(json!({ "status": ApplicationStatus::Skipped })))
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
    let removed = state.reset.execute(non_blank(query.slug)).await?;
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
    if !criteria.dry_run {
        ensure_can_send(&state)?;
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
