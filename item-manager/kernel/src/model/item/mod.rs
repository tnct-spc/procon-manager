use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use utoipa::ToSchema;

use super::id::ItemId;

pub mod book;
pub mod general;
pub mod laptop;

#[derive(Debug, Clone, Copy, EnumString, AsRefStr, PartialEq, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ItemCategory {
    General,
    Book,
    Laptop,
}

#[derive(Debug, Clone, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum Item {
    General(general::GeneralItem),
    Book(book::Book),
    Laptop(laptop::Laptop),
}

#[derive(Debug, Clone, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum CreateItem {
    General {
        name: String,
        description: String,
    },
    Book {
        name: String,
        author: String,
        isbn: String,
        description: String,
    },
    Laptop {
        name: String,
        mac_address: mac_address::MacAddress,
        description: String,
    },
}

#[derive(Debug, Clone, AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum UpdateItem {
    General {
        item_id: ItemId,
        name: String,
        description: String,
    },
    Book {
        item_id: ItemId,
        name: String,
        author: String,
        isbn: String,
        description: String,
    },
    Laptop {
        item_id: ItemId,
        name: String,
        mac_address: mac_address::MacAddress,
        description: String,
    },
}

#[derive(Debug, Clone)]
pub struct DeleteItem {
    pub item_id: ItemId,
}
