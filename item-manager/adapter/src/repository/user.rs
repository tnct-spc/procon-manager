use crate::database::{ConnectionPool, model::user::UserRow};
use async_trait::async_trait;
use derive_new::new;
use kernel::model::id::UserId;
use kernel::model::role::Role;
use kernel::model::user::{
    User,
    event::{
        CreateUser, DeleteUser, UpdateUserEmail, UpdateUserName, UpdateUserPassword, UpdateUserRole,
    },
};
use kernel::repository::user::UserRepository;
use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct UserRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_current_user(&self, current_user_id: UserId) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"
                SELECT
                u.user_id,
                u.name,
                u.email,
                r.name as role_name,
                u.created_at,
                u.updated_at
                FROM users AS u
                INNER JOIN roles AS r USING(role_id)
                WHERE u.user_id = $1
            "#,
            current_user_id.raw()
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        match row {
            Some(r) => Ok(Some(User::try_from(r)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        let users = sqlx::query_as!(
            UserRow,
            r#"
                SELECT
                    u.user_id,
                    u.name,
                    u.email,
                    r.name as role_name,
                    u.created_at,
                    u.updated_at
                FROM users AS u
                INNER JOIN roles AS r USING(role_id)
                ORDER BY u.created_at DESC;
            "#
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .filter_map(|row| User::try_from(row).ok())
        .collect();
        Ok(users)
    }

    async fn create(&self, event: CreateUser) -> AppResult<User> {
        let user_id = UserId::new();
        let hashed_password = hash_password(&event.password)?;
        let role = Role::User;
        let res = sqlx::query!(
            r#"
                INSERT INTO users(user_id, name, email, password_hash, role_id)
                SELECT $1, $2, $3, $4, role_id FROM roles WHERE name = $5;
            "#,
            user_id.raw(),
            event.name,
            event.email,
            hashed_password,
            role.as_ref()
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(map_sqlx_error)?;
        if res.rows_affected() < 1 {
            return Err(AppError::NoRowsAffectedError(
                "No user has been created".into(),
            ));
        }
        Ok(User {
            id: user_id,
            name: event.name,
            email: event.email,
            role,
        })
    }

    async fn update_password(&self, event: UpdateUserPassword) -> AppResult<()> {
        let mut tx = self.db.begin().await?;
        let original_password_hash = sqlx::query!(
            r#"
                SELECT password_hash FROM users WHERE user_id = $1;
            "#,
            event.user_id.raw()
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|err| match err {
            sqlx::Error::RowNotFound => AppError::EntityNotFound("Specified user not found".into()),
            _ => AppError::SpecificOperationError(err),
        })?
        .password_hash;
        verify_password(&event.current_password, &original_password_hash)?;
        let new_password_hash = hash_password(&event.new_password)?;
        sqlx::query!(
            r#"
                UPDATE users SET password_hash = $2 WHERE user_id = $1;
            "#,
            event.user_id.raw(),
            new_password_hash,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SpecificOperationError)?;
        tx.commit().await.map_err(AppError::TransactionError)?;
        Ok(())
    }

    async fn update_role(&self, event: UpdateUserRole) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE users
                SET role_id = (
                   SELECT role_id FROM roles WHERE name = $2
                )
                WHERE user_id = $1
            "#,
            event.user_id.raw(),
            event.role.as_ref()
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("Specified user not found".into()));
        }
        Ok(())
    }

    async fn update_name(&self, event: UpdateUserName) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE users
                SET name = $2
                WHERE user_id = $1
            "#,
            event.user_id.raw(),
            event.name
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("Specified user not found".into()));
        }
        Ok(())
    }

    async fn update_email(&self, event: UpdateUserEmail) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE users
                SET email = $2
                WHERE user_id = $1
            "#,
            event.user_id.raw(),
            event.email
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(map_sqlx_error)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("Specified user not found".into()));
        }
        Ok(())
    }

    async fn delete(&self, event: DeleteUser) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM users
                WHERE user_id = $1
            "#,
            event.user_id.raw()
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("Specified user not found".into()));
        }
        Ok(())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AppError::from)
}

fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    let valid = bcrypt::verify(password, hash)?;
    if !valid {
        return Err(AppError::UnauthenticatedError);
    }
    Ok(())
}

fn map_sqlx_error(err: sqlx::Error) -> AppError {
    match &err {
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
            let message = match db_err.constraint() {
                Some("users_email_key") => "Email already exists.",
                _ => "Unique constraint violation.",
            };
            AppError::Conflict(message.into())
        }
        _ => AppError::SpecificOperationError(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(fixtures("common"))]
    async fn test_user_crud(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = UserRepositoryImpl::new(ConnectionPool::new(pool));

        // Test user creation
        let name = "Test User".to_string();
        let email = "test@example.com".to_string();
        let password = "test_password".to_string();

        let create_event = CreateUser {
            name: name.clone(),
            email: email.clone(),
            password: password.clone(),
        };
        let user = repo.create(create_event).await?;

        // Test find current user
        let found_user = repo.find_current_user(user.id).await?.unwrap();
        assert_eq!(found_user.name, name);
        assert_eq!(found_user.email, email);
        assert_eq!(found_user.role, Role::User);

        // Test update password
        let new_password = "new_password".to_string();
        repo.update_password(UpdateUserPassword {
            user_id: user.id,
            current_password: password.clone(),
            new_password: new_password.clone(),
        })
        .await?;

        // Test wrong password
        let result = repo
            .update_password(UpdateUserPassword {
                user_id: user.id,
                current_password: "wrong_password".into(),
                new_password: new_password.clone(),
            })
            .await;
        assert!(result.is_err());

        // Test update role
        repo.update_role(UpdateUserRole {
            user_id: user.id,
            role: Role::Admin,
        })
        .await?;

        let updated_user = repo.find_current_user(user.id).await?.unwrap();
        assert_eq!(updated_user.role, Role::Admin);

        // Test update name
        let new_name = "Updated Name".to_string();
        repo.update_name(UpdateUserName {
            user_id: user.id,
            name: new_name.clone(),
        })
        .await?;

        let updated_user = repo.find_current_user(user.id).await?.unwrap();
        assert_eq!(updated_user.name, new_name);

        // Test update email
        let new_email = "updated@example.com".to_string();
        repo.update_email(UpdateUserEmail {
            user_id: user.id,
            email: new_email.clone(),
        })
        .await?;

        let updated_user = repo.find_current_user(user.id).await?.unwrap();
        assert_eq!(updated_user.email, new_email);

        // Test find all users
        let users = repo.find_all().await?;
        assert!(!users.is_empty());
        assert!(users.iter().any(|u| u.id == user.id));

        // Test delete user
        repo.delete(DeleteUser { user_id: user.id }).await?;

        // Verify user is deleted
        let deleted_user = repo.find_current_user(user.id).await?;
        assert!(deleted_user.is_none());

        Ok(())
    }

    #[sqlx::test(fixtures("common"))]
    async fn test_error_cases(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = UserRepositoryImpl::new(ConnectionPool::new(pool));

        // Test non-existent user
        let non_existent_id = UserId::new();
        let result = repo.find_current_user(non_existent_id).await?;
        assert!(result.is_none());

        // Test delete non-existent user
        let result = repo
            .delete(DeleteUser {
                user_id: non_existent_id,
            })
            .await;
        assert!(result.is_err());

        // Test update role for non-existent user
        let result = repo
            .update_role(UpdateUserRole {
                user_id: non_existent_id,
                role: Role::Admin,
            })
            .await;
        assert!(result.is_err());

        // Test update password for non-existent user
        let result = repo
            .update_password(UpdateUserPassword {
                user_id: non_existent_id,
                current_password: "password".into(),
                new_password: "new_password".into(),
            })
            .await;
        assert!(result.is_err());

        // Test update name for non-existent user
        let result = repo
            .update_name(UpdateUserName {
                user_id: non_existent_id,
                name: "New Name".into(),
            })
            .await;
        assert!(result.is_err());

        // Test update email for non-existent user
        let result = repo
            .update_email(UpdateUserEmail {
                user_id: non_existent_id,
                email: "new@example.com".into(),
            })
            .await;
        assert!(result.is_err());

        Ok(())
    }
}
