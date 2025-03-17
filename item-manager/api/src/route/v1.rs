use axum::Router;
use registry::AppRegistry;

use super::{health, item, user};

pub fn routes() -> Router<AppRegistry> {
    let router = Router::new()
        .merge(health::routes())
        .merge(item::routes())
        .merge(user::routes());

    Router::new().nest("/api/v1", router)
}
