use crate::model::{checkout::SimpleCheckout, id::ItemId};

#[derive(Debug, Clone)]
pub struct Book {
    pub id: ItemId,
    pub name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub checkout: Option<SimpleCheckout>,
}
