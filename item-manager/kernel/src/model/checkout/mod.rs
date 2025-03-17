use chrono::{DateTime, Utc};

use super::id::{CheckoutId, ItemId, UserId};
use super::user::CheckoutUser;

pub mod event;

#[derive(Debug, Clone)]
pub struct Checkout {
    pub id: CheckoutId,
    pub checked_out_by: UserId,
    pub checked_out_at: DateTime<Utc>,
    pub returned_at: Option<DateTime<Utc>>,
    pub item_id: ItemId,
}

#[derive(Debug, Clone)]
pub struct SimpleCheckout {
    pub checkout_id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    pub checked_out_at: chrono::DateTime<chrono::Utc>,
}
