use std::sync::Arc;

use adapter::redis::RedisClient;
use anyhow::Context;
use api::route::{auth, v1};
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger()?;
    bootstrap().await
}

fn init_logger() -> anyhow::Result<()> {
    let log_level = match shared::env::which() {
        shared::env::Environment::Production => "info",
        shared::env::Environment::Development => "debug",
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .try_init()?;

    Ok(())
}

async fn bootstrap() -> anyhow::Result<()> {
    let app_config = shared::config::AppConfig::new()?;
    let pool = adapter::database::connect_database_with(&app_config.database);
    let kv = Arc::new(RedisClient::new(&app_config.redis)?);
    let registry = registry::AppRegistry::new(pool, kv, app_config);
    let app = axum::Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(tracing::Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .with_state(registry);

    let addr = std::net::SocketAddr::new(std::net::Ipv4Addr::LOCALHOST.into(), 8081);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("Listening on {addr}");

    axum::serve(listener, app)
        .await
        .context("Failed to start server")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Unexpected error"
            );
        })
}
