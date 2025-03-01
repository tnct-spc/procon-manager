use kernel::model::{
    book::{Book, Checkout},
    id::{BookId, CheckoutId, UserId},
    user::{BookOwner, CheckoutUser},
};

pub struct BookRow {
    pub book_id: BookId,
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub owned_by: UserId,
    pub owner_name: String,
}

impl BookRow {
    pub fn into_book(self, checkout: Option<Checkout>) -> Book {
        Book {
            id: self.book_id,
            title: self.title,
            author: self.author,
            isbn: self.isbn,
            description: self.description,
            owner: BookOwner {
                id: self.owned_by,
                name: self.owner_name,
            },
            checkout,
        }
    }
}

pub struct PaginatedBookRow {
    pub total: i64,
    pub id: BookId,
}

pub struct BookCheckoutRow {
    pub checkout_id: CheckoutId,
    pub book_id: BookId,
    pub user_id: UserId,
    pub user_name: String,
    pub checked_out_at: chrono::DateTime<chrono::Utc>,
}

impl From<BookCheckoutRow> for Checkout {
    fn from(value: BookCheckoutRow) -> Self {
        Checkout {
            checkout_id: value.checkout_id,
            checked_out_by: CheckoutUser {
                id: value.user_id,
                name: value.user_name,
            },
            checked_out_at: value.checked_out_at,
        }
    }
}
