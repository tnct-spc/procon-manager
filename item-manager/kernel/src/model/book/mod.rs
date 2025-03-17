use super::{checkout::SimpleCheckout, id::BookId};

pub mod event;

#[derive(Debug)]
pub struct Book {
    pub id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub checkout: Option<SimpleCheckout>,
}
