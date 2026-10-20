use std::path::Path;

use axum::Router;
use axum::routing::{get, post};
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers::{
    applications_summary, explore_startups, health, hunt_status, inspect_startup,
    list_applications, list_batches, reset_applications, send_to_startup, settings, skip_startup,
    start_hunt,
};
use crate::state::AppState;

fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/settings", get(settings))
        .route("/batches", get(list_batches))
        .route("/startups", get(explore_startups))
        .route("/startups/inspect", post(inspect_startup))
        .route("/startups/send", post(send_to_startup))
        .route("/startups/skip", post(skip_startup))
        .route(
            "/applications",
            get(list_applications).delete(reset_applications),
        )
        .route("/applications/summary", get(applications_summary))
        .route("/hunt", get(hunt_status).post(start_hunt))
}

pub fn build_router(state: AppState, web_dir: &Path) -> Router {
    let spa = ServeDir::new(web_dir).fallback(ServeFile::new(web_dir.join("index.html")));
    Router::new()
        .nest("/api", api_router())
        .fallback_service(spa)
        .with_state(state)
}
