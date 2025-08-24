pub mod product_routes;
pub mod auth_routes;

use askama::Template;
use axum::{Router, Json, routing::{get, post}, response::Html, extract::Query};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::database::SharedConnection;
use crate::templates::IndexTemplate;
use crate::handlers::admin_handler::{list_users_html, promote_user};
use crate::middleware::admin_middleware;
// Auth routes moved to auth_routes.rs



async fn index(Query(params): Query<HashMap<String, String>>) -> Result<Html<String>, Json<Value>> {
    // Check if client wants HTML or JSON
    if params.get("format").map(|s| s.as_str()) == Some("json") {
        Err(Json(json!({
            "message": "Welcome to Zourit API",
            "version": "1.0.0",
            "endpoints": {
                "products": "/products"
            }
        })))
    } else {
        let template = IndexTemplate::new();
        Ok(Html(template.render().unwrap()))
    }
}

pub fn create_router() -> Router<SharedConnection> {
    Router::new()
        .route("/", get(index))
    .route("/admin/users", get(list_users_html).route_layer(axum::middleware::from_fn(admin_middleware)))
    .route("/admin/users/{id}/role", post(promote_user).route_layer(axum::middleware::from_fn(admin_middleware)))
        .merge(product_routes::create_product_routes())
        .merge(auth_routes::create_auth_routes())
}
