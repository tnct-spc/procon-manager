use std::sync::Arc;

use adapter::database::ConnectionPool;
use api::{
    handler::{
        auth::ApiDoc as AuthApiDoc, checkout::ApiDoc as CheckoutApiDoc,
        health::ApiDoc as HealthApiDoc, item::ApiDoc as ItemApiDoc, user::ApiDoc as UserApiDoc,
    },
    route::{auth, v1},
};
use axum::http::Method;
use tower_http::{
    LatencyUnit,
    cors::{self, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let app_config = shared::config::AppConfig::new(secrets)?;
    let pool = ConnectionPool::new(pool);
    sqlx::migrate!("./adapter/migrations")
        .run(pool.inner_ref())
        .await
        .expect("Failed to run migrations");
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

    Ok(app.into())
}

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(cors::Any)
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors::Any)
}
