use axum::{
    Router,
    routing::{delete, get, post, put},
};
use registry::AppRegistry;

use crate::handler::{
    checkout::{checkout_book, checkout_history, return_book, show_checked_out_list},
    item::{create_item, delete_item, get_item, list_items, update_item},
};

pub fn routes() -> Router<AppRegistry> {
    let items_router = Router::new()
        .route("/", get(list_items))
        .route("/", post(create_item))
        .route("/{item_id}", get(get_item))
        .route("/{item_id}", put(update_item))
        .route("/{item_id}", delete(delete_item));

    let checkout_router = Router::new()
        .route("/checkouts", get(show_checked_out_list))
        .route("/{item_id}/checkouts", post(checkout_book))
        .route(
            "/{item_id}/checkouts/{checkout_id}/returned",
            put(return_book),
        )
        .route("/{item_id}/checkout-history", get(checkout_history));

    Router::new().nest("/items", items_router.merge(checkout_router))
}
