use async_trait::async_trait;
use shared::error::AppResult;

use crate::model::{
    book::{
        Book,
        event::{CreateBook, DeleteBook, UpdateBook},
    },
    id::ItemId,
    list::{ListOptions, PaginatedList},
};

#[mockall::automock]
#[async_trait]
pub trait CommonItemRepository<
    T: Send + Sync,
    Id: Send + Sync,
    CreateEvent: Send + Sync,
    UpdateEvent: Send + Sync,
    DeleteEvent: Send + Sync,
>: Send + Sync
{
    async fn create(&self, event: CreateEvent) -> AppResult<()>;
    async fn find_all(&self, options: ListOptions) -> AppResult<PaginatedList<T>>;
    async fn find_by_id(&self, id: Id) -> AppResult<Option<T>>;
    async fn update(&self, event: UpdateEvent) -> AppResult<()>;
    async fn delete(&self, event: DeleteEvent) -> AppResult<()>;
}

pub type BookRepository =
    std::sync::Arc<dyn CommonItemRepository<Book, ItemId, CreateBook, UpdateBook, DeleteBook>>;
