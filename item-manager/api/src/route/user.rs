use axum::{
    Router,
    routing::{delete, get, put},
};
use registry::AppRegistry;

use crate::handler::user::{
    change_email, change_name, change_password, change_role, delete_user, get_checkouts,
    get_current_user, list_users, register_user,
};

pub fn routes() -> Router<AppRegistry> {
    Router::new()
        .route("/users/me", get(get_current_user))
        .route("/users/me/password", put(change_password))
        .route("/users/me/name", put(change_name))
        .route("/users/me/email", put(change_email))
        .route("/users/me/checkouts", get(get_checkouts))
        .route("/users", get(list_users).post(register_user))
        .route("/users/{user_id}", delete(delete_user))
        .route("/users/{user_id}/role", put(change_role))
}
