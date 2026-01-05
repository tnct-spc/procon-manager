use garde::Validate;
use kernel::model::{
    checkout::SimpleCheckout,
    id::{CheckoutId, ItemId},
    item::{CreateItem, Item, UpdateItem},
    list::PaginatedList,
};
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::user::CheckoutUser;

// Create Request types

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase", tag = "category")]
pub enum CreateItemRequest {
    #[serde(rename = "general")]
    General {
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        name: String,
        #[garde(length(max = 1024))]
        #[schema(max_length = 1024)]
        description: String,
    },
    #[serde(rename = "book")]
    Book {
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        name: String,
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        author: String,
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        isbn: String,
        #[garde(length(max = 1024))]
        #[schema(max_length = 1024)]
        description: String,
    },
    #[serde(rename = "laptop")]
    Laptop {
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        name: String,
        #[garde(skip)]
        #[schema(value_type = String, example = "00:00:00:00:00:00")]
        mac_address: MacAddress,
        #[garde(length(max = 1024))]
        #[schema(max_length = 1024)]
        description: String,
    },
}

impl From<CreateItemRequest> for CreateItem {
    fn from(value: CreateItemRequest) -> Self {
        match value {
            CreateItemRequest::General { name, description } => {
                CreateItem::General { name, description }
            }
            CreateItemRequest::Book {
                name,
                author,
                isbn,
                description,
            } => CreateItem::Book {
                name,
                author,
                isbn,
                description,
            },
            CreateItemRequest::Laptop {
                name,
                mac_address,
                description,
            } => CreateItem::Laptop {
                name,
                mac_address,
                description,
            },
        }
    }
}

// Update Request types

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase", tag = "category")]
pub enum UpdateItemRequest {
    #[serde(rename = "general")]
    General {
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        name: String,
        #[garde(length(max = 1024))]
        #[schema(max_length = 1024)]
        description: String,
    },
    #[serde(rename = "book")]
    Book {
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        name: String,
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        author: String,
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        isbn: String,
        #[garde(length(max = 1024))]
        #[schema(max_length = 1024)]
        description: String,
    },
    #[serde(rename = "laptop")]
    Laptop {
        #[garde(length(min = 1, max = 255))]
        #[schema(max_length = 255)]
        name: String,
        #[garde(skip)]
        #[schema(value_type = String, example = "00:00:00:00:00:00")]
        mac_address: MacAddress,
        #[garde(length(max = 1024))]
        #[schema(max_length = 1024)]
        description: String,
    },
}

impl UpdateItemRequest {
    pub fn into_update_item(self, item_id: ItemId) -> UpdateItem {
        match self {
            UpdateItemRequest::General { name, description } => UpdateItem::General {
                item_id,
                name,
                description,
            },
            UpdateItemRequest::Book {
                name,
                author,
                isbn,
                description,
            } => UpdateItem::Book {
                item_id,
                name,
                author,
                isbn,
                description,
            },
            UpdateItemRequest::Laptop {
                name,
                mac_address,
                description,
            } => UpdateItem::Laptop {
                item_id,
                name,
                mac_address,
                description,
            },
        }
    }
}

// Response types

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeneralItemResponse {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub checkout: Option<ItemCheckoutResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub id: ItemId,
    pub name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub checkout: Option<ItemCheckoutResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LaptopResponse {
    pub id: ItemId,
    pub name: String,
    #[schema(value_type = String, example = "00:00:00:00:00:00")]
    pub mac_address: MacAddress,
    pub description: String,
    pub checkout: Option<ItemCheckoutResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "category")]
pub enum ItemResponse {
    #[serde(rename = "general")]
    General(GeneralItemResponse),
    #[serde(rename = "book")]
    Book(BookResponse),
    #[serde(rename = "laptop")]
    Laptop(LaptopResponse),
}

impl TryFrom<Item> for ItemResponse {
    type Error = shared::error::AppError;

    fn try_from(value: Item) -> Result<Self, Self::Error> {
        Ok(match value {
            Item::General(item) => ItemResponse::General(GeneralItemResponse {
                id: item.id,
                name: item.name,
                description: item.description,
                checkout: item.checkout.map(ItemCheckoutResponse::from),
            }),
            Item::Book(book) => ItemResponse::Book(BookResponse {
                id: book.id,
                name: book.name,
                author: book.author,
                isbn: book.isbn,
                description: book.description,
                checkout: book.checkout.map(ItemCheckoutResponse::from),
            }),
            Item::Laptop(laptop) => ItemResponse::Laptop(LaptopResponse {
                id: laptop.id,
                name: laptop.name,
                mac_address: laptop.mac_address,
                description: laptop.description,
                checkout: laptop.checkout.map(ItemCheckoutResponse::from),
            }),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedItemResponse {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub items: Vec<ItemResponse>,
}

impl TryFrom<PaginatedList<Item>> for PaginatedItemResponse {
    type Error = shared::error::AppError;

    fn try_from(value: PaginatedList<Item>) -> Result<Self, Self::Error> {
        let PaginatedList {
            total,
            limit,
            offset,
            items,
        } = value;

        let converted_items: Result<Vec<_>, _> =
            items.into_iter().map(ItemResponse::try_from).collect();

        Ok(Self {
            total,
            limit,
            offset,
            items: converted_items?,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemCheckoutResponse {
    pub id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    #[schema(value_type = String, format = "date-time", example = "2024-04-10T13:15:00Z")]
    pub checked_out_at: chrono::DateTime<chrono::Utc>,
}

impl From<SimpleCheckout> for ItemCheckoutResponse {
    fn from(value: SimpleCheckout) -> Self {
        Self {
            id: value.checkout_id,
            checked_out_by: value.checked_out_by.into(),
            checked_out_at: value.checked_out_at,
        }
    }
}
