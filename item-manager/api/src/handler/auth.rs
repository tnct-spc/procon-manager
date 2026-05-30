use crate::model::{
    auth::{LoginRequest, LoginResponse},
    error::ErrorResponse,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
};
use kernel::model::auth::event::CreateToken;
use registry::AppRegistry;
use shared::error::AppResult;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(login, logout),
    components(
        schemas(LoginRequest, LoginResponse, ErrorResponse)
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
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
    ),
    tag = "auth"
)]
pub async fn login(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>,
) -> AppResult<(HeaderMap, Json<LoginResponse>)> {
    let user_id = registry
        .auth_repository()
        .verify_user(&req.email, &req.password)
        .await?;
    let access_token = registry
        .auth_repository()
        .create_token(CreateToken::new(user_id))
        .await?;

    let web_config = registry.web_config();
    let cookie = format!(
        "{}={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
        web_config.access_token_cookie_name,
        access_token.0,
        web_config.access_token_cookie_max_age_seconds
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| shared::error::AppError::UnauthorizedError)?,
    );

    Ok((headers, Json(LoginResponse { user_id })))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses((status = 204, description = "Logout successful")),
    tag = "auth"
)]
pub async fn logout(State(registry): State<AppRegistry>) -> AppResult<(HeaderMap, StatusCode)> {
    let web_config = registry.web_config();
    let cookie = format!(
        "{}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0",
        web_config.access_token_cookie_name,
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|_| shared::error::AppError::UnauthorizedError)?,
    );

    Ok((headers, StatusCode::NO_CONTENT))
}
