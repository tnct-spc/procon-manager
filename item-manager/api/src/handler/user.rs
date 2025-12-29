use crate::{
    extractor::AuthorizedUser,
    model::{
        checkout::CheckoutsResponse,
        error::ErrorResponse,
        user::{
            CreateUserRequest, RoleName, UpdateUserEmailRequest, UpdateUserEmailRequestWithUserId,
            UpdateUserNameRequest, UpdateUserNameRequestWithUserId, UpdateUserPasswordRequest,
            UpdateUserPasswordRequestWithUserId, UpdateUserRoleRequest,
            UpdateUserRoleRequestWithUserId, UserResponse, UsersResponse,
        },
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{id::UserId, user::event::DeleteUser};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        register_user,
        list_users,
        delete_user,
        change_role,
        get_current_user,
        change_password,
        change_name,
        change_email,
        get_checkouts
    ),
    components(
        schemas(
            UserResponse,
            UsersResponse,
            CreateUserRequest,
            UpdateUserPasswordRequest,
            UpdateUserRoleRequest,
            UpdateUserNameRequest,
            UpdateUserEmailRequest,
            CheckoutsResponse,
            RoleName,
            ErrorResponse
        )
    ),
    tags(
        (name = "users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;

/// Register a new user (Admin only)
///
/// Create a new user account. Only administrators can perform this operation.
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn register_user(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation(
            "Admin access required.".into(),
        ));
    }
    req.validate()?;

    let registered_user = registry.user_repository().create(req.into()).await?;

    Ok(Json(registered_user.into()))
}

/// Get a list of all users
///
/// Retrieve a list of all registered users
#[utoipa::path(
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "Success", body = UsersResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn list_users(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<UsersResponse>> {
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation(
            "Admin access required.".into(),
        ));
    }

    let items = registry
        .user_repository()
        .find_all()
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(Json(UsersResponse { items }))
}

/// Delete a user account (Admin only)
///
/// Delete an existing user account. Only administrators can perform this operation.
#[utoipa::path(
    delete,
    path = "/api/v1/users/{user_id}",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn delete_user(
    user: AuthorizedUser,
    Path(user_id): Path<UserId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation(
            "Admin access required.".into(),
        ));
    }

    registry
        .user_repository()
        .delete(DeleteUser { user_id })
        .await?;

    Ok(StatusCode::OK)
}
/// Change user role (Admin only)
///
/// Update the role of an existing user. Only administrators can perform this operation.
#[utoipa::path(
    put,
    path = "/api/v1/users/{user_id}/role",
    params(
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully"),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn change_role(
    user: AuthorizedUser,
    Path(user_id): Path<UserId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> AppResult<StatusCode> {
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperation(
            "Admin access required.".into(),
        ));
    }

    registry
        .user_repository()
        .update_role(UpdateUserRoleRequestWithUserId::new(user_id, req).into())
        .await?;

    Ok(StatusCode::OK)
}

/// Get current user information
///
/// Retrieve the authenticated user's profile information
#[utoipa::path(
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "Success", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn get_current_user(user: AuthorizedUser) -> Json<UserResponse> {
    Json(UserResponse::from(user.user))
}

/// Change user password
///
/// Update the authenticated user's password
#[utoipa::path(
    put,
    path = "/api/v1/users/me/password",
    request_body = UpdateUserPasswordRequest,
    responses(
        (status = 200, description = "Password updated successfully"),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Invalid current password", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn change_password(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserPasswordRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .user_repository()
        .update_password(UpdateUserPasswordRequestWithUserId::new(user.id(), req).into())
        .await?;

    Ok(StatusCode::OK)
}

/// Change user name
///
/// Update the authenticated user's name
#[utoipa::path(
    put,
    path = "/api/v1/users/me/name",
    request_body = UpdateUserNameRequest,
    responses(
        (status = 200, description = "Name updated successfully"),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn change_name(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserNameRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .user_repository()
        .update_name(UpdateUserNameRequestWithUserId::new(user.id(), req).into())
        .await?;

    Ok(StatusCode::OK)
}

/// Change user email
///
/// Update the authenticated user's email address
#[utoipa::path(
    put,
    path = "/api/v1/users/me/email",
    request_body = UpdateUserEmailRequest,
    responses(
        (status = 200, description = "Email updated successfully"),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn change_email(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserEmailRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .user_repository()
        .update_email(UpdateUserEmailRequestWithUserId::new(user.id(), req).into())
        .await?;

    Ok(StatusCode::OK)
}

/// Get user's active checkouts
///
/// Retrieve a list of items currently checked out by the authenticated user
#[utoipa::path(
    get,
    path = "/api/v1/users/me/checkouts",
    responses(
        (status = 200, description = "Success", body = CheckoutsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "users"
)]
pub async fn get_checkouts(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_unreturned_by_user_id(user.id())
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}
