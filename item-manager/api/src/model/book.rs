use derive_new::new;
use garde::Validate;
use kernel::model::{
    book::{
        Book,
        event::{CreateBook, UpdateBook},
    },
    checkout::SimpleCheckout,
    id::{CheckoutId, ItemId},
    list::PaginatedList,
};
use serde::{Deserialize, Serialize};

use super::user::CheckoutUser;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookRequest {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(length(min = 1))]
    pub author: String,
    #[garde(length(min = 1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

impl From<CreateBookRequest> for CreateBook {
    fn from(value: CreateBookRequest) -> Self {
        let CreateBookRequest {
            name,
            author,
            isbn,
            description,
        } = value;
        CreateBook {
            name,
            author,
            isbn,
            description,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBookRequest {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(length(min = 1))]
    pub author: String,
    #[garde(length(min = 1))]
    pub isbn: String,
    #[garde(skip)]
    pub description: String,
}

#[derive(new)]
pub struct UpdateBookRequestWithIds(ItemId, UpdateBookRequest);
impl From<UpdateBookRequestWithIds> for UpdateBook {
    fn from(value: UpdateBookRequestWithIds) -> Self {
        let UpdateBookRequestWithIds(
            item_id,
            UpdateBookRequest {
                name,
                author,
                isbn,
                description,
            },
        ) = value;
        UpdateBook {
            item_id,
            name,
            author,
            isbn,
            description,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookResponse {
    pub id: ItemId,
    pub name: String,
    pub author: String,
    pub isbn: String,
    pub description: String,
    pub checkout: Option<BookCheckoutResponse>,
}

impl From<Book> for BookResponse {
    fn from(value: Book) -> Self {
        Self {
            id: value.id,
            name: value.name,
            author: value.author,
            isbn: value.isbn,
            description: value.description,
            checkout: value.checkout.map(BookCheckoutResponse::from),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedBookResponse {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub items: Vec<BookResponse>,
}

impl From<PaginatedList<Book>> for PaginatedBookResponse {
    fn from(value: PaginatedList<Book>) -> Self {
        let PaginatedList {
            total,
            limit,
            offset,
            items,
        } = value;
        Self {
            total,
            limit,
            offset,
            items: items.into_iter().map(BookResponse::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookCheckoutResponse {
    pub id: CheckoutId,
    pub checked_out_by: CheckoutUser,
    pub checked_out_at: chrono::DateTime<chrono::Utc>,
}

impl From<SimpleCheckout> for BookCheckoutResponse {
    fn from(value: SimpleCheckout) -> Self {
        Self {
            id: value.checkout_id,
            checked_out_by: value.checked_out_by.into(),
            checked_out_at: value.checked_out_at,
        }
    }
}
