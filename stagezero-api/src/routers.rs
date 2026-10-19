use axum::Router;
use axum::routing::{get, post};

use crate::handlers::{
    applications_summary, explore_startups, health, hunt_status, index, inspect_startup,
    list_applications, list_batches, reset_applications, send_to_startup, settings, skip_startup,
    start_hunt,
};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
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
        .with_state(state)
}
