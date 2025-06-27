use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};
use std::str::FromStr;

use kernel::model::{auth::AccessToken, id::UserId};

pub struct UserItem {
    pub user_id: UserId,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // subject (user_id)
    pub exp: i64,    // expiration time
    pub iat: i64,    // issued at
}

pub struct JwtSecret(String);

impl JwtSecret {
    pub fn new(secret: String) -> Self {
        Self(secret)
    }

    pub fn create_token(&self, user_id: UserId, ttl: u64) -> AppResult<AccessToken> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::seconds(ttl as i64))
            .ok_or_else(|| {
                AppError::ConversionEntityError("Failed to calculate token expiration".to_string())
            })?;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration.timestamp(),
            iat: Utc::now().timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.0.as_bytes()),
        )
        .map_err(|e| AppError::ConversionEntityError(format!("Failed to create token: {e}")))?;

        Ok(AccessToken(token))
    }

    pub fn verify_token(&self, token: &AccessToken) -> AppResult<Option<UserId>> {
        match decode::<Claims>(
            &token.0,
            &DecodingKey::from_secret(self.0.as_bytes()),
            &Validation::default(),
        ) {
            Ok(token_data) => {
                let user_id = UserId::from_str(&token_data.claims.sub)
                    .map_err(|e| AppError::ConversionEntityError(e.to_string()))?;
                Ok(Some(user_id))
            }
            Err(err) => match err.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => Ok(None),
                _ => Err(AppError::UnauthenticatedError),
            },
        }
    }
}
