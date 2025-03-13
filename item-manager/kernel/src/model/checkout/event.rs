use chrono::{DateTime, Utc};
use derive_new::new;

use crate::model::id::{CheckoutId, ItemId, UserId};

#[derive(new)]
pub struct CreateCheckout {
    pub item_id: ItemId,
    pub checked_out_by: UserId,
    pub checked_out_at: DateTime<Utc>,
}

#[derive(new)]
pub struct UpdateReturned {
    pub checkout_id: CheckoutId,
    pub item_id: ItemId,
    pub returned_by: UserId,
    pub returned_at: DateTime<Utc>,
}
