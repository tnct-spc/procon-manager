use axum::{
    Router,
    routing::{delete, get, post, put},
};
use registry::AppRegistry;

use crate::handler::{
    book::{delete_book, register_book, show_book, show_book_list, update_book},
    checkout::{checkout_book, checkout_history, return_book, show_checked_out_list},
};

pub fn routes() -> Router<AppRegistry> {
    let books_routers = Router::new()
        .route("/", get(show_book_list))
        .route("/", post(register_book))
        .route("/{book_id}", get(show_book))
        .route("/{book_id}", put(update_book))
        .route("/{book_id}", delete(delete_book));

    let checkout_router = Router::new()
        .route("/checkouts", get(show_checked_out_list))
        .route("/{book_id}/checkouts", post(checkout_book))
        .route(
            "/{book_id}/checkouts/{checkout_id}/returned",
            put(return_book),
        )
        .route("/{book_id}/checkout-history", get(checkout_history));

    Router::new().nest("/books", books_routers.merge(checkout_router))
}
