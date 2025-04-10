use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{id::ItemId, item::DeleteItem};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};
use utoipa::OpenApi;

use crate::{
    extractor::AuthorizedUser,
    model::{
        item::{CreateItemRequest, ItemResponse, PaginatedItemResponse, UpdateItemRequest},
        list::ListQuery,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_item,
        list_items,
        get_item,
        update_item,
        delete_item
    ),
    components(
        schemas(
            CreateItemRequest,
            UpdateItemRequest,
            ItemResponse,
            crate::model::item::GeneralItemResponse,
            crate::model::item::BookResponse,
            crate::model::item::LaptopResponse,
            crate::model::item::PaginatedItemResponse,
            crate::model::item::ItemCheckoutResponse,
            ListQuery
        )
    ),
    tags(
        (name = "items", description = "Item management endpoints")
    )
)]
pub struct ApiDoc;

/// Create a new item
///
/// Create a new item with the provided details. The item category (general, book, or laptop) determines the required fields.
#[utoipa::path(
    post,
    path = "/api/v1/items",
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created successfully"),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "items"
)]
pub async fn create_item(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateItemRequest>,
) -> Result<StatusCode, AppError> {
    req.validate()?;

    registry
        .item_repository()
        .create(req.into())
        .await
        .map(|_| StatusCode::CREATED)
}

/// List items
///
/// Get a paginated list of all items
#[utoipa::path(
    get,
    path = "/api/v1/items",
    params(
        ("limit" = i64, Query, description = "Number of items to return"),
        ("offset" = i64, Query, description = "Number of items to skip"),
    ),
    responses(
        (status = 200, description = "Success", body = PaginatedItemResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "items"
)]
pub async fn list_items(
    _user: AuthorizedUser,
    Query(query): Query<ListQuery>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<PaginatedItemResponse>> {
    query.validate()?;

    registry
        .item_repository()
        .find_all(query.into())
        .await
        .and_then(PaginatedItemResponse::try_from)
        .map(Json)
}

/// Get item by ID
///
/// Get details of a specific item by its ID
#[utoipa::path(
    get,
    path = "/api/v1/items/{item_id}",
    params(
        ("item_id" = String, Path, description = "Item ID"),
    ),
    responses(
        (status = 200, description = "Success", body = ItemResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Item not found"),
    ),
    security(("jwt" = [])),
    tag = "items"
)]
pub async fn get_item(
    _user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<ItemResponse>> {
    registry
        .item_repository()
        .find_by_id(item_id)
        .await
        .and_then(|item| match item {
            Some(item) => ItemResponse::try_from(item),
            None => Err(AppError::EntityNotFound("Item not found".into())),
        })
        .map(Json)
}

/// Update item
///
/// Update an existing item with new details
#[utoipa::path(
    put,
    path = "/api/v1/items/{item_id}",
    params(
        ("item_id" = String, Path, description = "Item ID"),
    ),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, description = "Item updated successfully"),
        (status = 400, description = "Invalid request body"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Item not found"),
    ),
    security(("jwt" = [])),
    tag = "items"
)]
pub async fn update_item(
    _user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateItemRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .item_repository()
        .update(req.into_update_item(item_id))
        .await
        .map(|_| StatusCode::OK)
}

/// Delete item
///
/// Delete an existing item
#[utoipa::path(
    delete,
    path = "/api/v1/items/{item_id}",
    params(
        ("item_id" = String, Path, description = "Item ID"),
    ),
    responses(
        (status = 200, description = "Item deleted successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Item not found"),
    ),
    security(("jwt" = [])),
    tag = "items"
)]
pub async fn delete_item(
    _user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let delete_item = DeleteItem { item_id };
    registry
        .item_repository()
        .delete(delete_item)
        .await
        .map(|_| StatusCode::OK)
}
