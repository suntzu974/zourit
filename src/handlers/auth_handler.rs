use axum::{extract::{State, Extension, ConnectInfo}, http::{StatusCode, HeaderMap}, Json};
use std::net::SocketAddr;
use serde_json::{json, Value};
use crate::{database::SharedDatabase, auth::{RegisterUser, LoginUser, User, hash_password, verify_password, generate_token}};
use std::env;

const DEFAULT_SECRET: &str = "CHANGE_ME_DEV_SECRET";

pub async fn register(State(db): State<SharedDatabase>, Json(payload): Json<RegisterUser>) -> Result<Json<Value>, StatusCode> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_SECRET.to_string());
    if let Ok(Some(_)) = crate::auth::User::find_by_username(&db, &payload.username).await {
        return Err(StatusCode::CONFLICT);
    }
    let password_hash = hash_password(&payload.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut user = User { id: None, username: payload.username, password_hash, role: "user".to_string() };
    user.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = generate_token(user.id.unwrap(), &user.role, &secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"token": token, "role": user.role})))
}

pub async fn login(
    State(db): State<SharedDatabase>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<LoginUser>
) -> Result<Json<Value>, StatusCode> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_SECRET.to_string());
    // Gather client info
    let ip = addr.ip();
    let user_agent = headers.get(axum::http::header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("<unknown>");
    let host_reverse = dns_lookup::lookup_addr(&ip).unwrap_or_else(|_| "<reverse-dns-failed>".into());
    println!(
        "[AUTH] Login attempt user='{}' ip='{}' host='{}' ua='{}'",
        payload.username, ip, host_reverse, user_agent
    );
    match crate::auth::User::find_by_username(&db, &payload.username).await {
        Ok(Some(user)) => {
            if verify_password(&user.password_hash, &payload.password) {
                println!(
                    "[AUTH] Login success user='{}' role='{}' ip='{}' host='{}'",
                    user.username, user.role, ip, host_reverse
                );
                let token = generate_token(user.id.unwrap(), &user.role, &secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                Ok(Json(json!({"token": token, "role": user.role})))
            } else {
                println!(
                    "[AUTH] Login failed(bad-password) user='{}' ip='{}' host='{}'",
                    payload.username, ip, host_reverse
                );
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            println!(
                "[AUTH] Login failed(user-not-found) user='{}' ip='{}' host='{}'",
                payload.username, ip, host_reverse
            );
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser { pub user_id: i32, pub role: String }


pub async fn refresh_token(
    Extension(user): Extension<AuthUser>
) -> Result<Json<Value>, StatusCode> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_SECRET.to_string());
    let token = generate_token(user.user_id, &user.role, &secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"token": token, "role": user.role})))
}

pub async fn list_users(State(db): State<SharedDatabase>, Extension(user): Extension<AuthUser>) -> Result<Json<Value>, StatusCode> {
    if user.role != "admin" { return Err(StatusCode::FORBIDDEN); }
    let conn = db.connect().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut rows = conn.query("SELECT id, username, role FROM user", ()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut users = Vec::new();
    while let Some(row) = rows.next().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        users.push(json!({
            "id": row.get::<i64>(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            "username": row.get::<String>(1).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            "role": row.get::<String>(2).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }));
    }
    Ok(Json(json!({"users": users})))
}

// Create an admin user.
// Bootstrapping rule: if there is no existing admin user, endpoint can be called without auth.
// Otherwise a valid admin JWT must be provided in Authorization: Bearer <token>.
pub async fn create_admin(
    State(db): State<SharedDatabase>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<RegisterUser>
) -> Result<Json<Value>, StatusCode> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_SECRET.to_string());

    // Count existing admins
    let conn = db.connect().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut rows = conn.query("SELECT COUNT(*) FROM user WHERE role='admin'", ()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let admin_count: i64 = if let Some(row) = rows.next().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        row.get::<i64>(0).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        0
    };

    if admin_count > 0 {
        // Need admin auth
        let auth_header = headers.get(axum::http::header::AUTHORIZATION).and_then(|v| v.to_str().ok());
        let token = auth_header.and_then(|h| h.strip_prefix("Bearer ")) .ok_or(StatusCode::UNAUTHORIZED)?;
        let data = crate::auth::validate_token(token, &secret).map_err(|_| StatusCode::UNAUTHORIZED)?;
        if data.claims.role != "admin" { return Err(StatusCode::FORBIDDEN); }
    }

    // Check if username already exists
    if let Ok(Some(_)) = crate::auth::User::find_by_username(&db, &payload.username).await { return Err(StatusCode::CONFLICT); }

    let password_hash = hash_password(&payload.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut user = User { id: None, username: payload.username, password_hash, role: "admin".to_string() };
    user.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = generate_token(user.id.unwrap(), &user.role, &secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"token": token, "role": user.role})))
}
