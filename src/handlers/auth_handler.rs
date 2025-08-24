use axum::{extract::{State, Extension}, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::{database::SharedConnection, auth::{RegisterUser, LoginUser, User, hash_password, verify_password, generate_token}};
use std::env;

const DEFAULT_SECRET: &str = "CHANGE_ME_DEV_SECRET";

pub async fn register(State(conn): State<SharedConnection>, Json(payload): Json<RegisterUser>) -> Result<Json<Value>, StatusCode> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_SECRET.to_string());
    let conn = conn.lock().unwrap();
    if let Ok(Some(_)) = crate::auth::User::find_by_username(&conn, &payload.username) {
        return Err(StatusCode::CONFLICT);
    }
    let password_hash = hash_password(&payload.password).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut user = User { id: None, username: payload.username, password_hash, role: "user".to_string() };
    user.insert(&conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = generate_token(user.id.unwrap(), &user.role, &secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!({"token": token, "role": user.role})))
}

pub async fn login(State(conn): State<SharedConnection>, Json(payload): Json<LoginUser>) -> Result<Json<Value>, StatusCode> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_SECRET.to_string());
    let conn = conn.lock().unwrap();
    match crate::auth::User::find_by_username(&conn, &payload.username) {
        Ok(Some(user)) => {
            if verify_password(&user.password_hash, &payload.password) {
                let token = generate_token(user.id.unwrap(), &user.role, &secret).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                Ok(Json(json!({"token": token, "role": user.role})))
            } else { Err(StatusCode::UNAUTHORIZED) }
        }
        _ => Err(StatusCode::UNAUTHORIZED)
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

pub async fn list_users(State(conn): State<SharedConnection>, Extension(user): Extension<AuthUser>) -> Result<Json<Value>, StatusCode> {
    if user.role != "admin" { return Err(StatusCode::FORBIDDEN); }
    let conn = conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, username, role FROM user") .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt.query_map([], |row| {
        Ok(json!({"id": row.get::<_, i32>(0)?, "username": row.get::<_, String>(1)?, "role": row.get::<_, String>(2)?}))
    }).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut users = Vec::new();
    for r in rows { users.push(r.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?); }
    Ok(Json(json!({"users": users})))
}
