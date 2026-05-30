use kernel::model::{
    checkout::SimpleCheckout,
    id::{CheckoutId, ItemId, UserId},
    item::{Item, book, general, laptop},
    user::CheckoutUser,
};
use shared::error::{AppError, AppResult};

pub struct ItemRow {
    pub item_id: ItemId,
    pub category: String,
    pub name: String,
    pub description: String,
    pub location: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub mac_address: Option<mac_address::MacAddress>,
}

impl ItemRow {
    pub fn into_item(self, checkout: Option<SimpleCheckout>) -> AppResult<Item> {
        match self.category.as_str() {
            "general" => Ok(Item::General(general::GeneralItem {
                id: self.item_id,
                name: self.name,
                description: self.description,
                location: self.location,
                checkout,
            })),
            "book" => Ok(Item::Book(book::Book {
                id: self.item_id,
                name: self.name,
                author: self.author.ok_or_else(|| {
                    AppError::ConversionEntityError("Book item is missing author".into())
                })?,
                isbn: self.isbn.ok_or_else(|| {
                    AppError::ConversionEntityError("Book item is missing ISBN".into())
                })?,
                description: self.description,
                location: self.location,
                checkout,
            })),
            "laptop" => Ok(Item::Laptop(laptop::Laptop {
                id: self.item_id,
                name: self.name,
                mac_address: self.mac_address.ok_or_else(|| {
                    AppError::ConversionEntityError("Laptop item is missing MAC address".into())
                })?,
                description: self.description,
                location: self.location,
                checkout,
            })),
            _ => unreachable!("Invalid item category"),
        }
    }
}

pub struct PaginatedItemRow {
    pub id: ItemId,
}

pub struct ItemCheckoutRow {
    pub checkout_id: CheckoutId,
    pub item_id: ItemId,
    pub user_id: UserId,
    pub user_name: String,
    pub checked_out_at: chrono::DateTime<chrono::Utc>,
}

impl From<ItemCheckoutRow> for SimpleCheckout {
    fn from(value: ItemCheckoutRow) -> Self {
        SimpleCheckout {
            checkout_id: value.checkout_id,
            checked_out_by: CheckoutUser {
                id: value.user_id,
                name: value.user_name,
            },
            checked_out_at: value.checked_out_at,
        }
    }
}
