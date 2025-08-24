mod models;
mod repository;
mod handlers;
mod routes;
mod database;
mod entity;
mod templates;
mod auth;
mod middleware;

use tower_http::cors::CorsLayer;
use database::Database;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let db_path = env::var("DATABASE_PATH").unwrap_or_else(|_| "zourit.db".into());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".into());
    let addr = format!("0.0.0.0:{}", port);
    let shared_conn = Database::create_shared_connection(&db_path)?;
    
    let app = routes::create_router()
        .layer(CorsLayer::permissive())
        .with_state(shared_conn);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server running on http://{}", listener.local_addr()?.to_string());
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
