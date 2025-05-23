use async_trait::async_trait;
use shared::error::AppResult;

use crate::model::{
    checkout::{
        Checkout,
        event::{CreateCheckout, UpdateReturned},
    },
    id::{ItemId, UserId},
};

#[mockall::automock]
#[async_trait]
pub trait CheckoutRepository: Send + Sync {
    async fn create(&self, event: CreateCheckout) -> AppResult<()>;
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()>;
    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>>;
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>>;
    async fn find_history_by_item_id(&self, item_id: ItemId) -> AppResult<Vec<Checkout>>;
}
