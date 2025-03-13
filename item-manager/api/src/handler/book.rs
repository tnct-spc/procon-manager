use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{book::event::DeleteBook, id::ItemId};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

use crate::{
    extractor::AuthorizedUser,
    model::{
        book::{
            BookResponse, CreateBookRequest, PaginatedBookResponse, UpdateBookRequest,
            UpdateBookRequestWithIds,
        },
        list::ListQuery,
    },
};

pub async fn register_book(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateBookRequest>,
) -> Result<StatusCode, AppError> {
    req.validate()?;

    registry
        .book_repository()
        .create(req.into())
        .await
        .map(|_| StatusCode::CREATED)
}

pub async fn show_book_list(
    _user: AuthorizedUser,
    Query(query): Query<ListQuery>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<PaginatedBookResponse>> {
    query.validate()?;

    registry
        .book_repository()
        .find_all(query.into())
        .await
        .map(PaginatedBookResponse::from)
        .map(Json)
}

pub async fn show_book(
    _user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<BookResponse>> {
    registry
        .book_repository()
        .find_by_id(item_id)
        .await
        .and_then(|bc| match bc {
            Some(bc) => Ok(Json(bc.into())),
            None => Err(AppError::EntityNotFound("not found".into())),
        })
}

pub async fn update_book(
    _user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateBookRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    let update_book = UpdateBookRequestWithIds::new(item_id, req);
    registry
        .book_repository()
        .update(update_book.into())
        .await
        .map(|_| StatusCode::OK)
}

pub async fn delete_book(
    _user: AuthorizedUser,
    Path(item_id): Path<ItemId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let delete_book = DeleteBook { item_id };
    registry
        .book_repository()
        .delete(delete_book)
        .await
        .map(|_| StatusCode::OK)
}
