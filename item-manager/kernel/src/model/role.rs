use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};
use utoipa::ToSchema;

#[derive(
    Debug,
    EnumString,
    AsRefStr,
    EnumIter,
    Default,
    PartialEq,
    Eq,
    Copy,
    Clone,
    Serialize,
    Deserialize,
    ToSchema,
)]
pub enum Role {
    Admin,
    #[default]
    User,
}
