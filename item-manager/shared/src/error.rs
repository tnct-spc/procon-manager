use axum::{Json, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    UnprocessableEntity(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    EntityNotFound(String),
    #[error("{0}")]
    ValidationError(#[from] garde::Report),
    #[error("Transaction could not be executed.")]
    TransactionError(#[source] sqlx::Error),
    #[error("An error occurred while performing database processing.")]
    SpecificOperationError(#[source] sqlx::Error),
    #[error("No rows affected: {0}")]
    NoRowsAffectedError(String),
    #[error("{0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("{0}")]
    ConvertToUuidError(#[from] uuid::Error),
    #[error("Login failed.")]
    UnauthenticatedError,
    #[error("The authorization information is incorrect.")]
    UnauthorizedError,
    #[error("{0}")]
    ForbiddenOperation(String),
    #[error("{0}")]
    ConversionEntityError(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, message) = match self {
            AppError::UnprocessableEntity(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::EntityNotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::ValidationError(report) => (StatusCode::BAD_REQUEST, report.to_string()),
            AppError::ConvertToUuidError(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            AppError::UnauthorizedError => (
                StatusCode::FORBIDDEN,
                "The authorization information is incorrect.".into(),
            ),
            AppError::ForbiddenOperation(message) => (StatusCode::FORBIDDEN, message),
            AppError::UnauthenticatedError => (StatusCode::UNAUTHORIZED, "Login failed.".into()),
            e @ (AppError::TransactionError(_)
            | AppError::SpecificOperationError(_)
            | AppError::NoRowsAffectedError(_)
            | AppError::BcryptError(_)
            | AppError::ConversionEntityError(_)) => {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Unexpected error happened"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error.".into(),
                )
            }
        };
        (status_code, Json(ErrorResponse { message })).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
