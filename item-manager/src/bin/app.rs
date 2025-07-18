use std::sync::Arc;

use anyhow::Context;
use api::{
    handler::{
        auth::ApiDoc as AuthApiDoc, checkout::ApiDoc as CheckoutApiDoc,
        health::ApiDoc as HealthApiDoc, item::ApiDoc as ItemApiDoc, user::ApiDoc as UserApiDoc,
    },
    route::{auth, v1},
};
use axum::http::{HeaderName, Method};
use tower_http::{
    LatencyUnit,
    cors::{self, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if dotenvy::from_filename(".env-item").is_err() {
        println!("Failed to read .env-item file");
    }

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

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(vec![
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("accept"),
        ])
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors::Any)
}

async fn bootstrap() -> anyhow::Result<()> {
    let app_config = shared::config::AppConfig::new()?;
    let pool = adapter::database::connect_database_with(&app_config.database);
    let registry = Arc::new(registry::AppRegistryImpl::new(pool, app_config));

    let mut api_doc = HealthApiDoc::openapi();
    api_doc.merge(AuthApiDoc::openapi());
    api_doc.merge(CheckoutApiDoc::openapi());
    api_doc.merge(ItemApiDoc::openapi());
    api_doc.merge(UserApiDoc::openapi());

    let app = axum::Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api_doc))
        .layer(cors())
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

    let addr = std::net::SocketAddr::new(std::net::Ipv4Addr::UNSPECIFIED.into(), 8081);
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
