use crate::handler::auth::login;
use axum::{Router, routing::post};
use registry::AppRegistry;

pub fn routes() -> Router<AppRegistry> {
    let auth_router = Router::new().route("/login", post(login));

    Router::new().nest("/auth", auth_router)
}
