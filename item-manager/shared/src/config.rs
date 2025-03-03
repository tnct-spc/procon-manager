use anyhow::Context;

pub struct AppConfig {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
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
        Ok(Self { database, auth })
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
