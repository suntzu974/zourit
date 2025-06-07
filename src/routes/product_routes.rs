use axum::{
    routing::{get, post, delete, put},
    Router,
};
use crate::database::SharedConnection;
use crate::handlers::{
    product_handler::*,
};

pub fn create_product_routes() -> Router<SharedConnection> {
    Router::new()
        .route("/products", post(create_product))
        .route("/products", get(get_all_products))
        .route("/products/{id}", get(get_product))
        .route("/products/{id}", put(update_product))
        .route("/products/{id}", delete(delete_product))
}
