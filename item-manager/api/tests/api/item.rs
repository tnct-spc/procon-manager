use std::sync::Arc;

use api::model::{
    checkout::CheckoutsResponse,
    item::{CreateItemRequest, ItemResponse, PaginatedItemResponse, UpdateItemRequest},
};
use axum::{body::Body, http::Request};
use kernel::{
    model::{
        checkout::Checkout,
        id::{CheckoutId, ItemId, UserId},
        item::{Item, ItemCategory, book::Book},
        list::PaginatedList,
    },
    repository::{checkout::MockCheckoutRepository, item::MockItemRepository},
};
use rstest::rstest;
use tower::ServiceExt;

use crate::{
    deserialize_json,
    helper::{TestRequestExt, fixture, make_router, v1},
};

#[rstest]
#[case("/items", None, 20, 0)]
#[case("/items?category=book", Some(ItemCategory::Book), 20, 0)]
#[case("/items?limit=50", None, 50, 0)]
#[case("/items?limit=50&offset=20", None, 50, 20)]
#[tokio::test]
async fn list_items_200(
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
    #[case] category: Option<ItemCategory>,
    #[case] expected_limit: i64,
    #[case] expected_offset: i64,
) -> anyhow::Result<()> {
    let item_id = ItemId::new();

    fixture.expect_item_repository().returning(move || {
        let mut mock = MockItemRepository::new();
        mock.expect_find_all().returning(move |opt| {
            assert_eq!(opt.category, category.clone());
            let items = vec![Item::Book(Book {
                id: item_id,
                name: "RustによるWebアプリケーション開発".into(),
                isbn: "".into(),
                author: "Yuki Toyoda".into(),
                description: "RustによるWebアプリケーション開発".into(),
                checkout: None,
            })];
            Ok(PaginatedList {
                total: 1,
                limit: opt.limit,
                offset: opt.offset,
                items,
            })
        });
        Arc::new(mock)
    });

    let app: axum::Router = make_router(fixture);

    let req = Request::get(v1(path)).bearer().body(Body::empty())?;
    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, PaginatedItemResponse);
    assert_eq!(result.limit, expected_limit);
    assert_eq!(result.offset, expected_offset);

    Ok(())
}

#[rstest]
#[case("/items?limit=-1")]
#[case("/items?offset=aaa")]
#[tokio::test]
async fn list_items_400(
    fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
) -> anyhow::Result<()> {
    let app: axum::Router = make_router(fixture);

    let req = Request::get(v1(path)).bearer().body(Body::empty())?;
    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn create_item_201(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture.expect_item_repository().returning(move || {
        let mut mock = MockItemRepository::new();
        mock.expect_create().returning(|_| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = CreateItemRequest::Book {
        name: "Test Book".into(),
        author: "Test Author".into(),
        isbn: "1234567890123".into(),
        description: "Test Description".into(),
    };

    let req = Request::post(v1("/items"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::CREATED);

    Ok(())
}

#[rstest]
#[case("", "Test Author", "1234567890123", "Test Description")] // name empty
#[case("Test Book", "", "1234567890123", "Test Description")] // author empty
#[case("Test Book", "Test Author", "", "Test Description")] // ISBN empty
#[tokio::test]
async fn create_item_400(
    fixture: registry::MockAppRegistryExt,
    #[case] name: &str,
    #[case] author: &str,
    #[case] isbn: &str,
    #[case] description: &str,
) -> anyhow::Result<()> {
    let app = make_router(fixture);

    let req = CreateItemRequest::Book {
        name: name.into(),
        author: author.into(),
        isbn: isbn.into(),
        description: description.into(),
    };

    let req = Request::post(v1("/items"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn get_item_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    fixture.expect_item_repository().returning(move || {
        let mut mock = MockItemRepository::new();
        mock.expect_find_by_id().returning(move |_id| {
            Ok(Some(Item::Book(Book {
                id: item_id,
                name: "Test Book".into(),
                isbn: "1234567890123".into(),
                author: "Test Author".into(),
                description: "Test Description".into(),
                checkout: None,
            })))
        });
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1(&format!("/items/{}", item_id)))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, ItemResponse);
    match result {
        ItemResponse::Book(book) => {
            assert_eq!(book.name, "Test Book");
            assert_eq!(book.author, "Test Author");
            assert_eq!(book.isbn, "1234567890123");
            assert_eq!(book.description, "Test Description");
        }
        _ => panic!("Expected BookResponse"),
    }

    Ok(())
}

#[rstest]
#[tokio::test]
async fn get_item_404(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture.expect_item_repository().returning(move || {
        let mut mock = MockItemRepository::new();
        mock.expect_find_by_id().returning(|_id| Ok(None));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1(&format!("/items/{}", ItemId::new())))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn update_item_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    fixture.expect_item_repository().returning(move || {
        let mut mock = MockItemRepository::new();
        mock.expect_update().returning(|_| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = UpdateItemRequest::Book {
        name: "Updated Title".into(),
        author: "Updated Author".into(),
        isbn: "1234567890123".into(),
        description: "Updated Description".into(),
    };

    let req = Request::put(v1(&format!("/items/{}", item_id)))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[case("", "Test Author", "1234567890123", "Test Description")] // name empty
#[case("Test Book", "", "1234567890123", "Test Description")] // author empty
#[case("Test Book", "Test Author", "", "Test Description")] // ISBN empty
#[tokio::test]
async fn update_item_400(
    fixture: registry::MockAppRegistryExt,
    #[case] name: &str,
    #[case] author: &str,
    #[case] isbn: &str,
    #[case] description: &str,
) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    let app = make_router(fixture);

    let req = UpdateItemRequest::Book {
        name: name.into(),
        author: author.into(),
        isbn: isbn.into(),
        description: description.into(),
    };

    let req = Request::put(v1(&format!("/items/{}", item_id)))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn delete_item_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    fixture.expect_item_repository().returning(move || {
        let mut mock = MockItemRepository::new();
        mock.expect_delete().returning(|_| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::delete(v1(&format!("/items/{}", item_id)))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn show_checked_out_list_200(
    mut fixture: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    let user_id = UserId::new();
    let checkout_id = CheckoutId::new();
    let now = chrono::Utc::now();

    fixture.expect_checkout_repository().returning(move || {
        let mut mock = MockCheckoutRepository::new();
        mock.expect_find_unreturned_all().returning(move || {
            Ok(vec![Checkout {
                id: checkout_id,
                checked_out_by: user_id,
                checked_out_at: now,
                returned_at: None,
                item_id,
            }])
        });
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1("/items/checkouts"))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, CheckoutsResponse);
    assert_eq!(result.items.len(), 1);
    let checkout = &result.items[0];
    assert_eq!(checkout.id, checkout_id);
    assert_eq!(checkout.checked_out_by, user_id);
    assert_eq!(checkout.checked_out_at, now);
    assert_eq!(checkout.returned_at, None);
    assert_eq!(checkout.item_id, item_id);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn checkout_item_201(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    fixture.expect_checkout_repository().returning(move || {
        let mut mock = MockCheckoutRepository::new();
        mock.expect_create().returning(|_event| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::post(v1(&format!("/items/{}/checkouts", item_id)))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::CREATED);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn return_item_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    let checkout_id = CheckoutId::new();
    fixture.expect_checkout_repository().returning(move || {
        let mut mock = MockCheckoutRepository::new();
        mock.expect_update_returned().returning(|_event| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::put(v1(&format!(
        "/items/{}/checkouts/{}/returned",
        item_id, checkout_id
    )))
    .bearer()
    .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn checkout_history_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let item_id = ItemId::new();
    let user_id = UserId::new();
    let checkout_id = CheckoutId::new();
    let now = chrono::Utc::now();

    fixture.expect_checkout_repository().returning(move || {
        let mut mock = MockCheckoutRepository::new();
        mock.expect_find_history_by_item_id().returning(move |_id| {
            Ok(vec![Checkout {
                id: checkout_id,
                checked_out_by: user_id,
                checked_out_at: now,
                returned_at: Some(now),
                item_id,
            }])
        });
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1(&format!("/items/{}/checkout-history", item_id)))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, CheckoutsResponse);
    assert_eq!(result.items.len(), 1);
    let checkout = &result.items[0];
    assert_eq!(checkout.id, checkout_id);
    assert_eq!(checkout.checked_out_by, user_id);
    assert_eq!(checkout.checked_out_at, now);
    assert_eq!(checkout.returned_at, Some(now));
    assert_eq!(checkout.item_id, item_id);

    Ok(())
}
