use crate::model::{checkout::SimpleCheckout, id::ItemId};

#[derive(Debug, Clone)]
pub struct GeneralItem {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub checkout: Option<SimpleCheckout>,
}

#[derive(Debug, Clone)]
pub struct CreateGeneralItem {}

#[derive(Debug, Clone)]
pub struct UpdateGeneralItem {}

#[derive(Debug)]
pub struct DeleteGeneralItem {}
