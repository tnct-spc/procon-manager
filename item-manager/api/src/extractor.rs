use axum::extract::FromRequestParts;
use axum::http::{header::COOKIE, request::Parts};
use kernel::model::auth::AccessToken;
use kernel::model::id::UserId;
use kernel::model::role::Role;
use kernel::model::user::User;
use registry::AppRegistry;
use shared::error::AppError;

pub struct AuthorizedUser {
    pub access_token: AccessToken,
    pub user: User,
}

impl AuthorizedUser {
    pub fn id(&self) -> UserId {
        self.user.id
    }
    pub fn is_admin(&self) -> bool {
        self.user.role == Role::Admin
    }
}

impl FromRequestParts<AppRegistry> for AuthorizedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        registry: &AppRegistry,
    ) -> Result<Self, Self::Rejection> {
        let web_config = registry.web_config();
        let cookie_header = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::UnauthorizedError)?;
        let token = cookie_header
            .split(';')
            .filter_map(|cookie| cookie.trim().split_once('='))
            .find_map(|(name, value)| {
                (name == web_config.access_token_cookie_name).then(|| value.to_string())
            })
            .ok_or(AppError::UnauthorizedError)?;
        let access_token = AccessToken(token);

        let user_id = registry
            .auth_repository()
            .fetch_user_id_from_token(&access_token)
            .await?
            .ok_or(AppError::UnauthenticatedError)?;

        let user = registry
            .user_repository()
            .find_current_user(user_id)
            .await?
            .ok_or(AppError::UnauthenticatedError)?;

        Ok(Self { access_token, user })
    }
}
