use std::sync::Arc;

use api::model::auth::{LoginRequest, LoginResponse};
use axum::{
    body::Body,
    http::{Request, header::SET_COOKIE},
};
use kernel::{
    model::{auth::AccessToken, id::UserId},
    repository::auth::MockAuthRepository,
};
use rstest::rstest;
use shared::config::WebConfig;
use shared::error::AppError;
use tower::ServiceExt;

use crate::{
    deserialize_json,
    helper::{TestRequestExt, fixture_registry, make_router, v1},
};

fn expect_web_config(registry: &mut registry::MockAppRegistryExt) {
    registry.expect_web_config().returning(|| WebConfig {
        frontend_origin: "http://localhost:5173".to_string(),
        access_token_cookie_name: "access_token".to_string(),
        access_token_cookie_max_age_seconds: 86_400,
    });
}

#[rstest]
#[tokio::test]
async fn login_success_200(
    mut fixture_registry: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    let user_id = UserId::new();
    let test_token = Arc::new("test_token".to_string());

    expect_web_config(&mut fixture_registry);

    fixture_registry.expect_auth_repository().returning({
        let test_token = Arc::clone(&test_token);
        move || {
            let mut mock = MockAuthRepository::new();
            let user_id = user_id;
            let test_token = Arc::clone(&test_token);

            mock.expect_verify_user()
                .returning(move |_email, _password| Ok(user_id));

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

    let cookie = resp
        .headers()
        .get(SET_COOKIE)
        .and_then(|value| value.to_str().ok())
        .expect("login response should set access token cookie");
    assert!(cookie.contains("access_token=test_token"));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("Secure"));

    let result = deserialize_json!(resp, LoginResponse);
    assert_eq!(result.user_id, user_id);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn login_failed_401(
    mut fixture_registry: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    expect_web_config(&mut fixture_registry);

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
    expect_web_config(&mut fixture_registry);

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

    let req = Request::get(v1("/items")).bearer().body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn verify_token_invalid(
    mut fixture_registry: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    expect_web_config(&mut fixture_registry);

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

    let req = Request::get(v1("/items")).bearer().body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::UNAUTHORIZED);

    Ok(())
}
