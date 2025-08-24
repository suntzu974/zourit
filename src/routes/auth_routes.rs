use axum::{
    Router,
    routing::{get, post},
    extract::Extension,
};
use crate::database::SharedConnection;
use crate::handlers::auth_handler::{register, login, refresh_token, list_users, create_admin, AuthUser};
use crate::middleware::{auth_middleware, admin_middleware};

// GET /auth/me handler (local to auth routes)
async fn me_user(Extension(user): Extension<AuthUser>) -> axum::Json<serde_json::Value> {
    use serde_json::json; axum::Json(json!({"user_id": user.user_id, "role": user.role}))
}

pub fn create_auth_routes() -> Router<SharedConnection> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/me", get(me_user).route_layer(axum::middleware::from_fn(auth_middleware)))
        .route("/auth/refresh", get(refresh_token).route_layer(axum::middleware::from_fn(auth_middleware)))
        .route("/auth/users", get(list_users).route_layer(axum::middleware::from_fn(admin_middleware)))
    .route("/auth/admin", post(create_admin))
}
