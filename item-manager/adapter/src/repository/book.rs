use std::collections::HashMap;

use async_trait::async_trait;
use derive_new::new;
use kernel::model::{
    book::{Checkout, event::DeleteBook},
    id::BookId,
    list::PaginatedList,
};
use kernel::{
    model::book::{
        Book, BookListOptions,
        event::{CreateBook, UpdateBook},
    },
    repository::book::BookRepository,
};
use shared::error::{AppError, AppResult};

use crate::database::model::book::{BookRow, PaginatedBookRow};
use crate::database::{ConnectionPool, model::book::BookCheckoutRow};

#[derive(new)]
pub struct BookRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl BookRepository for BookRepositoryImpl {
    async fn create(&self, event: CreateBook) -> AppResult<()> {
        sqlx::query!(
            r#"
                INSERT INTO books (title, author, isbn, description)
                VALUES ($1, $2, $3, $4)
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    async fn find_all(&self, options: BookListOptions) -> AppResult<PaginatedList<Book>> {
        let BookListOptions { limit, offset } = options;
        let rows: Vec<PaginatedBookRow> = sqlx::query_as!(
            PaginatedBookRow,
            r#"
                SELECT
                COUNT(*) OVER() AS "total!",
                b.book_id AS id
                FROM books AS b
                ORDER BY b.created_at DESC
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
        let book_ids = rows.into_iter().map(|r| r.id).collect::<Vec<BookId>>();

        let rows: Vec<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description
                FROM books AS b
                WHERE b.book_id IN (SELECT * FROM UNNEST($1::uuid[]))
                ORDER BY b.created_at DESC
            "#,
            &book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let book_ids = rows.iter().map(|r| r.book_id).collect::<Vec<BookId>>();
        let mut checkouts = self.find_checkouts(&book_ids).await?;

        let items = rows
            .into_iter()
            .map(|row| {
                let checkout = checkouts.remove(&row.book_id);
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

    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>> {
        let row: Option<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
                SELECT
                    b.book_id AS book_id,
                    b.title AS title,
                    b.author AS author,
                    b.isbn AS isbn,
                    b.description AS description
                FROM books AS b
                WHERE b.book_id = $1
            "#,
            book_id.raw()
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        match row {
            Some(row) => {
                let checkout = self.find_checkouts(&[book_id]).await?.remove(&book_id);
                Ok(Some(row.into_book(checkout)))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, event: UpdateBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE books
                SET
                    title = $1,
                    author = $2,
                    isbn = $3,
                    description = $4
                WHERE book_id = $5
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            event.book_id.raw(),
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified book not found".into()));
        }

        Ok(())
    }

    async fn delete(&self, event: DeleteBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM books
                WHERE book_id = $1
            "#,
            event.book_id.raw(),
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
    async fn find_checkouts(&self, book_ids: &[BookId]) -> AppResult<HashMap<BookId, Checkout>> {
        let res = sqlx::query_as!(
            BookCheckoutRow,
            r#"
                SELECT
                    c.checkout_id,
                    c.book_id,
                    u.user_id,
                    u.name AS user_name,
                    c.checked_out_at
                FROM checkouts AS c
                INNER JOIN users AS u USING(user_id)
                WHERE book_id = ANY($1)
                ;
            "#,
            book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(|row| (row.book_id, Checkout::from(row)))
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
            title: "Test Title".into(),
            author: "Test Author".into(),
            isbn: "Test ISBN".into(),
            description: "Test Description".into(),
        };
        repo.create(book).await?;
        let options = BookListOptions {
            limit: 20,
            offset: 0,
        };
        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);
        let book_id = res.items[0].id;
        let res = repo.find_by_id(book_id).await?;
        assert!(res.is_some());
        let Book {
            id,
            title,
            author,
            isbn,
            description,
            ..
        } = res.unwrap();
        assert_eq!(id, book_id);
        assert_eq!(title, "Test Title");
        assert_eq!(author, "Test Author");
        assert_eq!(isbn, "Test ISBN");
        assert_eq!(description, "Test Description");
        Ok(())
    }

    #[sqlx::test(fixtures("common", "book"))]
    async fn test_update_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let book_id = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b").unwrap();
        let book = repo.find_by_id(book_id).await?.unwrap();
        const NEW_AUTHOR: &str = "Updated Author";
        assert_ne!(book.author, NEW_AUTHOR);

        let update_book = UpdateBook {
            book_id: book.id,
            title: book.title,
            author: NEW_AUTHOR.into(), // This is the difference
            isbn: book.isbn,
            description: book.description,
        };
        repo.update(update_book).await.unwrap();

        let book = repo.find_by_id(book_id).await?.unwrap();
        assert_eq!(book.author, NEW_AUTHOR);

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book"))]
    async fn test_delete_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let book_id = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b")?;

        repo.delete(DeleteBook { book_id }).await?;
        let book = repo.find_by_id(book_id).await?;

        assert!(book.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "book_list"))]
    async fn test_list_filters(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        const LEN: i64 = 50; // 50 is the number of records of fixtures "book_list"

        let res = repo
            .find_all(BookListOptions {
                limit: 10,
                offset: 0,
            })
            .await?;
        assert_eq!(res.total, LEN);
        assert_eq!(res.limit, 10);
        assert_eq!(res.offset, 0);
        assert_eq!(res.items[0].title, "title050");

        let res = repo
            .find_all(BookListOptions {
                limit: 10,
                offset: 10,
            })
            .await?;
        assert_eq!(res.total, LEN);
        assert_eq!(res.limit, 10);
        assert_eq!(res.offset, 10);
        assert_eq!(res.items[0].title, "title040");

        let res = repo
            .find_all(BookListOptions {
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
            .find_all(BookListOptions {
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
                    book_id: book.id,
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
                    book_id: book_co.id,
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
                    book_id: book.id,
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
                    book_id: book_co.id,
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
