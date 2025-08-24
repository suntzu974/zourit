use axum::{http::{Request, StatusCode, header}, middleware::Next, response::Response, body::Body};
use crate::handlers::auth_handler::AuthUser;

pub async fn require_auth(mut req: Request<Body>) -> Result<Request<Body>, Response> {
    let auth_header = req.headers().get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    if let Some(h) = auth_header { if let Some(token) = h.strip_prefix("Bearer ") {
        match crate::auth::validate_token(token, &std::env::var("JWT_SECRET").unwrap_or_else(|_| "CHANGE_ME_DEV_SECRET".into())) {
            Ok(data) => { req.extensions_mut().insert(AuthUser { user_id: data.claims.sub, role: data.claims.role }); Ok(req) },
            Err(_) => Err(Response::builder().status(StatusCode::UNAUTHORIZED).body(axum::body::Body::empty()).unwrap())
        }
    } else { Err(Response::builder().status(StatusCode::UNAUTHORIZED).body(axum::body::Body::empty()).unwrap()) }} else { Err(Response::builder().status(StatusCode::UNAUTHORIZED).body(axum::body::Body::empty()).unwrap()) }
}

pub async fn auth_middleware(req: Request<Body>, next: Next) -> Response {
    match require_auth(req).await { Ok(req) => next.run(req).await, Err(resp) => resp }
}

pub async fn admin_middleware(req: Request<Body>, next: Next) -> Response {
    match require_auth(req).await {
        Ok(req) => {
            if let Some(user) = req.extensions().get::<crate::handlers::auth_handler::AuthUser>() {
                if user.role == "admin" { return next.run(req).await; }
            }
            Response::builder().status(StatusCode::FORBIDDEN).body(axum::body::Body::empty()).unwrap()
        }
        Err(resp) => resp,
    }
}
