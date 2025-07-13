use derive_new::new;
use garde::Validate;
use kernel::model::{
    id::UserId,
    role::Role,
    user::{
        User,
        event::{CreateUser, UpdateUserEmail, UpdateUserName, UpdateUserPassword, UpdateUserRole},
    },
};
use serde::{Deserialize, Serialize};
use strum::VariantNames;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, VariantNames, ToSchema)]
#[strum(serialize_all = "kebab-case")]
pub enum RoleName {
    Admin,
    User,
}

impl From<Role> for RoleName {
    fn from(value: Role) -> Self {
        match value {
            Role::Admin => Self::Admin,
            Role::User => Self::User,
        }
    }
}

impl From<RoleName> for Role {
    fn from(value: RoleName) -> Self {
        match value {
            RoleName::Admin => Self::Admin,
            RoleName::User => Self::User,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UsersResponse {
    pub items: Vec<UserResponse>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub role: RoleName,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
            role: value.role.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserPasswordRequest {
    #[garde(length(min = 1))]
    pub current_password: String,
    #[garde(length(min = 1))]
    pub new_password: String,
}

#[derive(new)]
pub struct UpdateUserPasswordRequestWithUserId(UserId, UpdateUserPasswordRequest);
impl From<UpdateUserPasswordRequestWithUserId> for UpdateUserPassword {
    fn from(value: UpdateUserPasswordRequestWithUserId) -> Self {
        UpdateUserPassword {
            user_id: value.0,
            current_password: value.1.current_password,
            new_password: value.1.new_password,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 1))]
    pub password: String,
}

impl From<CreateUserRequest> for CreateUser {
    fn from(value: CreateUserRequest) -> Self {
        Self {
            name: value.name,
            email: value.email,
            password: value.password,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRoleRequest {
    pub role: RoleName,
}

#[derive(new)]
pub struct UpdateUserRoleRequestWithUserId(UserId, UpdateUserRoleRequest);
impl From<UpdateUserRoleRequestWithUserId> for UpdateUserRole {
    fn from(value: UpdateUserRoleRequestWithUserId) -> Self {
        Self {
            user_id: value.0,
            role: value.1.role.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserNameRequest {
    #[garde(length(min = 1))]
    pub name: String,
}

#[derive(new)]
pub struct UpdateUserNameRequestWithUserId(UserId, UpdateUserNameRequest);
impl From<UpdateUserNameRequestWithUserId> for UpdateUserName {
    fn from(value: UpdateUserNameRequestWithUserId) -> Self {
        Self {
            user_id: value.0,
            name: value.1.name,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserEmailRequest {
    #[garde(email)]
    pub email: String,
}

#[derive(new)]
pub struct UpdateUserEmailRequestWithUserId(UserId, UpdateUserEmailRequest);
impl From<UpdateUserEmailRequestWithUserId> for UpdateUserEmail {
    fn from(value: UpdateUserEmailRequestWithUserId) -> Self {
        Self {
            user_id: value.0,
            email: value.1.email,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutUser {
    pub id: UserId,
    pub name: String,
}

impl From<kernel::model::user::CheckoutUser> for CheckoutUser {
    fn from(value: kernel::model::user::CheckoutUser) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}
