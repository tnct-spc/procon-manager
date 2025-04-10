use crate::model::auth::{AccessTokenResponse, LoginRequest};
use axum::{Json, extract::State};
use kernel::model::auth::event::CreateToken;
use registry::AppRegistry;
use shared::error::AppResult;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(login),
    components(
        schemas(LoginRequest, AccessTokenResponse)
    ),
    tags(
        (name = "auth", description = "Authentication endpoints")
    )
)]
pub struct ApiDoc;

/// Login to get access token
///
/// Authenticate with email and password to get an access token
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AccessTokenResponse),
        (status = 400, description = "Invalid credentials"),
    ),
    tag = "auth"
)]
pub async fn login(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AccessTokenResponse>> {
    let user_id = registry
        .auth_repository()
        .verify_user(&req.email, &req.password)
        .await?;
    let access_token = registry
        .auth_repository()
        .create_token(CreateToken::new(user_id))
        .await?;

    Ok(Json(AccessTokenResponse {
        user_id,
        access_token: access_token.0,
    }))
}
