use axum::{
    routing::{get, post, delete, put},
    Router,
};
use crate::handlers::{ product_handler::* };
use crate::handlers::auth_handler::AuthUser;
use axum::extract::{State, Path, Extension};
use crate::database::SharedConnection;
use crate::middleware::auth_middleware;

pub fn create_product_routes() -> Router<SharedConnection> {
    let auth_layer = axum::middleware::from_fn(auth_middleware);

    Router::new()
        .route("/products", get(get_all_products))
        .route("/products/{id}", get(get_product))
        .route("/products", post(create_product).route_layer(auth_layer.clone()))
        .route("/products/{id}", put(update_product).route_layer(auth_layer.clone()))
        .route("/products/{id}", delete(delete_product_secure).route_layer(auth_layer))
}

async fn delete_product_secure(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
    Extension(user): Extension<AuthUser>
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    if user.role != "admin" { return Err(axum::http::StatusCode::FORBIDDEN); }
    crate::handlers::product_handler::delete_product(State(conn), Path(id)).await
}
