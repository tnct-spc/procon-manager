use anyhow::Context;

pub struct AppConfig {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub web: WebConfig,
}

impl AppConfig {
    pub fn new() -> anyhow::Result<Self> {
        let database = DatabaseConfig {
            host: std::env::var("DATABASE_HOST").context("DATABASE_HOST")?,
            username: std::env::var("DATABASE_USERNAME").context("DATABASE_USERNAME")?,
            password: std::env::var("DATABASE_PASSWORD").context("DATABASE_PASSWORD")?,
            database: std::env::var("DATABASE_NAME").context("DATABASE_NAME")?,
        };
        let auth = AuthConfig {
            ttl: std::env::var("AUTH_TOKEN_TTL")
                .context("AUTH_TOKEN_TTL")?
                .parse()?,
            secret: std::env::var("JWT_SECRET").context("JWT_SECRET")?,
        };
        let web = WebConfig {
            frontend_origin: std::env::var("FRONTEND_ORIGIN").context("FRONTEND_ORIGIN")?,
            access_token_cookie_name: std::env::var("ACCESS_TOKEN_COOKIE_NAME")
                .unwrap_or_else(|_| "access_token".to_string()),
            access_token_cookie_max_age_seconds: auth.ttl,
        };
        Ok(Self {
            database,
            auth,
            web,
        })
    }
}

pub struct DatabaseConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub struct AuthConfig {
    pub ttl: u64,
    pub secret: String,
}

#[derive(Clone)]
pub struct WebConfig {
    pub frontend_origin: String,
    pub access_token_cookie_name: String,
    pub access_token_cookie_max_age_seconds: u64,
}
