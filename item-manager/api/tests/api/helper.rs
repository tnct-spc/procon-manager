use std::sync::Arc;

use api::route::{auth, v1};
use axum::{Router, http::request::Builder};
use kernel::{
    model::{auth::AccessToken, id::UserId, role::Role, user::User},
    repository::{auth::MockAuthRepository, user::MockUserRepository},
};
use registry::MockAppRegistryExt;
use rstest::fixture;

pub fn v1(endpoint: &str) -> String {
    format!("/api/v1{endpoint}")
}

pub fn make_router(registry: MockAppRegistryExt) -> Router {
    Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .with_state(Arc::new(registry))
}

#[fixture]
pub fn fixture_registry() -> MockAppRegistryExt {
    MockAppRegistryExt::new()
}

#[fixture]
pub fn fixture_auth(mut fixture_registry: MockAppRegistryExt) -> MockAppRegistryExt {
    fixture_registry.expect_auth_repository().returning(|| {
        let mut mock_auth_repository = MockAuthRepository::new();
        mock_auth_repository
            .expect_fetch_user_id_from_token()
            .returning(|_| Ok(Some(UserId::new())));
        mock_auth_repository
            .expect_verify_user()
            .returning(|_, _| Ok(UserId::new()));
        mock_auth_repository
            .expect_create_token()
            .returning(|_| Ok(AccessToken("dummy".into())));
        Arc::new(mock_auth_repository)
    });
    fixture_registry
}

#[fixture]
pub fn fixture(mut fixture_auth: MockAppRegistryExt) -> MockAppRegistryExt {
    fixture_auth.expect_user_repository().returning(|| {
        let mut mock_user_repository = MockUserRepository::new();
        mock_user_repository
            .expect_find_current_user()
            .returning(|id| {
                Ok(Some(User {
                    id,
                    name: "dummy-user".into(),
                    email: "dummy@example.com".into(),
                    role: Role::User,
                }))
            });
        Arc::new(mock_user_repository)
    });
    fixture_auth
}

#[fixture]
pub fn fixture_admin(mut fixture_auth: MockAppRegistryExt) -> MockAppRegistryExt {
    fixture_auth.expect_user_repository().returning(|| {
        let mut mock_user_repository = MockUserRepository::new();
        mock_user_repository
            .expect_find_current_user()
            .returning(|id| {
                Ok(Some(User {
                    id,
                    name: "admin-user".into(),
                    email: "admin@example.com".into(),
                    role: Role::Admin,
                }))
            });
        Arc::new(mock_user_repository)
    });
    fixture_auth
}

pub trait TestRequestExt {
    fn bearer(self) -> Builder;
    fn application_json(self) -> Builder;
}

impl TestRequestExt for Builder {
    fn bearer(self) -> Builder {
        self.header("Authorization", "Bearer dummy")
    }

    fn application_json(self) -> Builder {
        self.header("Content-Type", "application/json")
    }
}

#[macro_export]
macro_rules! deserialize_json {
    ($res:expr, $target:ty) => {{
        use tokio_stream::StreamExt;

        let mut bytes = Vec::new();
        let body = $res.into_body();
        let mut stream = body.into_data_stream();
        while let Ok(Some(chunk)) = stream.try_next().await {
            bytes.extend_from_slice(&chunk[..]);
        }
        let body: $target = serde_json::from_slice(&bytes)?;
        body
    }};
}
