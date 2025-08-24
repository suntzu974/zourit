pub mod product_routes;
pub mod auth_routes;

use askama::Template;
use axum::{Router, Json, routing::get, response::Html, extract::Query};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::database::SharedConnection;
use crate::templates::IndexTemplate;
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
        .merge(product_routes::create_product_routes())
        .merge(auth_routes::create_auth_routes())
}
