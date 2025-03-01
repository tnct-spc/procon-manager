use strum::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, EnumString, AsRefStr, EnumIter, Default, PartialEq, Eq, Copy, Clone)]
pub enum Role {
    Admin,
    #[default]
    User,
}
