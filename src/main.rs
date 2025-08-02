mod cli;
mod config;
mod error;
mod handler;
mod rewriter;

use std::net::SocketAddr;

use axum::{
    Router,
    routing::{any, get},
};
use clap::Parser;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{Level, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    cli::Args,
    config::Config,
    handler::{AppState, proxy_handler},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = args.log_level.parse::<Level>().unwrap_or(Level::INFO);

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(log_level.to_string())),
        )
        .init();

    info!("Starting notion-site-proxy server");

    // Load configuration
    let mut config = load_config(&args.config)?;

    // Override config with CLI arguments
    if let Some(host) = args.host {
        config.server.host = host;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }

    info!("Loaded configuration for {} domains", config.domains.len());
    for domain in config.domains.keys() {
        info!("  - {}", domain);
    }

    // Create application state
    let state = AppState::new(config.clone());

    // Create router
    let app = Router::new()
        .route("/health", get(health_check))
        .fallback(any(proxy_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

fn load_config(config_path: &str) -> anyhow::Result<Config> {
    use std::fs;

    if fs::metadata(config_path).is_ok() {
        let config_content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    } else {
        info!("Config file not found, using default configuration");
        Ok(Config::default())
    }
}
