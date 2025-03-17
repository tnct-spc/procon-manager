use garde::Validate;
use kernel::model::{item::ItemCategory, list::ListOptions};
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct ListQuery {
    #[garde(range(min = 0))]
    #[serde(default = "default_limit")]
    pub limit: i64,

    #[garde(range(min = 0))]
    #[serde(default)]
    pub offset: i64,

    #[serde(default)]
    #[garde(skip)]
    pub category: Option<ItemCategory>,
}

const DEFAULT_LIMIT: i64 = 20;
const fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

impl From<ListQuery> for ListOptions {
    fn from(value: ListQuery) -> Self {
        Self {
            limit: value.limit,
            offset: value.offset,
            category: value.category,
        }
    }
}
