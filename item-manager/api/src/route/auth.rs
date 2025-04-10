use axum::{Router, routing::post};
use registry::AppRegistry;

use crate::handler::auth::login;

pub fn routes() -> Router<AppRegistry> {
    let auth_router = Router::new().route("/login", post(login));

    Router::new().nest("/auth", auth_router)
}
