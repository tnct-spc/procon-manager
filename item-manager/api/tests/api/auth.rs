use std::sync::Arc;

use api::model::auth::{AccessTokenResponse, LoginRequest};
use axum::{body::Body, http::Request};
use kernel::{
    model::{auth::AccessToken, id::UserId},
    repository::auth::MockAuthRepository,
};
use rstest::rstest;
use shared::error::AppError;
use tower::ServiceExt;

use crate::{
    deserialize_json,
    helper::{TestRequestExt, fixture_registry, make_router, v1},
};

#[rstest]
#[tokio::test]
async fn login_success_200(
    mut fixture_registry: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    let user_id = UserId::new();
    let test_token = Arc::new("test_token".to_string());

    fixture_registry.expect_auth_repository().returning({
        let test_token = Arc::clone(&test_token);
        move || {
            let mut mock = MockAuthRepository::new();
            let user_id = user_id;
            let test_token = Arc::clone(&test_token);

            mock.expect_verify_user()
                .returning(move |_email, _password| {
                    eprintln!("{user_id:?}");
                    Ok(user_id)
                });

            mock.expect_create_token()
                .returning(move |_event| Ok(AccessToken((*test_token).clone())));

            Arc::new(mock)
        }
    });

    let app = make_router(fixture_registry);

    let req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };

    let req = Request::post("/auth/login")
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    let result = deserialize_json!(resp, AccessTokenResponse);
    assert_eq!(result.user_id, user_id);
    assert_eq!(result.access_token, *test_token);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn login_failed_401(
    mut fixture_registry: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_registry
        .expect_auth_repository()
        .returning(move || {
            let mut mock = MockAuthRepository::new();

            mock.expect_verify_user()
                .returning(move |_email, _password| Err(AppError::UnauthenticatedError));

            Arc::new(mock)
        });

    let app = make_router(fixture_registry);

    let req = LoginRequest {
        email: "test@example.com".to_string(),
        password: "wrong_password".to_string(),
    };

    let req = Request::post("/auth/login")
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn verify_token_expired(
    mut fixture_registry: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_registry
        .expect_auth_repository()
        .returning(move || {
            let mut mock = MockAuthRepository::new();

            // Simulate expired token
            mock.expect_fetch_user_id_from_token()
                .returning(|_token| Ok(None));

            Arc::new(mock)
        });

    let app = make_router(fixture_registry);

    let req = Request::get(v1("/books")).bearer().body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn verify_token_invalid(
    mut fixture_registry: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_registry
        .expect_auth_repository()
        .returning(move || {
            let mut mock = MockAuthRepository::new();

            // Simulate invalid token
            mock.expect_fetch_user_id_from_token()
                .returning(|_token| Err(AppError::UnauthenticatedError));

            Arc::new(mock)
        });

    let app = make_router(fixture_registry);

    let req = Request::get(v1("/books")).bearer().body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

    Ok(())
}
