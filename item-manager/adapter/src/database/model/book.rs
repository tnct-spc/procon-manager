use kernel::model::{
    book::Book,
    checkout::SimpleCheckout,
    id::{CheckoutId, ItemId, UserId},
    user::CheckoutUser,
};

pub struct BookRow {
    pub item_id: ItemId,
    pub name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
}

impl BookRow {
    pub fn into_book(self, checkout: Option<SimpleCheckout>) -> Book {
        Book {
            id: self.item_id,
            name: self.name,
            author: self.author,
            isbn: self.isbn,
            description: self.description,
            checkout,
        }
    }
}

pub struct PaginatedBookRow {
    pub total: i64,
    pub id: ItemId,
}

pub struct BookCheckoutRow {
    pub checkout_id: CheckoutId,
    pub item_id: ItemId,
    pub user_id: UserId,
    pub user_name: String,
    pub checked_out_at: chrono::DateTime<chrono::Utc>,
}

impl From<BookCheckoutRow> for SimpleCheckout {
    fn from(value: BookCheckoutRow) -> Self {
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
