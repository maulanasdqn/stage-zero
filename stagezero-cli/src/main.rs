mod commands;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::commands::{Cli, run};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();
    run(Cli::parse()).await
}
