use async_trait::async_trait;
use derive_new::new;
use kernel::model::checkout::{
    Checkout,
    event::{CreateCheckout, UpdateReturned},
};
use kernel::model::id::{BookId, CheckoutId, UserId};
use kernel::repository::checkout::CheckoutRepository;
use shared::error::{AppError, AppResult};

use crate::database::{
    ConnectionPool,
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
};

#[derive(new)]
pub struct CheckoutRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CheckoutRepository for CheckoutRepositoryImpl {
    async fn create(&self, event: CreateCheckout) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        self.set_transaction_serializable(&mut tx).await?;

        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    b.book_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    NULL AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id.raw()
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                Some(CheckoutStateRow {
                    checkout_id: Some(_),
                    ..
                }) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "The book ({}) has already been checked out.",
                        event.book_id
                    )));
                }
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "Book ({}) not found.",
                        event.book_id
                    )));
                }
                _ => {}
            }
        }

        let checkout_id = CheckoutId::new();
        let res = sqlx::query!(
            r#"
                INSERT INTO checkouts
                (checkout_id, book_id, user_id, checked_out_at)
                VALUES ($1, $2, $3, $4)
                ;
            "#,
            checkout_id.raw(),
            event.book_id.raw(),
            event.checked_out_by.raw(),
            event.checked_out_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been created".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        self.set_transaction_serializable(&mut tx).await?;

        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    b.book_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    c.user_id AS "user_id?: UserId"
                    FROM books AS b
                    LEFT OUTER JOIN checkouts AS c USING(book_id)
                    WHERE book_id = $1;
                "#,
                event.book_id.raw(),
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                Some(CheckoutStateRow {
                    checkout_id: Some(c),
                    user_id: Some(u),
                    ..
                }) if (c, u) != (event.checkout_id, event.returned_by) => {
                    return Err(AppError::UnprocessableEntity(format!(
                        "Designated checkout (id({}), users({}), books({})) cannot be returned",
                        event.checkout_id, event.returned_by, event.book_id
                    )));
                }
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "Book ({}) not found.",
                        event.book_id
                    )));
                }
                _ => {}
            }
        }

        let res = sqlx::query!(
            r#"
                INSERT INTO returned_checkouts
                (checkout_id, book_id, user_id, checked_out_at, returned_at)
                SELECT checkout_id, book_id, user_id, checked_out_at, $2
                FROM checkouts
                WHERE checkout_id = $1
                ;
            "#,
            event.checkout_id.raw(),
            event.returned_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No returning record has been updated".into(),
            ));
        }

        let res = sqlx::query!(
            r#"
                DELETE FROM checkouts WHERE checkout_id = $1;
            "#,
            event.checkout_id.raw(),
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No checkout record has been deleted".into(),
            ));
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>> {
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                ORDER BY c.checked_out_at ASC
                ;
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>> {
        sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.user_id = $1
                ORDER BY c.checked_out_at ASC
                ;
            "#,
            user_id.raw()
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map(|rows| rows.into_iter().map(Checkout::from).collect())
        .map_err(AppError::SpecificOperationError)
    }

    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>> {
        let checkout: Option<Checkout> = self.find_unreturned_by_book_id(book_id).await?;
        let mut checkout_histories: Vec<Checkout> = sqlx::query_as!(
            ReturnedCheckoutRow,
            r#"
                SELECT
                rc.checkout_id,
                rc.book_id,
                rc.user_id,
                rc.checked_out_at,
                rc.returned_at,
                b.title,
                b.author,
                b.isbn
                FROM returned_checkouts AS rc
                INNER JOIN books AS b USING(book_id)
                WHERE rc.book_id = $1
                ORDER BY rc.checked_out_at DESC
            "#,
            book_id.raw()
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(Checkout::from)
        .collect();

        if let Some(co) = checkout {
            checkout_histories.insert(0, co);
        }

        Ok(checkout_histories)
    }
}

impl CheckoutRepositoryImpl {
    async fn set_transaction_serializable(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> AppResult<()> {
        sqlx::query!("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut **tx)
            .await
            .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    async fn find_unreturned_by_book_id(&self, book_id: BookId) -> AppResult<Option<Checkout>> {
        let res = sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.book_id,
                c.user_id,
                c.checked_out_at,
                b.title,
                b.author,
                b.isbn
                FROM checkouts AS c
                INNER JOIN books AS b USING(book_id)
                WHERE c.book_id = $1
            "#,
            book_id.raw(),
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .map(Checkout::from);

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Utc;

    use super::*;

    #[sqlx::test(fixtures("common", "book_checkout"))]
    async fn test_checkout_flow(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));

        // Test basic checkout flow
        let book_id = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b")?;
        let user_id = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let checkout_time = Utc::now();

        // Create checkout
        let event = CreateCheckout {
            book_id,
            checked_out_by: user_id,
            checked_out_at: checkout_time,
        };
        repo.create(event).await?;

        // Verify unreturned checkout exists
        let unreturned = repo.find_unreturned_by_user_id(user_id).await?;
        assert_eq!(unreturned.len(), 1);
        assert_eq!(unreturned[0].book.book_id, book_id);
        assert_eq!(unreturned[0].checked_out_by, user_id);

        // Test checkout history
        let history = repo.find_history_by_book_id(book_id).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].book.book_id, book_id);

        // Return the book
        let return_time = Utc::now();
        let event = UpdateReturned {
            checkout_id: unreturned[0].id,
            book_id,
            returned_by: user_id,
            returned_at: return_time,
        };
        repo.update_returned(event).await?;

        // Verify checkout is now in history
        let history = repo.find_history_by_book_id(book_id).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].book.book_id, book_id);

        // Verify no unreturned checkouts exist
        let unreturned = repo.find_unreturned_by_user_id(user_id).await?;
        assert_eq!(unreturned.len(), 0);

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book_checkout"))]
    async fn test_checkout_errors(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let book_id = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b")?;
        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5")?;
        let checkout_time = Utc::now();

        // First checkout
        let event = CreateCheckout {
            book_id,
            checked_out_by: user_id1,
            checked_out_at: checkout_time,
        };
        repo.create(event).await?;

        // Test duplicate checkout
        let event = CreateCheckout {
            book_id,
            checked_out_by: user_id2,
            checked_out_at: checkout_time,
        };
        assert!(repo.create(event).await.is_err());

        // Test non-existent book checkout
        let non_existent_book_id = BookId::new();
        let event = CreateCheckout {
            book_id: non_existent_book_id,
            checked_out_by: user_id1,
            checked_out_at: checkout_time,
        };
        assert!(repo.create(event).await.is_err());

        // Test incorrect user return
        let unreturned = repo.find_unreturned_by_user_id(user_id1).await?;
        let event = UpdateReturned {
            checkout_id: unreturned[0].id,
            book_id,
            returned_by: user_id2, // Wrong user
            returned_at: Utc::now(),
        };
        assert!(repo.update_returned(event).await.is_err());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book_checkout"))]
    async fn test_find_operations(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let book_id = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b")?;
        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5")?;
        let checkout_time = Utc::now();

        // Create checkouts
        let event = CreateCheckout {
            book_id,
            checked_out_by: user_id1,
            checked_out_at: checkout_time,
        };
        repo.create(event).await?;

        // Test find_unreturned_all
        let all_unreturned = repo.find_unreturned_all().await?;
        assert_eq!(all_unreturned.len(), 1);

        // Test find_unreturned_by_user_id
        let user1_unreturned = repo.find_unreturned_by_user_id(user_id1).await?;
        assert_eq!(user1_unreturned.len(), 1);
        let user2_unreturned = repo.find_unreturned_by_user_id(user_id2).await?;
        assert_eq!(user2_unreturned.len(), 0);

        // Return the book
        let event = UpdateReturned {
            checkout_id: all_unreturned[0].id,
            book_id,
            returned_by: user_id1,
            returned_at: Utc::now(),
        };
        repo.update_returned(event).await?;

        // Test history
        let history = repo.find_history_by_book_id(book_id).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].book.book_id, book_id);

        Ok(())
    }
}
