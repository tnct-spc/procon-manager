use chrono::{DateTime, Utc};

use super::id::{BookId, CheckoutId, UserId};
use super::user::CheckoutUser;

pub mod event;

#[derive(Debug)]
pub struct Checkout {
    pub id: CheckoutId,
    pub checked_out_by: UserId,
    pub checked_out_at: DateTime<Utc>,
    pub returned_at: Option<DateTime<Utc>>,
    pub book: CheckoutBook,
}

#[derive(Debug)]
pub struct CheckoutBook {
    pub book_id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
}

#[derive(Debug)]
pub struct SimpleCheckout {
    pub checkout_id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    pub checked_out_at: chrono::DateTime<chrono::Utc>,
}
