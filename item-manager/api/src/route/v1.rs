use axum::Router;
use registry::AppRegistry;

use super::{book, health, user};

pub fn routes() -> Router<AppRegistry> {
    let router = Router::new()
        .merge(health::routes())
        .merge(book::routes())
        .merge(user::routes());

    Router::new().nest("/api/v1", router)
}
