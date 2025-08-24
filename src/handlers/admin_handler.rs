use axum::{
    extract::{State, Path},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, Redirect, IntoResponse, Response},
    Form,
};
use serde::Deserialize;
use rusqlite::params;
use crate::database::SharedConnection;
use askama::Template;
use rand::distributions::{Alphanumeric, DistString};

#[derive(Template)]
#[template(path = "admin/users.html")]
struct UsersTemplate {
    users: Vec<UserRow>,
    error: Option<String>,
    message: Option<String>,
    csrf_token: String,
}

#[derive(Clone)]
struct UserRow { id: i32, username: String, role: String }

#[derive(Deserialize)]
pub struct PromoteForm { pub role: String, pub csrf_token: String }

// Simple cookie parsing helper (no extra dependencies)
fn find_cookie(cookies: &str, name: &str) -> Option<String> {
    let prefix = format!("{}=", name);
    cookies.split(';').map(|c| c.trim()).find_map(|c| c.strip_prefix(&prefix).map(|v| v.to_string()))
}

pub async fn list_users_html(
    State(conn): State<SharedConnection>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let conn = conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, username, role FROM user ORDER BY id")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map([], |r| Ok(UserRow { id: r.get(0)?, username: r.get(1)?, role: r.get(2)? }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut users = Vec::new();
    for r in rows { users.push(r.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?); }

    let cookie_header = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("");
    let mut csrf_token = find_cookie(cookie_header, "csrf_token");
    let mut set_cookie_header = None;
    if csrf_token.is_none() {
        // Generate new random token (32 chars)
        let new_tok = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
        csrf_token = Some(new_tok.clone());
        // HttpOnly so scripts can't read; SameSite=Strict to reduce CSRF risk
        set_cookie_header = Some(format!("csrf_token={}; Path=/; HttpOnly; SameSite=Strict", new_tok));
    }
    let csrf_token = csrf_token.unwrap();

    let tpl = UsersTemplate { users, error: None, message: None, csrf_token: csrf_token.clone() };
    let body = tpl.render().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut response = Html(body).into_response();
    if let Some(set_cookie) = set_cookie_header {
        if let Ok(val) = HeaderValue::from_str(&set_cookie) { response.headers_mut().append(header::SET_COOKIE, val); }
    }
    Ok(response)
}

pub async fn promote_user(
    State(conn): State<SharedConnection>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    Form(form): Form<PromoteForm>
) -> Result<Redirect, StatusCode> {
    if form.role != "admin" && form.role != "user" { return Err(StatusCode::BAD_REQUEST); }

    // Double-submit cookie validation
    let cookie_header = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()).unwrap_or("");
    let cookie_token = find_cookie(cookie_header, "csrf_token").ok_or(StatusCode::FORBIDDEN)?;
    if cookie_token != form.csrf_token { return Err(StatusCode::FORBIDDEN); }

    let conn = conn.lock().unwrap();
    conn.execute("UPDATE user SET role = ?1 WHERE id = ?2", params![form.role, id])
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Redirect::to("/admin/users"))
}