use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use derive_new::new;
use kernel::model::id::ItemId;
use kernel::model::item::{CreateItem, DeleteItem, ItemCategory, UpdateItem};
use kernel::model::list::{ListOptions, PaginatedList};
use kernel::model::{checkout::SimpleCheckout, item::Item};
use kernel::repository::item::ItemRepository;
use shared::error::{AppError, AppResult};

use crate::database::ConnectionPool;
use crate::database::model::item::{ItemCheckoutRow, ItemRow, PaginatedItemRow};
use crate::database::set_transaction_serializable;

#[derive(new)]
pub struct ItemRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl ItemRepository for ItemRepositoryImpl {
    async fn create(&self, event: CreateItem) -> AppResult<()> {
        let category = event.as_ref();

        let (name, description) = match &event {
            CreateItem::General { name, description }
            | CreateItem::Book {
                name, description, ..
            }
            | CreateItem::Laptop {
                name, description, ..
            } => (name, description),
        };

        let mut tx = self.db.begin().await?;
        set_transaction_serializable(&mut tx).await?;

        let item_id = sqlx::query!(
            r#"
            INSERT INTO items (name, description, category)
            VALUES ($1, $2, $3)
            RETURNING item_id
        "#,
            name,
            description,
            category
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?
        .item_id;

        match &event {
            CreateItem::Book { author, isbn, .. } => {
                sqlx::query!(
                    r#"
                        INSERT INTO books (item_id, author, isbn)
                        VALUES ($1, $2, $3)
                    "#,
                    item_id,
                    author,
                    isbn,
                )
                .execute(&mut *tx)
                .await
                .map_err(AppError::SpecificOperationError)?;
            }
            CreateItem::Laptop { mac_address, .. } => {
                sqlx::query!(
                    r#"
                        INSERT INTO laptops (item_id, mac_address)
                        VALUES ($1, $2)
                    "#,
                    item_id,
                    mac_address,
                )
                .execute(&mut *tx)
                .await
                .map_err(AppError::SpecificOperationError)?;
            }
            _ => {}
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }

    async fn find_all(&self, options: ListOptions) -> AppResult<PaginatedList<Item>> {
        let ListOptions {
            limit,
            offset,
            category,
        } = options;

        let rows: Vec<PaginatedItemRow> = sqlx::query_as!(
            PaginatedItemRow,
            r#"
                SELECT
                    COUNT(*) OVER() AS "total!",
                    i.item_id AS id,
                    i.category AS category
                FROM items AS i
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

        let item_ids = rows
            .into_iter()
            .filter_map(|r| {
                let Some(ref category) = category else {
                    return Some(r.id);
                };
                if ItemCategory::from_str(&r.category).unwrap() == *category {
                    Some(r.id)
                } else {
                    None
                }
            })
            .collect::<Vec<ItemId>>();
        let total = item_ids.len() as i64;
        let mut checkouts = self.find_checkouts(&item_ids).await?;

        let rows: Vec<ItemRow> = sqlx::query_as!(
            ItemRow,
            r#"
                SELECT
                    i.item_id AS item_id,
                    i.category AS category,
                    i.name AS name,
                    i.description AS description,
                    b.author AS "author?",
                    b.isbn AS "isbn?",
                    l.mac_address AS "mac_address?"
                FROM items AS i
                LEFT JOIN books b ON i.item_id = b.item_id
                LEFT JOIN laptops l ON i.item_id = l.item_id
                WHERE i.item_id IN (SELECT * FROM UNNEST($1::uuid[]))
                ORDER BY i.created_at DESC
            "#,
            &item_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let items = rows
            .into_iter()
            .map(|row| {
                let checkout = checkouts.remove(&row.item_id);
                row.into_item(checkout)
            })
            .collect();

        Ok(PaginatedList {
            total,
            limit,
            offset,
            items,
        })
    }

    async fn find_by_id(&self, item_id: ItemId) -> AppResult<Option<Item>> {
        let row: Option<ItemRow> = sqlx::query_as!(
            ItemRow,
            r#"
                SELECT
                    i.item_id AS item_id,
                    i.category AS category,
                    i.name AS name,
                    i.description AS description,
                    b.author AS "author?",
                    b.isbn AS "isbn?",
                    l.mac_address AS "mac_address?"
                FROM items AS i
                LEFT JOIN books b ON i.item_id = b.item_id
                LEFT JOIN laptops l ON i.item_id = l.item_id
                WHERE i.item_id = $1
                ORDER BY i.created_at DESC
            "#,
            item_id.raw()
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        match row {
            Some(row) => {
                let checkout = self.find_checkouts(&[item_id]).await?.remove(&item_id);
                Ok(Some(row.into_item(checkout)))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, event: UpdateItem) -> AppResult<()> {
        let (item_id, name, description) = match &event {
            UpdateItem::General {
                item_id,
                name,
                description,
            }
            | UpdateItem::Book {
                item_id,
                name,
                description,
                ..
            }
            | UpdateItem::Laptop {
                item_id,
                name,
                description,
                ..
            } => (item_id, name, description),
        };

        let mut tx = self.db.begin().await?;
        set_transaction_serializable(&mut tx).await?;

        let res = sqlx::query!(
            r#"
                UPDATE items
                SET
                    name = $1,
                    description = $2
                WHERE item_id = $3
            "#,
            name,
            description,
            item_id.raw(),
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;

        match &event {
            UpdateItem::Book { author, isbn, .. } => {
                sqlx::query!(
                    r#"
                        UPDATE books
                        SET
                            author = $1,
                            isbn = $2
                        WHERE item_id = $3
                    "#,
                    author,
                    isbn,
                    item_id.raw(),
                )
                .execute(&mut *tx)
                .await
                .map_err(AppError::SpecificOperationError)?;
            }
            UpdateItem::Laptop { mac_address, .. } => {
                sqlx::query!(
                    r#"
                        UPDATE laptops
                        SET
                            mac_address = $1
                        WHERE item_id = $2
                    "#,
                    mac_address,
                    item_id.raw(),
                )
                .execute(&mut *tx)
                .await
                .map_err(AppError::SpecificOperationError)?;
            }
            _ => {}
        }

        tx.commit().await.map_err(AppError::TransactionError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified item not found".into()));
        }

        Ok(())
    }

    async fn delete(&self, event: DeleteItem) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM items
                WHERE item_id = $1 AND category = 'general'
            "#,
            event.item_id.raw(),
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified item not found".into()));
        }

        Ok(())
    }
}

impl ItemRepositoryImpl {
    async fn find_checkouts(
        &self,
        item_ids: &[ItemId],
    ) -> AppResult<HashMap<ItemId, SimpleCheckout>> {
        let res = sqlx::query_as!(
            ItemCheckoutRow,
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
            item::general::GeneralItem,
        },
        repository::checkout::CheckoutRepository,
    };
    use mac_address::MacAddress;

    use crate::repository::{checkout::CheckoutRepositoryImpl, item::ItemRepositoryImpl};

    use super::*;

    #[sqlx::test(fixtures("common"))]
    async fn test_register_general_item(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        let original_name = "Test Item".to_string();
        let original_description = "Test Description".to_string();

        let create_event = CreateItem::General {
            name: original_name.clone(),
            description: original_description.clone(),
        };
        repo.create(create_event).await?;

        let options = ListOptions {
            limit: 20,
            offset: 0,
            category: Some(ItemCategory::General),
        };

        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);

        let Item::General(ref item) = res.items[0] else {
            panic!("Expected item to be General");
        };
        let item_id = item.id;
        let res = repo.find_by_id(item_id).await?;
        assert!(res.is_some());
        let Item::General(GeneralItem {
            id,
            name,
            description,
            ..
        }) = res.unwrap()
        else {
            panic!("Expected item to be General");
        };
        assert_eq!(id, item_id);
        assert_eq!(name, original_name);
        assert_eq!(description, original_description);
        Ok(())
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_register_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        let original_name = "Test Book".to_string();
        let original_description = "Test Book Description".to_string();
        let original_author = "Test Author".to_string();
        let original_isbn = "1234567890123".to_string();

        let create_event = CreateItem::Book {
            name: original_name.clone(),
            description: original_description.clone(),
            author: original_author.clone(),
            isbn: original_isbn.clone(),
        };
        repo.create(create_event).await?;

        let options = ListOptions {
            limit: 20,
            offset: 0,
            category: Some(ItemCategory::Book),
        };

        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);

        let Item::Book(ref item) = res.items[0] else {
            panic!("Expected item to be Book");
        };
        let item_id = item.id;
        let res = repo.find_by_id(item_id).await?;
        assert!(res.is_some());
        let Item::Book(book) = res.unwrap() else {
            panic!("Expected item to be Book");
        };
        assert_eq!(book.id, item_id);
        assert_eq!(book.name, original_name);
        assert_eq!(book.description, original_description);
        assert_eq!(book.author, original_author);
        assert_eq!(book.isbn, original_isbn);
        Ok(())
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_register_laptop(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        let original_name = "Test Laptop".to_string();
        let original_description = "Test Laptop Description".to_string();
        let original_mac_address = MacAddress::from_str("00:00:00:00:00:00")?;

        let create_event = CreateItem::Laptop {
            name: original_name.clone(),
            description: original_description.clone(),
            mac_address: original_mac_address,
        };
        repo.create(create_event).await?;

        let options = ListOptions {
            limit: 20,
            offset: 0,
            category: Some(ItemCategory::Laptop),
        };

        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);

        let Item::Laptop(ref item) = res.items[0] else {
            panic!("Expected item to be Laptop");
        };
        let item_id = item.id;
        let res = repo.find_by_id(item_id).await?;
        assert!(res.is_some());
        let Item::Laptop(laptop) = res.unwrap() else {
            panic!("Expected item to be Laptop");
        };
        assert_eq!(laptop.id, item_id);
        assert_eq!(laptop.name, original_name);
        assert_eq!(laptop.description, original_description);
        assert_eq!(laptop.mac_address, original_mac_address);
        Ok(())
    }

    #[sqlx::test(fixtures("common", "item"))]
    async fn test_update_item(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef113")?;
        let item = repo.find_by_id(item_id).await?.unwrap();

        let Item::General(general_item) = item else {
            panic!("Expected item to be General");
        };

        let update_event = UpdateItem::General {
            item_id: general_item.id,
            name: "Updated Name".into(),
            description: "Updated Description".into(),
        };
        repo.update(update_event).await?;

        let updated_item = repo.find_by_id(item_id).await?.unwrap();
        let Item::General(updated_item) = updated_item else {
            panic!("Expected item to be General");
        };
        assert_eq!(updated_item.name, "Updated Name");
        assert_eq!(updated_item.description, "Updated Description");

        Ok(())
    }

    #[sqlx::test(fixtures("common", "item"))]
    async fn test_delete_item(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let item_id = ItemId::from_str("9890736e-a4e4-461a-a77d-eac3517ef113")?;

        let item = repo.find_by_id(item_id).await?;
        assert!(item.is_some());

        repo.delete(DeleteItem { item_id }).await?;
        let item = repo.find_by_id(item_id).await?;
        assert!(item.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures("common", "item_list"))]
    async fn test_list_filters_and_categories(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        // Check total items
        let res = repo
            .find_all(ListOptions {
                limit: 100,
                offset: 0,
                category: None,
            })
            .await?;
        assert_eq!(res.total, 50); // 30 general + 10 books + 10 laptops

        // Test pagination
        let res = repo
            .find_all(ListOptions {
                limit: 10,
                offset: 0,
                category: None,
            })
            .await?;
        assert_eq!(res.limit, 10);
        assert_eq!(res.offset, 0);

        let Item::General(first_item) = &res.items[0] else {
            panic!("Expected first item to be General");
        };
        assert_eq!(first_item.name, "item001"); // Most recent general item

        let res = repo
            .find_all(ListOptions {
                limit: 10,
                offset: 10,
                category: None,
            })
            .await?;
        assert_eq!(res.limit, 10);
        assert_eq!(res.offset, 10);

        let Item::General(first_item) = &res.items[0] else {
            panic!("Expected first item to be General");
        };
        assert_eq!(first_item.name, "item011");

        // Test category filters
        let res = repo
            .find_all(ListOptions {
                limit: 100,
                offset: 0,
                category: Some(ItemCategory::General),
            })
            .await?;
        assert_eq!(res.total, 30);
        assert!(
            res.items
                .iter()
                .all(|item| matches!(item, Item::General(_)))
        );

        let res = repo
            .find_all(ListOptions {
                limit: 100,
                offset: 0,
                category: Some(ItemCategory::Book),
            })
            .await?;
        assert_eq!(res.total, 10);
        assert!(res.items.iter().all(|item| matches!(item, Item::Book(_))));

        let res = repo
            .find_all(ListOptions {
                limit: 100,
                offset: 0,
                category: Some(ItemCategory::Laptop),
            })
            .await?;
        assert_eq!(res.total, 10);
        assert!(res.items.iter().all(|item| matches!(item, Item::Laptop(_))));

        // Test invalid offset
        let res = repo
            .find_all(ListOptions {
                limit: 10,
                offset: 100,
                category: None,
            })
            .await?;
        assert_eq!(res.total, 0);
        assert_eq!(res.items.len(), 0);

        Ok(())
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_error_cases(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        // Test updating non-existent item
        let non_existent_id = ItemId::from_str("00000000-0000-0000-0000-000000000000")?;
        let update_result = repo
            .update(UpdateItem::General {
                item_id: non_existent_id,
                name: "New Name".into(),
                description: "New Description".into(),
            })
            .await;
        assert!(matches!(update_result, Err(AppError::EntityNotFound(_))));

        // Test deleting non-existent item
        let delete_result = repo
            .delete(DeleteItem {
                item_id: non_existent_id,
            })
            .await;
        assert!(matches!(delete_result, Err(AppError::EntityNotFound(_))));

        Ok(())
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_concurrent_item_creation(pool: sqlx::PgPool) -> anyhow::Result<()> {
        use tokio::task;

        let repo1 = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let repo2 = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        // Try to create two items with the same name concurrently
        let create_task1 = task::spawn(async move {
            repo1
                .create(CreateItem::General {
                    name: "Concurrent Item".into(),
                    description: "Description 1".into(),
                })
                .await
        });

        let create_task2 = task::spawn(async move {
            repo2
                .create(CreateItem::General {
                    name: "Concurrent Item".into(),
                    description: "Description 2".into(),
                })
                .await
        });

        let (result1, result2) = tokio::join!(create_task1, create_task2);
        result1??;
        result2??;

        // Both creations should succeed due to serializable transaction level
        let repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let items = repo
            .find_all(ListOptions {
                limit: 10,
                offset: 0,
                category: Some(ItemCategory::General),
            })
            .await?;

        assert_eq!(items.total, 2);
        let names: Vec<_> = items
            .items
            .iter()
            .map(|item| match item {
                Item::General(item) => item.name.clone(),
                _ => panic!("Expected all items to be General"),
            })
            .collect();
        assert!(names.contains(&"Concurrent Item".to_string()));

        Ok(())
    }

    #[sqlx::test(fixtures("common", "item"))]
    async fn test_item_checkout(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let item_repo = ItemRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let checkout_repo = CheckoutRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        let user_id1 = UserId::from_str("9582f9de-0fd1-4892-b20c-70139a7eb95b")?;
        let user_id2 = UserId::from_str("050afe56-c3da-4448-8e4d-6f44007d2ca5")?;

        let res = item_repo
            .find_all(ListOptions {
                limit: 20,
                offset: 0,
                category: None,
            })
            .await?;

        let item = res.items.into_iter().next().unwrap();
        let (item_id, checkout) = match &item {
            Item::General(item) => (item.id, item.checkout.as_ref()),
            Item::Book(item) => (item.id, item.checkout.as_ref()),
            Item::Laptop(item) => (item.id, item.checkout.as_ref()),
        };
        assert!(checkout.is_none());

        {
            checkout_repo
                .create(CreateCheckout {
                    item_id,
                    checked_out_by: user_id1,
                    checked_out_at: Utc::now(),
                })
                .await?;

            let item_co = item_repo.find_by_id(item_id).await?.unwrap();
            let checkout = match &item_co {
                Item::General(item) => item.checkout.as_ref(),
                Item::Book(item) => item.checkout.as_ref(),
                Item::Laptop(item) => item.checkout.as_ref(),
            };
            assert!(checkout.is_some());
            let co = checkout.unwrap();
            assert_eq!(co.checked_out_by.id, user_id1);

            checkout_repo
                .update_returned(UpdateReturned {
                    checkout_id: co.checkout_id,
                    item_id,
                    returned_by: user_id1,
                    returned_at: Utc::now(),
                })
                .await?;

            let item_re = item_repo.find_by_id(item_id).await?.unwrap();
            let checkout = match &item_re {
                Item::General(item) => item.checkout.as_ref(),
                Item::Book(item) => item.checkout.as_ref(),
                Item::Laptop(item) => item.checkout.as_ref(),
            };
            assert!(checkout.is_none());
        }

        {
            checkout_repo
                .create(CreateCheckout {
                    item_id,
                    checked_out_by: user_id2,
                    checked_out_at: Utc::now(),
                })
                .await?;

            let item_co = item_repo.find_by_id(item_id).await?.unwrap();
            let checkout = match &item_co {
                Item::General(item) => item.checkout.as_ref(),
                Item::Book(item) => item.checkout.as_ref(),
                Item::Laptop(item) => item.checkout.as_ref(),
            };
            assert!(checkout.is_some());
            let co = checkout.unwrap();
            assert_eq!(co.checked_out_by.id, user_id2);

            checkout_repo
                .update_returned(UpdateReturned {
                    checkout_id: co.checkout_id,
                    item_id,
                    returned_by: user_id2,
                    returned_at: Utc::now(),
                })
                .await?;

            let item_re = item_repo.find_by_id(item_id).await?.unwrap();
            let checkout = match &item_re {
                Item::General(item) => item.checkout.as_ref(),
                Item::Book(item) => item.checkout.as_ref(),
                Item::Laptop(item) => item.checkout.as_ref(),
            };
            assert!(checkout.is_none());
        }

        Ok(())
    }
}
