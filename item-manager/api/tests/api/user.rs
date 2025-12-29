use std::sync::Arc;

use api::model::user::{
    CreateUserRequest, RoleName, UpdateUserEmailRequest, UpdateUserNameRequest,
    UpdateUserPasswordRequest, UpdateUserRoleRequest,
};
use axum::{body::Body, http::Request};
use kernel::{
    model::{id::UserId, role::Role, user::User},
    repository::{checkout::MockCheckoutRepository, user::MockUserRepository},
};
use rstest::rstest;
use tower::ServiceExt;

use crate::helper::{TestRequestExt, fixture, fixture_auth, make_router, v1};

#[rstest]
#[tokio::test]
async fn update_user_name_200(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(kernel::model::user::User {
                id,
                name: "dummy-user".into(),
                email: "dummy@example.com".into(),
                role: kernel::model::role::Role::User,
            }))
        });

        mock.expect_update_name().returning(|_| Ok(()));

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = UpdateUserNameRequest {
        name: "Updated Name".to_string(),
    };

    let req = Request::put(v1("/users/me/name"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn update_user_email_200(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(kernel::model::user::User {
                id,
                name: "dummy-user".into(),
                email: "dummy@example.com".into(),
                role: kernel::model::role::Role::User,
            }))
        });

        mock.expect_update_email().returning(|_| Ok(()));

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = UpdateUserEmailRequest {
        email: "updated@example.com".to_string(),
    };

    let req = Request::put(v1("/users/me/email"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn update_user_email_409_duplicate(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(kernel::model::user::User {
                id,
                name: "dummy-user".into(),
                email: "dummy@example.com".into(),
                role: kernel::model::role::Role::User,
            }))
        });

        mock.expect_update_email().returning(|_| {
            Err(shared::error::AppError::Conflict(
                "Email already exists.".into(),
            ))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = UpdateUserEmailRequest {
        email: "duplicate@example.com".to_string(),
    };

    let req = Request::put(v1("/users/me/email"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::CONFLICT);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn update_user_name_400_empty_name(
    fixture: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    let app = make_router(fixture);

    let req = UpdateUserNameRequest {
        name: "".to_string(), // Empty name should fail validation
    };

    let req = Request::put(v1("/users/me/name"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn update_user_email_400_invalid_email(
    fixture: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    let app = make_router(fixture);

    let req = UpdateUserEmailRequest {
        email: "invalid-email".to_string(), // Invalid email should fail validation
    };

    let req = Request::put(v1("/users/me/email"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn register_user_200(mut fixture_auth: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "admin-user".into(),
                email: "admin@example.com".into(),
                role: Role::Admin,
            }))
        });

        mock.expect_create().returning(|event| {
            Ok(User {
                id: UserId::new(),
                name: event.name,
                email: event.email,
                role: Role::User,
            })
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = CreateUserRequest {
        name: "New User".to_string(),
        email: "newuser@example.com".to_string(),
        password: "password123".to_string(),
    };

    let req = Request::post(v1("/users"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn register_user_409_duplicate_email(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "admin-user".into(),
                email: "admin@example.com".into(),
                role: Role::Admin,
            }))
        });

        mock.expect_create().returning(|_event| {
            Err(shared::error::AppError::Conflict(
                "Email already exists.".into(),
            ))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = CreateUserRequest {
        name: "New User".to_string(),
        email: "existing@example.com".to_string(),
        password: "password123".to_string(),
    };

    let req = Request::post(v1("/users"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::CONFLICT);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn register_user_403_not_admin(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "regular-user".into(),
                email: "user@example.com".into(),
                role: Role::User,
            }))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = CreateUserRequest {
        name: "New User".to_string(),
        email: "newuser@example.com".to_string(),
        password: "password123".to_string(),
    };

    let req = Request::post(v1("/users"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn register_user_400_invalid_email(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "admin-user".into(),
                email: "admin@example.com".into(),
                role: Role::Admin,
            }))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = CreateUserRequest {
        name: "New User".to_string(),
        email: "invalid-email".to_string(), // Invalid email
        password: "password123".to_string(),
    };

    let req = Request::post(v1("/users"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn list_users_200(mut fixture_auth: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "test-user".into(),
                email: "test@example.com".into(),
                role: Role::Admin,
            }))
        });

        mock.expect_find_all().returning(|| {
            Ok(vec![
                User {
                    id: UserId::new(),
                    name: "User 1".into(),
                    email: "user1@example.com".into(),
                    role: Role::User,
                },
                User {
                    id: UserId::new(),
                    name: "User 2".into(),
                    email: "user2@example.com".into(),
                    role: Role::Admin,
                },
            ])
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = Request::get(v1("/users")).bearer().body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn list_users_403(mut fixture_auth: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "test-user".into(),
                email: "test@example.com".into(),
                role: Role::User,
            }))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = Request::get(v1("/users")).bearer().body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn delete_user_200(mut fixture_auth: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "admin-user".into(),
                email: "admin@example.com".into(),
                role: Role::Admin,
            }))
        });

        mock.expect_delete().returning(|_| Ok(()));

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let user_id = UserId::new();
    let req = Request::delete(v1(&format!("/users/{user_id}")))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn delete_user_403_not_admin(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "regular-user".into(),
                email: "user@example.com".into(),
                role: Role::User,
            }))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let user_id = UserId::new();
    let req = Request::delete(v1(&format!("/users/{user_id}")))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn change_role_200(mut fixture_auth: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "admin-user".into(),
                email: "admin@example.com".into(),
                role: Role::Admin,
            }))
        });

        mock.expect_update_role().returning(|_| Ok(()));

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = UpdateUserRoleRequest {
        role: RoleName::Admin,
    };

    let user_id = UserId::new();
    let req = Request::put(v1(&format!("/users/{user_id}/role")))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn change_role_403_not_admin(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "regular-user".into(),
                email: "user@example.com".into(),
                role: Role::User,
            }))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = UpdateUserRoleRequest {
        role: RoleName::Admin,
    };

    let user_id = UserId::new();
    let req = Request::put(v1(&format!("/users/{user_id}/role")))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::FORBIDDEN);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn get_current_user_200(
    mut fixture_auth: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "test-user".into(),
                email: "test@example.com".into(),
                role: Role::User,
            }))
        });

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = Request::get(v1("/users/me")).bearer().body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn change_password_200(mut fixture_auth: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "test-user".into(),
                email: "test@example.com".into(),
                role: Role::User,
            }))
        });

        mock.expect_update_password().returning(|_| Ok(()));

        Arc::new(mock)
    });

    let app = make_router(fixture_auth);

    let req = UpdateUserPasswordRequest {
        current_password: "oldpassword".to_string(),
        new_password: "newpassword".to_string(),
    };

    let req = Request::put(v1("/users/me/password"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn change_password_400_empty_password(
    fixture: registry::MockAppRegistryExt,
) -> anyhow::Result<()> {
    let app = make_router(fixture);

    let req = UpdateUserPasswordRequest {
        current_password: "".to_string(), // Empty password should fail validation
        new_password: "newpassword".to_string(),
    };

    let req = Request::put(v1("/users/me/password"))
        .bearer()
        .application_json()
        .body(Body::from(serde_json::to_string(&req)?))?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn get_checkouts_200(mut fixture_auth: registry::MockAppRegistryExt) -> anyhow::Result<()> {
    fixture_auth.expect_user_repository().returning(move || {
        let mut mock = MockUserRepository::new();

        mock.expect_find_current_user().returning(|id| {
            Ok(Some(User {
                id,
                name: "test-user".into(),
                email: "test@example.com".into(),
                role: Role::User,
            }))
        });

        Arc::new(mock)
    });

    fixture_auth
        .expect_checkout_repository()
        .returning(move || {
            let mut mock = MockCheckoutRepository::new();

            mock.expect_find_unreturned_by_user_id()
                .returning(|_| Ok(vec![]));

            Arc::new(mock)
        });

    let app = make_router(fixture_auth);

    let req = Request::get(v1("/users/me/checkouts"))
        .bearer()
        .body(Body::empty())?;

    let resp = app.oneshot(req).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    Ok(())
}
