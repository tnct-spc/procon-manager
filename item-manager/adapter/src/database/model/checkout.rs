use kernel::model::{
    checkout::Checkout,
    id::{CheckoutId, ItemId, UserId},
};
use sqlx::types::chrono::{DateTime, Utc};

pub struct CheckoutStateRow {
    pub checkout_id: Option<CheckoutId>,
    pub item_id: ItemId,
    pub user_id: Option<UserId>,
}

pub struct CheckoutRow {
    pub checkout_id: CheckoutId,
    pub item_id: ItemId,
    pub user_id: UserId,
    pub checked_out_at: DateTime<Utc>,
}

impl From<CheckoutRow> for Checkout {
    fn from(value: CheckoutRow) -> Self {
        Checkout {
            id: value.checkout_id,
            checked_out_by: value.user_id,
            checked_out_at: value.checked_out_at,
            returned_at: None,
            item_id: value.item_id,
        }
    }
}

pub struct ReturnedCheckoutRow {
    pub checkout_id: CheckoutId,
    pub item_id: ItemId,
    pub user_id: UserId,
    pub checked_out_at: DateTime<Utc>,
    pub returned_at: DateTime<Utc>,
}

impl From<ReturnedCheckoutRow> for Checkout {
    fn from(value: ReturnedCheckoutRow) -> Self {
        Checkout {
            id: value.checkout_id,
            checked_out_by: value.user_id,
            checked_out_at: value.checked_out_at,
            returned_at: Some(value.returned_at),
            item_id: value.item_id,
        }
    }
}
