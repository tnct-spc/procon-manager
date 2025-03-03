use std::sync::Arc;

use api::model::book::{BookResponse, CreateBookRequest, PaginatedBookResponse, UpdateBookRequest};
use api::model::checkout::CheckoutsResponse;
use axum::{body::Body, http::Request};
use kernel::{
    model::{
        book::Book,
        checkout::{Checkout, CheckoutBook},
        id::{BookId, CheckoutId, UserId},
        list::PaginatedList,
        user::BookOwner,
    },
    repository::{book::MockBookRepository, checkout::MockCheckoutRepository},
};
use rstest::rstest;
use tower::ServiceExt;

use crate::{
    deserialize_json,
    helper::{TestRequestExt, fixture, make_router, v1},
};

#[rstest]
#[case("/books", 20, 0)]
#[case("/books?limit=50", 50, 0)]
#[case("/books?limit=50&offset=20", 50, 20)]
#[case("/books?offset=20", 20, 20)]
#[tokio::test]
async fn show_book_list_with_query_200(
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
    #[case] expected_limit: i64,
    #[case] expected_offset: i64,
) -> anyhow::Result<()> {
    let book_id = BookId::new();

    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_find_all().returning(move |opt| {
            let items = vec![Book {
                id: book_id,
                title: "RustによるWebアプリケーション開発".into(),
                isbn: "".into(),
                author: "Yuki Toyoda".into(),
                description: "RustによるWebアプリケーション開発".into(),
                owner: BookOwner {
                    id: UserId::new(),
                    name: "Yuki Toyoda".into(),
                },
                checkout: None,
            }];
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

    let result = deserialize_json!(resp, PaginatedBookResponse);
    assert_eq!(result.limit, expected_limit);
    assert_eq!(result.offset, expected_offset);

    Ok(())
}

#[rstest]
#[case("/books?limit=-1")]
#[case("/books?offset=aaa")]
#[tokio::test]
async fn show_book_list_with_query_400(
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
) -> anyhow::Result<()> {
    let book_id = BookId::new();

    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_find_all().returning(move |opt| {
            let items = vec![Book {
                id: book_id,
                title: "RustによるWebアプリケーション開発".into(),
                isbn: "".into(),
                author: "Yuki Toyoda".into(),
                description: "RustによるWebアプリケーション開発".into(),
                owner: BookOwner {
                    id: UserId::new(),
                    name: "Yuki Toyoda".into(),
                },
                checkout: None,
            }];
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
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn register_book_201(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_create().returning(|_book, _user_id| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = CreateBookRequest {
        title: "Test Book".into(),
        author: "Test Author".into(),
        isbn: "1234567890123".into(),
        description: "Test Description".into(),
    };

    let req = Request::post(v1("/books"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::CREATED);

    Ok(())
}

#[rstest]
#[case("", "Test Author", "1234567890123", "Test Description")] // title empty
#[case("Test Book", "", "1234567890123", "Test Description")] // author empty
#[case("Test Book", "Test Author", "", "Test Description")] // ISBN empty
#[tokio::test]
async fn register_book_400(
    mut fixture: registry::MockAppRegistryExt,
    #[case] title: &str,
    #[case] author: &str,
    #[case] isbn: &str,
    #[case] description: &str,
) -> anyhow::Result<()> {
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_create().returning(|_book, _user_id| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = CreateBookRequest {
        title: title.into(),
        author: author.into(),
        isbn: isbn.into(),
        description: description.into(),
    };

    let req = Request::post(v1("/books"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn show_book_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let book_id = BookId::new();
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_find_by_id().returning(move |_id| {
            Ok(Some(Book {
                id: book_id,
                title: "Test Book".into(),
                isbn: "1234567890123".into(),
                author: "Test Author".into(),
                description: "Test Description".into(),
                owner: BookOwner {
                    id: UserId::new(),
                    name: "Test User".into(),
                },
                checkout: None,
            }))
        });
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1(&format!("/books/{}", book_id)))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, BookResponse);
    assert_eq!(result.title, "Test Book");
    assert_eq!(result.author, "Test Author");
    assert_eq!(result.isbn, "1234567890123");
    assert_eq!(result.description, "Test Description");

    Ok(())
}

#[rstest]
#[tokio::test]
async fn show_book_404(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_find_by_id().returning(|_id| Ok(None));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1(&format!("/books/{}", BookId::new())))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::NOT_FOUND);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn update_book_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let book_id = BookId::new();
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_update().returning(|_book| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = UpdateBookRequest {
        title: "Updated Title".into(),
        author: "Updated Author".into(),
        isbn: "1234567890123".into(),
        description: "Updated Description".into(),
    };

    let req = Request::put(v1(&format!("/books/{}", book_id)))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[case("", "Test Author", "1234567890123", "Test Description")] // title empty
#[case("Test Book", "", "1234567890123", "Test Description")] // author empty
#[case("Test Book", "Test Author", "", "Test Description")] // ISBN empty
#[tokio::test]
async fn update_book_400(
    mut fixture: registry::MockAppRegistryExt,
    #[case] title: &str,
    #[case] author: &str,
    #[case] isbn: &str,
    #[case] description: &str,
) -> anyhow::Result<()> {
    let book_id = BookId::new();
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_update().returning(|_book| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = UpdateBookRequest {
        title: title.into(),
        author: author.into(),
        isbn: isbn.into(),
        description: description.into(),
    };

    let req = Request::put(v1(&format!("/books/{}", book_id)))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn delete_book_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let book_id = BookId::new();
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();
        mock.expect_delete().returning(|_book| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::delete(v1(&format!("/books/{}", book_id)))
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
    let book_id = BookId::new();
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
                book: CheckoutBook {
                    book_id,
                    title: "Test Book".into(),
                    author: "Test Author".into(),
                    isbn: "1234567890123".into(),
                },
            }])
        });
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1("/books/checkouts"))
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
    assert_eq!(checkout.book.id, book_id);
    assert_eq!(checkout.book.title, "Test Book");
    assert_eq!(checkout.book.author, "Test Author");
    assert_eq!(checkout.book.isbn, "1234567890123");

    Ok(())
}

#[rstest]
#[tokio::test]
async fn checkout_book_201(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let book_id = BookId::new();
    fixture.expect_checkout_repository().returning(move || {
        let mut mock = MockCheckoutRepository::new();
        mock.expect_create().returning(|_event| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::post(v1(&format!("/books/{}/checkouts", book_id)))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::CREATED);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn return_book_200(mut fixture: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    let book_id = BookId::new();
    let checkout_id = CheckoutId::new();
    fixture.expect_checkout_repository().returning(move || {
        let mut mock = MockCheckoutRepository::new();
        mock.expect_update_returned().returning(|_event| Ok(()));
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::put(v1(&format!(
        "/books/{}/checkouts/{}/returned",
        book_id, checkout_id
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
    let book_id = BookId::new();
    let user_id = UserId::new();
    let checkout_id = CheckoutId::new();
    let now = chrono::Utc::now();

    fixture.expect_checkout_repository().returning(move || {
        let mut mock = MockCheckoutRepository::new();
        mock.expect_find_history_by_book_id().returning(move |_id| {
            Ok(vec![Checkout {
                id: checkout_id,
                checked_out_by: user_id,
                checked_out_at: now,
                returned_at: Some(now),
                book: CheckoutBook {
                    book_id,
                    title: "Test Book".into(),
                    author: "Test Author".into(),
                    isbn: "1234567890123".into(),
                },
            }])
        });
        Arc::new(mock)
    });

    let app = make_router(fixture);

    let req = Request::get(v1(&format!("/books/{}/checkout-history", book_id)))
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
    assert_eq!(checkout.book.id, book_id);
    assert_eq!(checkout.book.title, "Test Book");
    assert_eq!(checkout.book.author, "Test Author");
    assert_eq!(checkout.book.isbn, "1234567890123");

    Ok(())
}
