use chrono::{DateTime, Utc};
use kernel::model::{
    checkout::Checkout,
    id::{CheckoutId, ItemId, UserId},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutsResponse {
    pub items: Vec<CheckoutResponse>,
}

impl From<Vec<Checkout>> for CheckoutsResponse {
    fn from(value: Vec<Checkout>) -> Self {
        Self {
            items: value.into_iter().map(CheckoutResponse::from).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutResponse {
    pub id: CheckoutId,
    pub checked_out_by: UserId,
    #[schema(value_type = String, format = "date-time", example = "2024-04-10T13:15:00Z")]
    pub checked_out_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub returned_at: Option<DateTime<Utc>>,
    pub item_id: ItemId,
}

impl From<Checkout> for CheckoutResponse {
    fn from(value: Checkout) -> Self {
        Self {
            id: value.id,
            checked_out_by: value.checked_out_by,
            checked_out_at: value.checked_out_at,
            returned_at: value.returned_at,
            item_id: value.item_id,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutBookResponse {
    pub id: ItemId,
    pub title: String,
    pub author: String,
    pub isbn: String,
}
