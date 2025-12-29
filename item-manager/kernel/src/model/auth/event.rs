use crate::model::id::UserId;

pub struct CreateToken {
    pub user_id: UserId,
}

impl CreateToken {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}
