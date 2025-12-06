use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use std::sync::Arc;
use crate::infrastructure::auth::JwtHandler;

/// Auth middleware
pub async fn auth_middleware(
    State(jwt_handler): State<Arc<JwtHandler>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract token from Authorization header
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|auth_header| auth_header.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Missing authorization token"
                })),
            )
        })?;

    // Validate token
    jwt_handler.validate_token(token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Invalid or expired token"
            })),
        )
    })?;

    Ok(next.run(request).await)
}
