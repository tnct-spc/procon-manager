use kernel::model::{
    checkout::SimpleCheckout,
    id::{CheckoutId, ItemId, UserId},
    item::{Item, book, general, laptop},
    user::CheckoutUser,
};

pub struct ItemRow {
    pub item_id: ItemId,
    pub category: String,
    pub name: String,
    pub description: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub mac_address: Option<mac_address::MacAddress>,
}

impl ItemRow {
    pub fn into_item(self, checkout: Option<SimpleCheckout>) -> Item {
        match self.category.as_str() {
            "general" => Item::General(general::GeneralItem {
                id: self.item_id,
                name: self.name,
                description: self.description,
                checkout,
            }),
            "book" => Item::Book(book::Book {
                id: self.item_id,
                name: self.name,
                author: self.author.unwrap_or_default(),
                isbn: self.isbn.unwrap_or_default(),
                description: self.description,
                checkout,
            }),
            "laptop" => Item::Laptop(laptop::Laptop {
                id: self.item_id,
                name: self.name,
                mac_address: self.mac_address.unwrap_or_default(),
                description: self.description,
                checkout,
            }),
            _ => unreachable!("Invalid item category"),
        }
    }
}

pub struct PaginatedItemRow {
    pub total: i64,
    pub id: ItemId,
    pub category: String,
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
