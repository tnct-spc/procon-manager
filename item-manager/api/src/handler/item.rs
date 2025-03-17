use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{id::ItemId, item::DeleteItem};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

use crate::{
    extractor::AuthorizedUser,
    model::{
        item::{CreateItemRequest, ItemResponse, PaginatedItemResponse, UpdateItemRequest},
        list::ListQuery,
    },
};

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
