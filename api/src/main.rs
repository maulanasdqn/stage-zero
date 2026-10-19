mod handlers;
mod routers;
mod state;

use std::path::PathBuf;

use anyhow::Context;
use stagezero_infra::config::{DEFAULT_CONFIG_PATH, StageZeroConfig};
use stagezero_infra::container::build_live_services;
use tracing_subscriber::EnvFilter;

use crate::routers::build_router;
use crate::state::AppState;

const CONFIG_PATH_ENV: &str = "STAGEZERO_CONFIG";
const LISTEN_ADDRESS_ENV: &str = "STAGEZERO_ADDR";
const DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config_path = std::env::var(CONFIG_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_CONFIG_PATH));
    let config = StageZeroConfig::load(&config_path)?;
    let services = build_live_services(&config)?;
    let router = build_router(AppState::new(services));

    let address =
        std::env::var(LISTEN_ADDRESS_ENV).unwrap_or_else(|_| DEFAULT_LISTEN_ADDRESS.to_string());
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .with_context(|| format!("cannot bind {address}"))?;
    tracing::info!("StageZero API listening on http://{address}");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
