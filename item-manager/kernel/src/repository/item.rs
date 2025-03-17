use async_trait::async_trait;
use shared::error::AppResult;

use crate::model::{
    id::ItemId,
    item::{CreateItem, DeleteItem, Item, UpdateItem},
    list::{ListOptions, PaginatedList},
};

#[mockall::automock]
#[async_trait]
pub trait ItemRepository: Send + Sync {
    async fn create(&self, event: CreateItem) -> AppResult<()>;
    async fn find_all(&self, options: ListOptions) -> AppResult<PaginatedList<Item>>;
    async fn find_by_id(&self, id: ItemId) -> AppResult<Option<Item>>;
    async fn update(&self, event: UpdateItem) -> AppResult<()>;
    async fn delete(&self, event: DeleteItem) -> AppResult<()>;
}
