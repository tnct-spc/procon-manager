use derive_new::new;
use shared::{
    config::DatabaseConfig,
    error::{AppError, AppResult},
};
use sqlx::postgres::PgConnectOptions;

pub mod model;

fn make_pg_connect_options(cfg: &DatabaseConfig) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&cfg.host)
        // .port(cfg.port)
        .username(&cfg.username)
        .password(&cfg.password)
        .database(&cfg.database)
}

#[derive(Clone, new)]
pub struct ConnectionPool(sqlx::PgPool);

impl ConnectionPool {
    pub fn inner_ref(&self) -> &sqlx::PgPool {
        &self.0
    }

    pub async fn begin(&self) -> AppResult<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.0.begin().await.map_err(AppError::TransactionError)
    }
}

pub fn connect_database_with(cfg: &DatabaseConfig) -> ConnectionPool {
    ConnectionPool(sqlx::PgPool::connect_lazy_with(make_pg_connect_options(
        cfg,
    )))
}

pub async fn set_transaction_serializable(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> AppResult<()> {
    sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut **tx)
        .await
        .map_err(AppError::SpecificOperationError)?;
    Ok(())
}
