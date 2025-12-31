use crate::model::{checkout::SimpleCheckout, id::ItemId};

#[derive(Debug, Clone)]
pub struct GeneralItem {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub checkout: Option<SimpleCheckout>,
}
