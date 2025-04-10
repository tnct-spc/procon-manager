use derive_new::new;
use shared::error::{AppError, AppResult};

pub mod model;

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

pub async fn set_transaction_serializable(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> AppResult<()> {
    sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut **tx)
        .await
        .map_err(AppError::SpecificOperationError)?;
    Ok(())
}
