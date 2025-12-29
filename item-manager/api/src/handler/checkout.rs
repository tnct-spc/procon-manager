use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use kernel::model::{
    checkout::event::{CreateCheckout, UpdateReturned},
    id::{CheckoutId, ItemId},
};
use registry::AppRegistry;
use shared::error::AppResult;
use utoipa::OpenApi;

use crate::{
    extractor::AuthorizedUser,
    model::{checkout::CheckoutsResponse, error::ErrorResponse},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        checkout_book,
        return_book,
        show_checked_out_list,
        checkout_history
    ),
    components(
        schemas(CheckoutsResponse, ErrorResponse)
    ),
    tags(
        (name = "checkouts", description = "Item checkout management endpoints")
    )
)]
pub struct ApiDoc;

/// Checkout an item
///
/// Create a new checkout record for an item
#[utoipa::path(
    post,
    path = "/api/v1/items/{item_id}/checkouts",
    params(
        ("item_id" = String, Path, description = "Item ID to checkout"),
    ),
    responses(
        (status = 201, description = "Item checked out successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 409, description = "Item already checked out", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "checkouts"
)]
pub async fn checkout_book(
    user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let create_checkout_history = CreateCheckout::new(item_id, user.id(), chrono::Utc::now());

    registry
        .checkout_repository()
        .create(create_checkout_history)
        .await
        .map(|_| StatusCode::CREATED)
}

/// Return a checked out item
///
/// Mark a checked out item as returned
#[utoipa::path(
    put,
    path = "/api/v1/items/{item_id}/checkouts/{checkout_id}/returned",
    params(
        ("item_id" = String, Path, description = "Item ID to return"),
        ("checkout_id" = String, Path, description = "Checkout record ID"),
    ),
    responses(
        (status = 200, description = "Item returned successfully"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Item or checkout record not found", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "checkouts"
)]
pub async fn return_book(
    user: AuthorizedUser,
    Path((item_id, checkout_id)): Path<(ItemId, CheckoutId)>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let update_returned = UpdateReturned::new(
        checkout_id,
        item_id,
        user.id(),
        user.user.role,
        chrono::Utc::now(),
    );

    registry
        .checkout_repository()
        .update_returned(update_returned)
        .await
        .map(|_| StatusCode::OK)
}

/// List all currently checked out items
///
/// Get a list of all items that are currently checked out
#[utoipa::path(
    get,
    path = "/api/v1/items/checkouts",
    responses(
        (status = 200, description = "Success", body = CheckoutsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "checkouts"
)]
pub async fn show_checked_out_list(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_unreturned_all()
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}

/// Get item checkout history
///
/// Get the complete checkout history for a specific item
#[utoipa::path(
    get,
    path = "/api/v1/items/{item_id}/checkout-history",
    params(
        ("item_id" = String, Path, description = "Item ID"),
    ),
    responses(
        (status = 200, description = "Success", body = CheckoutsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Item not found", body = ErrorResponse),
    ),
    security(("jwt" = [])),
    tag = "checkouts"
)]
pub async fn checkout_history(
    _user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    let exists = registry.item_repository().find_by_id(item_id).await?;
    if exists.is_none() {
        return Err(shared::error::AppError::EntityNotFound(
            "Item not found".into(),
        ));
    }

    registry
        .checkout_repository()
        .find_history_by_item_id(item_id)
        .await
        .map(CheckoutsResponse::from)
        .map(Json)
}
