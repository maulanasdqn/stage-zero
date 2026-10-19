use axum::Router;
use axum::routing::get;

use crate::handlers::{
    applications_summary, health, hunt_status, index, list_applications, reset_applications,
    start_hunt,
};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route(
            "/applications",
            get(list_applications).delete(reset_applications),
        )
        .route("/applications/summary", get(applications_summary))
        .route("/hunt", get(hunt_status).post(start_hunt))
        .with_state(state)
}
