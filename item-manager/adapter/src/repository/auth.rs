use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        auth::{AccessToken, event::CreateToken},
        id::UserId,
    },
    repository::auth::AuthRepository,
};
use shared::error::{AppError, AppResult};

use crate::database::{
    ConnectionPool,
    model::auth::{JwtSecret, UserItem},
};

#[derive(new)]
pub struct AuthRepositoryImpl {
    db: ConnectionPool,
    secret: JwtSecret,
    ttl: u64,
}

#[async_trait]
impl AuthRepository for AuthRepositoryImpl {
    async fn fetch_user_id_from_token(
        &self,
        access_token: &AccessToken,
    ) -> AppResult<Option<UserId>> {
        self.secret.verify_token(access_token)
    }

    async fn verify_user(&self, email: &str, password: &str) -> AppResult<UserId> {
        let user_item = sqlx::query_as!(
            UserItem,
            r#"
                SELECT user_id, password_hash FROM users
                WHERE email = $1;
            "#,
            email
        )
        .fetch_one(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        let valid = bcrypt::verify(password, &user_item.password_hash)?;
        if !valid {
            return Err(AppError::UnauthenticatedError);
        }
        Ok(user_item.user_id)
    }

    async fn create_token(&self, event: CreateToken) -> AppResult<AccessToken> {
        self.secret.create_token(event.user_id, self.ttl)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use kernel::{model::user::event::CreateUser, repository::user::UserRepository};

    use crate::repository::user::UserRepositoryImpl;

    use super::*;

    #[sqlx::test(fixtures("common"))]
    async fn test_verify_user(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let user_repo = UserRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let ttl = 3600; // 1 hour
        let secret = JwtSecret::new("test_secret".to_string());
        let auth_repo = AuthRepositoryImpl::new(ConnectionPool::new(pool), secret, ttl);

        // Create a test user
        let user = user_repo
            .create(CreateUser {
                name: "Auth Test User".into(),
                email: "auth_test@example.com".into(),
                password: "test_password".into(),
            })
            .await?;

        // Test with correct credentials
        let result = auth_repo
            .verify_user("auth_test@example.com", "test_password")
            .await?;
        assert_eq!(result, user.id);

        // Test with incorrect password
        let result = auth_repo
            .verify_user("auth_test@example.com", "wrong_password")
            .await;
        assert!(result.is_err());

        // Test with non-existent email
        let result = auth_repo
            .verify_user("nonexistent@example.com", "test_password")
            .await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_token_operations(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let secret = JwtSecret::new("test_secret".to_string());
        let ttl = 3600; // 1 hour
        let auth_repo = AuthRepositoryImpl::new(ConnectionPool::new(pool), secret, ttl);

        let user_id = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;

        // Test token creation and verification
        let event = CreateToken::new(user_id);
        let token = auth_repo.create_token(event).await?;

        // Test fetching user ID from token
        let fetched_user_id = auth_repo.fetch_user_id_from_token(&token).await?;
        assert_eq!(fetched_user_id, Some(user_id));

        Ok(())
    }
}
