use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use export_service::{
    domain::validators::DefaultExportValidator,
    infrastructure::auth::JwtHandler,
    infrastructure::exporters::*,
    application::use_cases::ExportUseCase,
    presentation::{
        handlers::{handle_export, health_check, get_token},
        auth::auth_middleware,
    },
    AppState,
};

#[tokio::main]
async fn main() {
    // Initialize JWT handler
    let jwt_handler = Arc::new(JwtHandler::new(
        std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key".to_string()),
        3600, // 1 hour
    ));

    // Initialize validator
    let validator = Arc::new(DefaultExportValidator);

    // Initialize exporters
    let excel_exporter = Arc::new(ExcelExporter);
    let csv_exporter = Arc::new(CsvExporter);
    let pdf_exporter = Arc::new(PdfExporter::new());

    // Initialize use case
    let use_case = Arc::new(ExportUseCase::new(
        validator,
        excel_exporter,
        csv_exporter,
        pdf_exporter,
    ));

    // Create app state
    let state = AppState {
        jwt_handler: jwt_handler.clone(),
        use_case,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/token", get(get_token))
        .route(
            "/api/export",
            post(handle_export).layer(middleware::from_fn_with_state(
                jwt_handler,
                auth_middleware,
            )),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!("🚀 Export Service running on http://127.0.0.1:3001");
    println!("📝 GET  /health             - Health check");
    println!("📝 GET  /api/auth/token     - Get JWT token");
    println!("📤 POST /api/export         - Export data (requires token)");

    axum::serve(listener, app).await.unwrap();
}
