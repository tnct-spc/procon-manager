use crate::model::id::UserId;

pub struct CreateToken {
    pub user_id: UserId,
    pub access_token: String,
}
impl CreateToken {
    pub fn new(user_id: UserId) -> Self {
        let access_token = uuid::Uuid::new_v4().simple().to_string();
        Self {
            user_id,
            access_token,
        }
    }
}
