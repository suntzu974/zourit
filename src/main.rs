mod models;
mod repository;
mod handlers;
mod routes;
mod database;
mod entity;
mod templates;

use tower_http::cors::CorsLayer;
use database::Database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_conn = Database::create_shared_connection("zourit.db")?;
    
    let app = routes::create_router()
        .layer(CorsLayer::permissive())
        .with_state(shared_conn);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
