use crate::model::{checkout::SimpleCheckout, id::ItemId};

#[derive(Debug, Clone)]
pub struct Laptop {
    pub id: ItemId,
    pub name: String,
    pub mac_address: mac_address::MacAddress,
    pub description: String,
    pub checkout: Option<SimpleCheckout>,
}
