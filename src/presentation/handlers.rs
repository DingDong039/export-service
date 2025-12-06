use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use crate::application::dto::ExportRequest;

/// Health check endpoint
pub async fn health_check() -> &'static str {
    "OK"
}

/// Get JWT token
pub async fn get_token(
    State(state): State<crate::AppState>
) -> Json<crate::presentation::dto::TokenResponse> {
    let token = state.jwt_handler.generate_token();
    Json(crate::presentation::dto::TokenResponse {
        token,
        expires_in: state.jwt_handler.expiration(),
        token_type: "Bearer".to_string(),
    })
}

/// Handle export request
pub async fn handle_export(
    State(state): State<crate::AppState>,
    Json(req): Json<ExportRequest>,
) -> Response {
    // Convert DTO to domain model
    let data = match req.to_domain() {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid format",
                    "message": e
                })),
            )
                .into_response();
        }
    };

    // Execute use case
    match state.use_case.execute(data.clone()) {
        Ok(bytes) => {
            // Generate filename
            let filename = format!(
                "{}_{}.{}",
                data.title.replace(" ", "_"),
                chrono::Utc::now().timestamp(),
                data.format.extension()
            );

            // Return binary file
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, data.format.mime_type()),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                Body::from(bytes),
            )
                .into_response()
        }
        Err(e) => {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Export failed",
                    "message": e.to_string()
                })),
            )
                .into_response()
        }
    }
}
