use axum::{extract::State, http::StatusCode};
use registry::AppRegistry;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(health_check, health_check_db),
    tags(
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;

/// Check if the API server is running
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "API server is healthy"),
    ),
    tag = "health"
)]
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Check if the database connection is healthy
#[utoipa::path(
    get,
    path = "/api/v1/health/db",
    responses(
        (status = 200, description = "Database connection is healthy"),
        (status = 500, description = "Database connection failed"),
    ),
    tag = "health"
)]
pub async fn health_check_db(State(registry): State<AppRegistry>) -> StatusCode {
    if registry.health_check_repository().check_db().await {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
