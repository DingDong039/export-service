# Export Service - Implementation Steps (Step-by-Step)

## Phase 1: Project Setup (5 minutes)

### Step 1.1: Create Cargo Project
```bash
cd /tmp
cargo new export-service
cd export-service
```

### Step 1.2: Update Cargo.toml
Replace entire Cargo.toml with:
```toml
[package]
name = "export-service"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
jsonwebtoken = "9"
chrono = "0.4"
uuid = { version = "1", features = ["v4"] }
rust_xlsxwriter = "0.66"
csv = "1.3"
printpdf = "0.7"
thiserror = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors"] }

[dev-dependencies]
tokio-test = "0.4"
```

### Step 1.3: Create Project Structure
```bash
mkdir -p src/{domain,application,infrastructure/{exporters,auth},presentation}
```

Result:
```
src/
├── main.rs
├── lib.rs
├── domain/
├── application/
├── infrastructure/
│   ├── exporters/
│   └── auth/
└── presentation/
```

---

## Phase 2: Domain Layer (10 minutes)

### Step 2.1: Create `src/domain/mod.rs`
```rust
pub mod models;
pub mod errors;
pub mod validators;
```

### Step 2.2: Create `src/domain/models.rs`
```rust
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ExportData {
    pub title: String,
    pub format: ExportFormat,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub options: Option<ExportOptions>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Excel,
    Csv,
    Pdf,
}

impl ExportFormat {
    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Excel => "xlsx",
            ExportFormat::Csv => "csv",
            ExportFormat::Pdf => "pdf",
        }
    }

    pub fn mime_type(&self) -> &str {
        match self {
            ExportFormat::Excel => {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            }
            ExportFormat::Csv => "text/csv",
            ExportFormat::Pdf => "application/pdf",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportOptions {
    pub freeze_headers: Option<bool>,
    pub auto_fit_columns: Option<bool>,
    pub header_bold: Option<bool>,
    pub header_background: Option<String>,
    pub include_header_row: Option<bool>,
    pub delimiter: Option<String>,
}
```

### Step 2.3: Create `src/domain/errors.rs`
```rust
use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    InvalidFormat(String),
    EmptyData(String),
    ColumnCountMismatch {
        row: usize,
        expected: usize,
        actual: usize,
    },
    CellTooLong(usize),
    TooManyRows(usize),
    InvalidToken,
    TokenExpired,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DomainError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            DomainError::EmptyData(msg) => write!(f, "Empty data: {}", msg),
            DomainError::ColumnCountMismatch {
                row,
                expected,
                actual,
            } => write!(
                f,
                "Row {}: column count mismatch (expected {}, got {})",
                row, expected, actual
            ),
            DomainError::CellTooLong(len) => write!(f, "Cell content too long: {} chars", len),
            DomainError::TooManyRows(count) => write!(f, "Too many rows: {} (max 10000)", count),
            DomainError::InvalidToken => write!(f, "Invalid token"),
            DomainError::TokenExpired => write!(f, "Token expired"),
        }
    }
}

impl std::error::Error for DomainError {}
```

### Step 2.4: Create `src/domain/validators.rs`
```rust
use super::models::ExportData;
use super::errors::DomainError;

pub trait ExportValidator: Send + Sync {
    fn validate(&self, data: &ExportData) -> Result<(), DomainError>;
}

pub struct DefaultExportValidator;

impl ExportValidator for DefaultExportValidator {
    fn validate(&self, data: &ExportData) -> Result<(), DomainError> {
        // Check headers
        if data.headers.is_empty() {
            return Err(DomainError::EmptyData("Headers cannot be empty".to_string()));
        }

        // Check rows
        if data.rows.is_empty() {
            return Err(DomainError::EmptyData("Data rows cannot be empty".to_string()));
        }

        // Check row count limit
        if data.rows.len() > 10000 {
            return Err(DomainError::TooManyRows(data.rows.len()));
        }

        let header_count = data.headers.len();

        // Check each row
        for (i, row) in data.rows.iter().enumerate() {
            if row.len() != header_count {
                return Err(DomainError::ColumnCountMismatch {
                    row: i + 1,
                    expected: header_count,
                    actual: row.len(),
                });
            }

            // Check cell length
            for cell in row.iter() {
                if cell.len() > 1000 {
                    return Err(DomainError::CellTooLong(cell.len()));
                }
            }
        }

        // Check headers length
        for header in &data.headers {
            if header.len() > 1000 {
                return Err(DomainError::CellTooLong(header.len()));
            }
        }

        Ok(())
    }
}
```

---

## Phase 3: Application Layer (10 minutes)

### Step 3.1: Create `src/application/mod.rs`
```rust
pub mod ports;
pub mod use_cases;
pub mod dto;
```

### Step 3.2: Create `src/application/ports.rs`
```rust
use crate::domain::models::ExportData;

pub trait ExportService: Send + Sync {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
```

### Step 3.3: Create `src/application/use_cases.rs`
```rust
use std::sync::Arc;
use crate::domain::models::{ExportData, ExportFormat};
use crate::domain::validators::ExportValidator;
use crate::domain::errors::DomainError;
use super::ports::ExportService;

pub struct ExportUseCase {
    validator: Arc<dyn ExportValidator>,
    excel_service: Arc<dyn ExportService>,
    csv_service: Arc<dyn ExportService>,
    pdf_service: Arc<dyn ExportService>,
}

impl ExportUseCase {
    pub fn new(
        validator: Arc<dyn ExportValidator>,
        excel_service: Arc<dyn ExportService>,
        csv_service: Arc<dyn ExportService>,
        pdf_service: Arc<dyn ExportService>,
    ) -> Self {
        Self {
            validator,
            excel_service,
            csv_service,
            pdf_service,
        }
    }

    pub fn execute(&self, data: ExportData) -> Result<Vec<u8>, DomainError> {
        // Validate
        self.validator.validate(&data)?;

        // Select service
        let service = match data.format {
            ExportFormat::Excel => self.excel_service.clone(),
            ExportFormat::Csv => self.csv_service.clone(),
            ExportFormat::Pdf => self.pdf_service.clone(),
        };

        // Export
        service
            .export(&data)
            .map_err(|e| DomainError::InvalidFormat(e.to_string()))
    }
}
```

### Step 3.4: Create `src/application/dto.rs`
```rust
use serde::{Deserialize, Serialize};
use crate::domain::models::{ExportData, ExportFormat, ExportOptions};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportRequest {
    pub title: String,
    pub format: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub options: Option<ExportOptions>,
}

impl ExportRequest {
    pub fn to_domain(&self) -> Result<ExportData, String> {
        let format = match self.format.to_lowercase().as_str() {
            "excel" => ExportFormat::Excel,
            "csv" => ExportFormat::Csv,
            "pdf" => ExportFormat::Pdf,
            _ => return Err("Invalid format".to_string()),
        };

        Ok(ExportData {
            title: self.title.clone(),
            format,
            headers: self.headers.clone(),
            rows: self.rows.clone(),
            options: self.options.clone(),
        })
    }
}
```

---

## Phase 4: Infrastructure - Exporters (15 minutes)

### Step 4.1: Create `src/infrastructure/mod.rs`
```rust
pub mod exporters;
pub mod auth;
```

### Step 4.2: Create `src/infrastructure/exporters/mod.rs`
```rust
mod excel;
mod csv;
mod pdf;

pub use excel::ExcelExporter;
pub use csv::CsvExporter;
pub use pdf::PdfExporter;
```

### Step 4.3: Create `src/infrastructure/exporters/excel.rs`
```rust
use rust_xlsxwriter::*;
use crate::application::ports::ExportService;
use crate::domain::models::ExportData;

pub struct ExcelExporter;

impl ExportService for ExcelExporter {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Write headers
        for (col, header) in data.headers.iter().enumerate() {
            worksheet.write_string(0, col as u16, header)?;
            worksheet.set_column_width(col as u16, 20)?;
        }

        // Write data
        for (row_idx, row) in data.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                worksheet.write_string((row_idx + 1) as u32, col_idx as u16, cell)?;
            }
        }

        // Freeze headers if requested
        if let Some(opts) = &data.options {
            if opts.freeze_headers.unwrap_or(false) {
                worksheet.freeze_panes(1, 0)?;
            }
        }

        workbook
            .save_to_buffer()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
```

### Step 4.4: Create `src/infrastructure/exporters/csv.rs`
```rust
use csv::Writer;
use crate::application::ports::ExportService;
use crate::domain::models::ExportData;

pub struct CsvExporter;

impl ExportService for CsvExporter {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        let mut writer = Writer::from_writer(&mut buffer);

        // Write headers
        writer.write_record(&data.headers)?;

        // Write rows
        for row in &data.rows {
            writer.write_record(row)?;
        }

        writer.flush()?;
        Ok(buffer)
    }
}
```

### Step 4.5: Create `src/infrastructure/exporters/pdf.rs`
```rust
use printpdf::*;
use crate::application::ports::ExportService;
use crate::domain::models::ExportData;

pub struct PdfExporter;

impl ExportService for PdfExporter {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut doc = PdfDocument::new("Export");
        let page_id = doc.add_page(Mm(210.0), Mm(297.0), "Page 1");
        let page = doc.get_page_mut(page_id);
        let font = doc.add_builtin_font(BuiltinFont::TimesRoman)?;

        let mut y = 280.0;

        // Title
        page.use_text(&data.title, 14.0, Mm(10.0), Mm(y), &font);
        y -= 8.0;

        // Headers
        let headers_str = data.headers.join(" | ");
        page.use_text(&headers_str, 11.0, Mm(10.0), Mm(y), &font);
        y -= 4.0;

        // Data rows
        for row in data.rows.iter().take(30) {
            let row_str = row.join(" | ");
            page.use_text(&row_str, 10.0, Mm(10.0), Mm(y), &font);
            y -= 3.0;

            if y < 15.0 {
                break;
            }
        }

        doc.save_to_bytes()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
```

---

## Phase 5: Infrastructure - Auth (10 minutes)

### Step 5.1: Create `src/infrastructure/auth/mod.rs`
```rust
mod jwt_handler;

pub use jwt_handler::{JwtHandler, Claims};
```

### Step 5.2: Create `src/infrastructure/auth/jwt_handler.rs`
```rust
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtHandler {
    secret: String,
    expiration: i64,
}

impl JwtHandler {
    pub fn new(secret: String, expiration: i64) -> Self {
        Self { secret, expiration }
    }

    pub fn generate_token(&self) -> String {
        let now = Utc::now().timestamp();
        let claims = Claims {
            iss: "export-service".to_string(),
            sub: "web-client".to_string(),
            exp: now + self.expiration,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap_or_default()
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| e.to_string())
    }
}
```

---

## Phase 6: Presentation Layer (15 minutes)

### Step 6.1: Create `src/presentation/mod.rs`
```rust
pub mod handlers;
pub mod auth;
pub mod dto;
```

### Step 6.2: Create `src/presentation/dto.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: i64,
    pub token_type: String,
}
```

### Step 6.3: Create `src/presentation/auth.rs`
```rust
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use std::sync::Arc;
use crate::infrastructure::auth::JwtHandler;

pub async fn auth_middleware(
    State(jwt_handler): State<Arc<JwtHandler>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
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
```

### Step 6.4: Create `src/presentation/handlers.rs`
```rust
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use crate::{AppState, application::dto::ExportRequest};

pub async fn health_check() -> &'static str {
    "✅ OK"
}

pub async fn get_token(State(state): State<AppState>) -> Json<crate::presentation::dto::TokenResponse> {
    let token = state.jwt_handler.generate_token();
    Json(crate::presentation::dto::TokenResponse {
        token,
        expires_in: 3600,
        token_type: "Bearer".to_string(),
    })
}

pub async fn handle_export(
    State(state): State<AppState>,
    Json(req): Json<ExportRequest>,
) -> Response {
    // Convert to domain
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
            let filename = format!(
                "{}_{}.{}",
                data.title.replace(" ", "_"),
                chrono::Utc::now().timestamp(),
                data.format.extension()
            );

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
```

---

## Phase 7: Main Setup (10 minutes)

### Step 7.1: Create `src/lib.rs`
```rust
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
```

### Step 7.2: Create `src/main.rs`
```rust
use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

mod domain;
mod application;
mod infrastructure;
mod presentation;

use domain::validators::DefaultExportValidator;
use infrastructure::auth::JwtHandler;
use infrastructure::exporters::*;
use application::use_cases::ExportUseCase;
use presentation::{
    handlers::{handle_export, health_check, get_token},
    auth::auth_middleware,
    dto::TokenResponse,
};

#[derive(Clone)]
pub struct AppState {
    jwt_handler: Arc<JwtHandler>,
    use_case: Arc<ExportUseCase>,
}

#[tokio::main]
async fn main() {
    // Initialize
    let jwt_handler = Arc::new(JwtHandler::new(
        std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key".to_string()),
        3600,
    ));

    let validator = Arc::new(DefaultExportValidator);
    let excel_exporter = Arc::new(ExcelExporter);
    let csv_exporter = Arc::new(CsvExporter);
    let pdf_exporter = Arc::new(PdfExporter);

    let use_case = Arc::new(ExportUseCase::new(
        validator,
        excel_exporter,
        csv_exporter,
        pdf_exporter,
    ));

    let state = AppState { jwt_handler, use_case };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/auth/token", get(get_token))
        .route(
            "/api/export",
            post(handle_export).layer(middleware::from_fn_with_state(
                state.jwt_handler.clone(),
                auth_middleware,
            )),
        )
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("✅ Export Service running on http://127.0.0.1:3000");
    println!("📝 GET  /api/auth/token     - Get JWT token");
    println!("📤 POST /api/export        - Export data (requires token)");

    axum::serve(listener, app).await.unwrap();
}
```

---

## Phase 8: Environment Setup (2 minutes)

### Step 8.1: Create `.env` file
```bash
JWT_SECRET=dev-secret-key-change-in-production
JWT_EXPIRATION_SECONDS=3600
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
LOG_LEVEL=info
```

---

## Phase 9: Test Build (5 minutes)

```bash
# Build project
cargo build

# If errors, check:
# 1. All files created in correct locations
# 2. Module declarations match file names
# 3. All imports are correct

# Build release
cargo build --release
```

---

## Phase 10: Run & Test (5 minutes)

```bash
# Terminal 1: Start server
cargo run

# Terminal 2: Test API
# Get token
curl http://localhost:3000/api/auth/token

# Export Excel
TOKEN="<paste-token-here>"
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test Report",
    "format": "excel",
    "headers": ["Product", "Q1", "Q2"],
    "rows": [
      ["Product A", "100", "150"],
      ["Product B", "120", "140"]
    ],
    "options": {
      "freeze_headers": true,
      "auto_fit_columns": true,
      "header_bold": true
    }
  }' -o report.xlsx

# Check file
file report.xlsx
ls -lh report.xlsx
```

---

## Summary

| Phase | Task | Time |
|-------|------|------|
| 1 | Project setup | 5 min |
| 2 | Domain layer | 10 min |
| 3 | Application layer | 10 min |
| 4 | Exporters | 15 min |
| 5 | Auth | 10 min |
| 6 | Presentation | 15 min |
| 7 | Main setup | 10 min |
| 8 | .env config | 2 min |
| 9 | Build | 5 min |
| 10 | Test | 5 min |
| **Total** | | **~90 min** |

**Ready? Let's code! 🚀**
