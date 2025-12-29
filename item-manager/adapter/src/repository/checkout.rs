use async_trait::async_trait;
use derive_new::new;
use kernel::model::checkout::{
    Checkout,
    event::{CreateCheckout, UpdateReturned},
};
use kernel::model::id::{CheckoutId, ItemId, UserId};
use kernel::model::role::Role;
use kernel::repository::checkout::CheckoutRepository;
use shared::error::{AppError, AppResult};

use crate::database::{
    ConnectionPool,
    model::checkout::{CheckoutRow, CheckoutStateRow, ReturnedCheckoutRow},
    set_transaction_serializable,
};

#[derive(new)]
pub struct CheckoutRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CheckoutRepository for CheckoutRepositoryImpl {
    async fn create(&self, event: CreateCheckout) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        set_transaction_serializable(&mut tx).await?;

        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    i.item_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    NULL AS "user_id?: UserId"
                    FROM items AS i
                    LEFT OUTER JOIN checkouts AS c USING(item_id)
                    WHERE item_id = $1;
                "#,
                event.item_id.raw()
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                Some(CheckoutStateRow {
                    checkout_id: Some(_),
                    ..
                }) => {
                    return Err(AppError::Conflict(format!(
                        "The item ({}) has already been checked out.",
                        event.item_id
                    )));
                }
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "Item ({}) not found.",
                        event.item_id
                    )));
                }
                _ => {}
            }
        }

        let res = sqlx::query!(
            r#"
                INSERT INTO checkouts
                (item_id, user_id, checked_out_at)
                VALUES ($1, $2, $3)
                ;
            "#,
            event.item_id.raw(),
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

        set_transaction_serializable(&mut tx).await?;

        {
            let res = sqlx::query_as!(
                CheckoutStateRow,
                r#"
                    SELECT
                    c.item_id,
                    c.checkout_id AS "checkout_id?: CheckoutId",
                    c.user_id AS "user_id?: UserId"
                    FROM checkouts AS c
                    WHERE c.checkout_id = $1
                    AND c.item_id = $2;
                "#,
                event.checkout_id.raw(),
                event.item_id.raw(),
            )
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::SpecificOperationError)?;

            match res {
                Some(CheckoutStateRow {
                    checkout_id: Some(_c),
                    user_id: Some(u),
                    ..
                }) if u != event.returned_by && event.returned_by_role != Role::Admin => {
                    return Err(AppError::ForbiddenOperation(format!(
                        "Designated checkout (id({}), users({}), items({})) cannot be returned by non-admin user",
                        event.checkout_id, event.returned_by, event.item_id
                    )));
                }
                None => {
                    return Err(AppError::EntityNotFound(format!(
                        "Checkout ({}) for item ({}) not found.",
                        event.checkout_id, event.item_id
                    )));
                }
                _ => {}
            }
        }

        let res = sqlx::query!(
            r#"
                INSERT INTO returned_checkouts
                (checkout_id, item_id, user_id, checked_out_at, returned_at)
                SELECT checkout_id, item_id, user_id, checked_out_at, $2
                FROM checkouts
                WHERE checkout_id = $1
                  AND item_id = $3
                ;
            "#,
            event.checkout_id.raw(),
            event.returned_at,
            event.item_id.raw(),
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
                DELETE FROM checkouts WHERE checkout_id = $1 AND item_id = $2;
            "#,
            event.checkout_id.raw(),
            event.item_id.raw(),
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
                c.item_id,
                c.user_id,
                c.checked_out_at
                FROM checkouts AS c
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
                c.item_id,
                c.user_id,
                c.checked_out_at
                FROM checkouts AS c
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

    async fn find_history_by_item_id(&self, item_id: ItemId) -> AppResult<Vec<Checkout>> {
        let checkout: Option<Checkout> = self.find_unreturned_by_item_id(item_id).await?;
        let mut checkout_histories: Vec<Checkout> = sqlx::query_as!(
            ReturnedCheckoutRow,
            r#"
                SELECT
                rc.checkout_id,
                rc.item_id,
                rc.user_id,
                rc.checked_out_at,
                rc.returned_at
                FROM returned_checkouts AS rc
                WHERE rc.item_id = $1
                ORDER BY rc.checked_out_at DESC
            "#,
            item_id.raw()
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
    async fn find_unreturned_by_item_id(&self, item_id: ItemId) -> AppResult<Option<Checkout>> {
        let res = sqlx::query_as!(
            CheckoutRow,
            r#"
                SELECT
                c.checkout_id,
                c.item_id,
                c.user_id,
                c.checked_out_at
                FROM checkouts AS c
                WHERE c.item_id = $1
            "#,
            item_id.raw(),
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

    #[sqlx::test(fixtures("common", "item"))]
    async fn test_checkout_flow(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));

        // Test basic checkout flow
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef113")?;
        let user_id = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let checkout_time = Utc::now();

        // Create checkout
        let event = CreateCheckout {
            item_id,
            checked_out_by: user_id,
            checked_out_at: checkout_time,
        };
        repo.create(event).await?;

        // Verify unreturned checkout exists
        // Test initial checkout state
        let unreturned = repo.find_unreturned_by_user_id(user_id).await?;
        assert_eq!(unreturned.len(), 1);
        let checkout = &unreturned[0];
        assert_eq!(checkout.item_id, item_id);
        assert_eq!(checkout.checked_out_by, user_id);
        assert!(checkout.returned_at.is_none());

        // Test initial checkout history
        let history = repo.find_history_by_item_id(item_id).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].item_id, item_id);
        assert!(history[0].returned_at.is_none());

        // Return the item
        let return_time = Utc::now();
        let event = UpdateReturned {
            checkout_id: unreturned[0].id,
            item_id,
            returned_by: user_id,
            returned_by_role: Role::User,
            returned_at: return_time,
        };
        repo.update_returned(event).await?;

        // Verify checkout is now in history
        let history = repo.find_history_by_item_id(item_id).await?;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].item_id, item_id);

        // Verify no unreturned checkouts exist
        let unreturned = repo.find_unreturned_by_user_id(user_id).await?;
        assert_eq!(unreturned.len(), 0);

        Ok(())
    }

    #[sqlx::test(fixtures("common", "item"))]
    async fn test_checkout_errors(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef113")?;
        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5")?;
        let checkout_time = Utc::now();

        // First checkout
        let event = CreateCheckout {
            item_id,
            checked_out_by: user_id1,
            checked_out_at: checkout_time,
        };
        repo.create(event).await?;

        // Test duplicate checkout
        let event = CreateCheckout {
            item_id,
            checked_out_by: user_id2,
            checked_out_at: checkout_time,
        };
        assert!(repo.create(event).await.is_err());

        // Test non-existent item checkout
        let non_existent_item_id = ItemId::new();
        let event = CreateCheckout {
            item_id: non_existent_item_id,
            checked_out_by: user_id1,
            checked_out_at: checkout_time,
        };
        assert!(repo.create(event).await.is_err());

        // Test incorrect user return (non-admin)
        let unreturned = repo.find_unreturned_by_user_id(user_id1).await?;
        let event = UpdateReturned {
            checkout_id: unreturned[0].id,
            item_id,
            returned_by: user_id2,        // Wrong user
            returned_by_role: Role::User, // Non-admin user
            returned_at: Utc::now(),
        };
        assert!(repo.update_returned(event).await.is_err());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "item_list"))]
    async fn test_return_with_mismatched_item_id_is_rejected(
        pool: sqlx::PgPool,
    ) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let admin_user_id = UserId::from_str("5b4c96ac-316a-4bee-8e69-cac5eb84ff4c")?;

        let rows = sqlx::query!(
            r#"
                SELECT item_id
                FROM items
                ORDER BY created_at ASC
                LIMIT 2;
            "#
        )
        .fetch_all(&pool)
        .await?;

        let item_id1 = ItemId::from(rows[0].item_id);
        let item_id2 = ItemId::from(rows[1].item_id);

        let event = CreateCheckout {
            item_id: item_id1,
            checked_out_by: admin_user_id,
            checked_out_at: Utc::now(),
        };
        repo.create(event).await?;

        let unreturned = repo.find_unreturned_by_user_id(admin_user_id).await?;
        assert_eq!(unreturned.len(), 1);

        let event = UpdateReturned {
            checkout_id: unreturned[0].id,
            item_id: item_id2,
            returned_by: admin_user_id,
            returned_by_role: Role::Admin,
            returned_at: Utc::now(),
        };
        assert!(repo.update_returned(event).await.is_err());

        let unreturned_after = repo.find_unreturned_by_user_id(admin_user_id).await?;
        assert_eq!(unreturned_after.len(), 1);
        assert_eq!(unreturned_after[0].item_id, item_id1);

        Ok(())
    }

    #[sqlx::test(fixtures("common", "item"))]
    async fn test_admin_can_return_any_item(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef113")?;
        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?; // Regular user who checks out
        let admin_user_id = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5")?; // Admin user who returns
        let checkout_time = Utc::now();

        // Regular user checks out item
        let event = CreateCheckout {
            item_id,
            checked_out_by: user_id1,
            checked_out_at: checkout_time,
        };
        repo.create(event).await?;

        // Verify checkout exists
        let unreturned = repo.find_unreturned_by_user_id(user_id1).await?;
        assert_eq!(unreturned.len(), 1);
        let checkout = &unreturned[0];

        // Admin returns the item (different user than who checked it out)
        let event = UpdateReturned {
            checkout_id: checkout.id,
            item_id,
            returned_by: admin_user_id,    // Different user (admin)
            returned_by_role: Role::Admin, // Admin role
            returned_at: Utc::now(),
        };

        // This should succeed because admin can return any item
        repo.update_returned(event).await?;

        // Verify the item is now returned
        let unreturned_after = repo.find_unreturned_by_user_id(user_id1).await?;
        assert_eq!(unreturned_after.len(), 0);

        // Verify it appears in history
        let history = repo.find_history_by_item_id(item_id).await?;
        assert_eq!(history.len(), 1);
        assert!(history[0].returned_at.is_some());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "item"))]
    async fn test_find_operations(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool));
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef113")?;
        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5")?;
        let checkout_time = Utc::now();

        // Create checkouts
        let event = CreateCheckout {
            item_id,
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

        // Return the item
        let event = UpdateReturned {
            checkout_id: all_unreturned[0].id,
            item_id,
            returned_by: user_id1,
            returned_by_role: Role::User,
            returned_at: Utc::now(),
        };
        repo.update_returned(event).await?;

        // Test multiple checkout history
        // Create another checkout
        let event = CreateCheckout {
            item_id,
            checked_out_by: user_id1,
            checked_out_at: Utc::now(),
        };
        repo.create(event).await?;

        let second_checkout = repo.find_unreturned_by_user_id(user_id1).await?[0].clone();

        // Return second checkout
        let event = UpdateReturned {
            checkout_id: second_checkout.id,
            item_id,
            returned_by: user_id1,
            returned_by_role: Role::User,
            returned_at: Utc::now(),
        };
        repo.update_returned(event).await?;

        // Verify history contains both checkouts in correct order
        let history = repo.find_history_by_item_id(item_id).await?;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].item_id, item_id); // Most recent first
        assert_eq!(history[1].item_id, item_id);
        assert!(history[0].returned_at.is_some());
        assert!(history[1].returned_at.is_some());

        Ok(())
    }
}
