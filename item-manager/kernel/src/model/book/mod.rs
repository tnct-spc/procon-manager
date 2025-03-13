use super::{checkout::SimpleCheckout, id::ItemId};

pub mod event;

#[derive(Debug)]
pub struct Book {
    pub id: ItemId,
    pub name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub checkout: Option<SimpleCheckout>,
}
