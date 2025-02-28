use axum::{
    Router,
    routing::{delete, get, post, put},
};
use registry::AppRegistry;

use crate::handler::book::{delete_book, register_book, show_book, show_book_list, update_book};

pub fn routes() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/", get(show_book_list))
        .route("/", post(register_book))
        .route("/{book_id}", get(show_book))
        .route("/{book_id}", put(update_book))
        .route("/{book_id}", delete(delete_book));

    Router::new().nest("/books", routers)
}
