use std::collections::HashMap;

use async_trait::async_trait;
use derive_new::new;
use kernel::model::id::ItemId;
use kernel::model::list::{ListOptions, PaginatedList};
use kernel::model::{
    book::{
        Book,
        event::{CreateBook, DeleteBook, UpdateBook},
    },
    checkout::SimpleCheckout,
};
use kernel::repository::item::CommonItemRepository;
use shared::error::{AppError, AppResult};

use crate::database::model::book::{BookRow, PaginatedBookRow};
use crate::database::set_transaction_serializable;
use crate::database::{ConnectionPool, model::book::BookCheckoutRow};

#[derive(new)]
pub struct BookRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl CommonItemRepository<Book, ItemId, CreateBook, UpdateBook, DeleteBook> for BookRepositoryImpl {
    async fn create(&self, event: CreateBook) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        set_transaction_serializable(&mut tx).await?;

        let item_id = sqlx::query!(
            r#"
                INSERT INTO items (name, description, category)
                VALUES ($1, $2, 'book')
                RETURNING item_id
            "#,
            event.name,
            event.description,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?
        .item_id;

        eprintln!("item_id: {:?}", item_id);

        sqlx::query!(
            r#"
                INSERT INTO books (item_id, author, isbn)
                VALUES ($1, $2, $3)
            "#,
            item_id,
            event.author,
            event.isbn,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn find_all(&self, options: ListOptions) -> AppResult<PaginatedList<Book>> {
        let ListOptions { limit, offset } = options;
        let rows: Vec<PaginatedBookRow> = sqlx::query_as!(
            PaginatedBookRow,
            r#"
                SELECT
                COUNT(*) OVER() AS "total!",
                b.item_id AS id
                FROM books AS b
                JOIN items AS i ON b.item_id = i.item_id
                ORDER BY i.created_at DESC
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let total = rows.first().map(|r| r.total).unwrap_or_default(); // If there are no records, then total is also 0.
        let item_ids = rows.into_iter().map(|r| r.id).collect::<Vec<ItemId>>();

        let rows: Vec<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT
                    b.item_id AS item_id,
                    i.name AS name,
                    b.author AS author,
                    b.isbn AS isbn,
                    i.description AS description
                FROM books AS b
                JOIN items AS i ON b.item_id = i.item_id
                WHERE b.item_id IN (SELECT * FROM UNNEST($1::uuid[]))
                ORDER BY i.created_at DESC
            "#,
            &item_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let item_ids = rows.iter().map(|r| r.item_id).collect::<Vec<ItemId>>();
        let mut checkouts = self.find_checkouts(&item_ids).await?;

        let items = rows
            .into_iter()
            .map(|row| {
                let checkout = checkouts.remove(&row.item_id);
                row.into_book(checkout)
            })
            .collect();

        Ok(PaginatedList {
            total,
            limit,
            offset,
            items,
        })
    }

    async fn find_by_id(&self, item_id: ItemId) -> AppResult<Option<Book>> {
        let row: Option<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT
                    b.item_id AS item_id,
                    i.name AS name,
                    b.author AS author,
                    b.isbn AS isbn,
                    i.description AS description
                FROM books AS b
                JOIN items AS i ON b.item_id = i.item_id
                WHERE b.item_id = $1
            "#,
            item_id.raw()
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        match row {
            Some(row) => {
                let checkout = self.find_checkouts(&[item_id]).await?.remove(&item_id);
                Ok(Some(row.into_book(checkout)))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, event: UpdateBook) -> AppResult<()> {
        let mut tx = self.db.begin().await?;

        set_transaction_serializable(&mut tx).await?;

        sqlx::query!(
            r#"
                UPDATE items
                SET
                    name = $1,
                    description = $2
                WHERE item_id = $3
            "#,
            event.name,
            event.description,
            event.item_id.raw(),
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        let res = sqlx::query!(
            r#"
                UPDATE books
                SET
                    author = $1,
                    isbn = $2
                WHERE item_id = $3
            "#,
            event.author,
            event.isbn,
            event.item_id.raw(),
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        tx.commit().await.map_err(AppError::TransactionError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified book not found".into()));
        }

        Ok(())
    }

    async fn delete(&self, event: DeleteBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM items
                WHERE item_id = $1
            "#,
            event.item_id.raw(),
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified book not found".into()));
        }

        Ok(())
    }
}

impl BookRepositoryImpl {
    async fn find_checkouts(
        &self,
        item_ids: &[ItemId],
    ) -> AppResult<HashMap<ItemId, SimpleCheckout>> {
        let res = sqlx::query_as!(
            BookCheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.item_id,
                    u.user_id,
                    u.name AS user_name,
                    c.checked_out_at
                FROM checkouts AS c
                INNER JOIN users AS u USING(user_id)
                WHERE item_id = ANY($1)
                ;
            "#,
            item_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(|row| (row.item_id, SimpleCheckout::from(row)))
        .collect();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Utc;
    use kernel::{
        model::{
            checkout::event::{CreateCheckout, UpdateReturned},
            id::UserId,
        },
        repository::checkout::CheckoutRepository,
    };

    use crate::repository::{book::BookRepositoryImpl, checkout::CheckoutRepositoryImpl};

    use super::*;

    #[sqlx::test(fixtures("common"))]
    async fn test_register_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let book = CreateBook {
            name: "Test Title".into(),
            author: "Test Author".into(),
            isbn: "Test ISBN".into(),
            description: "Test Description".into(),
        };
        repo.create(book).await?;
        let options = ListOptions {
            limit: 20,
            offset: 0,
        };
        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);
        let item_id = res.items[0].id;
        let res = repo.find_by_id(item_id).await?;
        assert!(res.is_some());
        let Book {
            id,
            name,
            author,
            isbn,
            description,
            ..
        } = res.unwrap();
        assert_eq!(id, item_id);
        assert_eq!(name, "Test Title");
        assert_eq!(author, "Test Author");
        assert_eq!(isbn, "Test ISBN");
        assert_eq!(description, "Test Description");
        Ok(())
    }

    #[sqlx::test(fixtures("common", "book"))]
    async fn test_update_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b").unwrap();
        let book = repo.find_by_id(item_id).await?.unwrap();
        const NEW_AUTHOR: &str = "Updated Author";
        assert_ne!(book.author, NEW_AUTHOR);

        let update_book = UpdateBook {
            item_id: book.id,
            name: book.name,
            author: NEW_AUTHOR.into(), // This is the difference
            isbn: book.isbn,
            description: book.description,
        };
        repo.update(update_book).await.unwrap();

        let book = repo.find_by_id(item_id).await?.unwrap();
        assert_eq!(book.author, NEW_AUTHOR);

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book"))]
    async fn test_delete_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b")?;

        repo.delete(DeleteBook { item_id }).await?;
        let book = repo.find_by_id(item_id).await?;

        assert!(book.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book_list"))]
    async fn test_list_filters(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        const LEN: i64 = 50; // 50 is the number of records of fixtures "book_list"

        let res = repo
            .find_all(ListOptions {
                limit: 10,
                offset: 0,
            })
            .await?;
        assert_eq!(res.total, LEN);
        assert_eq!(res.limit, 10);
        assert_eq!(res.offset, 0);
        assert_eq!(res.items[0].name, "title050");

        let res = repo
            .find_all(ListOptions {
                limit: 10,
                offset: 10,
            })
            .await?;
        assert_eq!(res.total, LEN);
        assert_eq!(res.limit, 10);
        assert_eq!(res.offset, 10);
        assert_eq!(res.items[0].name, "title040");

        let res = repo
            .find_all(ListOptions {
                limit: 10,
                offset: 100,
            })
            .await?;
        assert_eq!(res.total, 0); // If offset exceeds total, it becomes 0.
        assert_eq!(res.limit, 10);
        assert_eq!(res.offset, 100);
        assert_eq!(res.items.len(), 0);

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book_checkout"))]
    async fn test_book_checkout(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let book_repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let checkout_repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        // fixtures/book_checkout.sql
        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b").unwrap();
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5").unwrap();

        let book = book_repo
            .find_all(ListOptions {
                limit: 20,
                offset: 0,
            })
            .await?
            .into_inner()
            .pop()
            .unwrap();

        assert!(book.checkout.is_none());

        {
            checkout_repo
                .create(CreateCheckout {
                    item_id: book.id,
                    checked_out_by: user_id1,
                    checked_out_at: Utc::now(),
                })
                .await?;

            let book_co = book_repo.find_by_id(book.id).await?.unwrap();
            assert!(book_co.checkout.is_some());
            let co = book_co.checkout.unwrap();
            assert_eq!(co.checked_out_by.id, user_id1);

            checkout_repo
                .update_returned(UpdateReturned {
                    checkout_id: co.checkout_id,
                    item_id: book_co.id,
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await?;

            let book_re = book_repo.find_by_id(book.id).await?.unwrap();
            assert!(book_re.checkout.is_none());
        }

        {
            checkout_repo
                .create(CreateCheckout {
                    item_id: book.id,
                    checked_out_by: user_id2,
                    checked_out_at: Utc::now(),
                })
                .await?;

            let book_co = book_repo.find_by_id(book.id).await?.unwrap();
            assert!(book_co.checkout.is_some());
            let co = book_co.checkout.unwrap();
            assert_eq!(co.checked_out_by.id, user_id2);

            checkout_repo
                .update_returned(UpdateReturned {
                    checkout_id: co.checkout_id,
                    item_id: book_co.id,
                    returned_by: user_id2,
                    returned_at: Utc::now(),
                })
                .await?;

            let book_re = book_repo.find_by_id(book.id).await?.unwrap();
            assert!(book_re.checkout.is_none());
        }

        Ok(())
    }
}
