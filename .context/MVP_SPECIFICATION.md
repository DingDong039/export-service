# Export Service - MVP Specification

## Project Overview

**Objective**: Lightweight export service (Excel, CSV, PDF) for web client with basic auth token

**Timeline**: No rush - iterative development  
**Data Volume**: < 1000 rows  
**Deployment**: Binary file sent directly to web client  
**Storage**: In-memory processing (no persistence)

---

## Architecture

```
┌─────────────────────────────────┐
│   Web Client                    │
│  (sends data + config)          │
└────────────┬────────────────────┘
             │ POST /api/export
             │ with token auth
             │
┌────────────▼────────────────────┐
│   Export Service (Rust)         │
│  ┌────────────────────────────┐ │
│  │ 1. Validate token         │ │
│  │ 2. Validate data          │ │
│  │ 3. Export (Excel/CSV/PDF) │ │
│  │ 4. Return binary file     │ │
│  └────────────────────────────┘ │
└─────────────────────────────────┘
```

---

## 1. Authentication (Basic Token)

### Token Flow

```
1. Web Client → Service: GET /api/auth/token
   (initial request, or service-to-service handshake)

2. Service → Web Client: 
   {
     "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
     "expires_in": 3600,
     "type": "Bearer"
   }

3. Web Client → Service: POST /api/export
   Headers: {
     "Authorization": "Bearer <token>",
     "Content-Type": "application/json"
   }

4. Service: 
   - Validate token signature
   - Check expiration
   - Process export request
```

### Token Details

- **Type**: JWT (JSON Web Token)
- **Algorithm**: HS256 (HMAC SHA-256)
- **Secret**: Environment variable (e.g., `JWT_SECRET`)
- **Expiration**: 1 hour (configurable)
- **Payload**:
  ```json
  {
    "iss": "export-service",
    "exp": 1234567890,
    "iat": 1234567890,
    "sub": "web-client"
  }
  ```

---

## 2. API Endpoints

### 2.1 Get Token
```http
GET /api/auth/token
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### 2.2 Export Data
```http
POST /api/export
Authorization: Bearer <token>
Content-Type: application/json
```

**Request Body:**
```json
{
  "title": "Sales Report",
  "format": "excel",
  "headers": ["Product", "Q1", "Q2", "Q3"],
  "rows": [
    ["Product A", "100", "150", "200"],
    ["Product B", "120", "140", "180"]
  ],
  "options": {
    "freeze_headers": true,
    "auto_fit_columns": true,
    "header_bold": true,
    "include_header_row": true
  }
}
```

**Response (Binary):**
- Content-Type: `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` (Excel)
- Content-Type: `text/csv` (CSV)
- Content-Type: `application/pdf` (PDF)
- Content-Disposition: `attachment; filename="Sales_Report.xlsx"`
- Body: Binary file data

**Error Response (401):**
```json
{
  "error": "Unauthorized",
  "message": "Invalid or expired token"
}
```

**Error Response (400):**
```json
{
  "error": "Bad Request",
  "message": "Headers and rows length mismatch at row 1"
}
```

---

## 3. Data Model (Web Client sends)

### Request DTO

```rust
pub struct ExportRequest {
    pub title: String,                    // File name
    pub format: String,                   // "excel", "csv", "pdf"
    pub headers: Vec<String>,             // Column names
    pub rows: Vec<Vec<String>>,           // Data rows
    pub options: Option<ExportOptions>,   // Optional styling
}

pub struct ExportOptions {
    pub freeze_headers: Option<bool>,      // Freeze first row (Excel)
    pub auto_fit_columns: Option<bool>,    // Auto column width (Excel)
    pub header_bold: Option<bool>,         // Bold headers
    pub header_background: Option<String>, // Hex color
    pub include_header_row: Option<bool>,  // Include headers in output
    pub delimiter: Option<String>,         // CSV only: comma, semicolon, tab
}
```

### Example Requests

#### Simple CSV
```json
{
  "title": "Data Export",
  "format": "csv",
  "headers": ["Name", "Email"],
  "rows": [
    ["John", "john@example.com"],
    ["Jane", "jane@example.com"]
  ]
}
```

#### Advanced Excel
```json
{
  "title": "Sales Report Q1",
  "format": "excel",
  "headers": ["Product", "Sales", "Target", "Status"],
  "rows": [
    ["Product A", "1000", "1200", "Low"],
    ["Product B", "1500", "1400", "High"]
  ],
  "options": {
    "freeze_headers": true,
    "auto_fit_columns": true,
    "header_bold": true,
    "header_background": "CCCCCC"
  }
}
```

#### Simple PDF
```json
{
  "title": "Report",
  "format": "pdf",
  "headers": ["Column1", "Column2"],
  "rows": [
    ["Value1", "Value2"]
  ]
}
```

---

## 4. Validation Rules

### Data Validation

- [ ] Headers must not be empty
- [ ] At least one data row required
- [ ] Header count must match row column count for all rows
- [ ] String length limit: 1000 chars per cell
- [ ] Row count limit: 10,000 rows
- [ ] No null/None values (convert to empty string)
- [ ] Format must be: "excel", "csv", or "pdf"

### Example Error Handling

```rust
// Validator Result
Err("Row 5: Expected 4 columns, got 3")
Err("Headers cannot be empty")
Err("Invalid format: json (valid: excel, csv, pdf)")
Err("Maximum 10000 rows allowed")
```

---

## 5. Export Features

### Excel (.xlsx)

**MVP Features:**
- ✅ Basic data export
- ✅ Bold headers
- ✅ Auto-fit columns
- ✅ Borders
- ✅ Center alignment for headers
- ✅ Number formatting (auto-detect)
- ✅ Date formatting (if date format detected)

**Not in MVP:**
- ❌ Multiple sheets
- ❌ Formulas/Functions
- ❌ Conditional formatting
- ❌ Charts/Graphs

**Excel Output Example:**
```
┌─────────┬────┬────┬────┐
│ Product │ Q1 │ Q2 │ Q3 │  ← Bold, centered
├─────────┼────┼────┼────┤
│ Product A│ 100│ 150│ 200│
│ Product B│ 120│ 140│ 180│
└─────────┴────┴────┴────┘
```

### CSV (.csv)

**MVP Features:**
- ✅ Standard comma delimiter
- ✅ Proper quote escaping
- ✅ UTF-8 encoding
- ✅ Header row included
- ✅ Custom delimiter option (semicolon, tab)

**Output Example:**
```csv
Product,Q1,Q2,Q3
Product A,100,150,200
Product B,120,140,180
```

### PDF (.pdf)

**MVP Features:**
- ✅ Simple table layout
- ✅ Title/header section
- ✅ Column headers
- ✅ Data rows
- ✅ Basic formatting
- ✅ Page breaks for long data

**Not in MVP:**
- ❌ Multiple columns
- ❌ Footers/page numbers
- ❌ Charts/graphs
- ❌ Complex styling

---

## 6. API Response Types

### Success Response

**Headers:**
```
HTTP/1.1 200 OK
Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet
Content-Disposition: attachment; filename="Sales_Report.xlsx"
Content-Length: 5432
```

**Body:** Binary file data

### Error Responses

**400 - Validation Error**
```json
{
  "error": "ValidationError",
  "message": "Row 3: column count mismatch (expected 4, got 3)",
  "details": {
    "row_index": 3,
    "expected_columns": 4,
    "actual_columns": 3
  }
}
```

**401 - Authentication Error**
```json
{
  "error": "Unauthorized",
  "message": "Token expired",
  "code": "TOKEN_EXPIRED"
}
```

**500 - Server Error**
```json
{
  "error": "InternalServerError",
  "message": "Export failed: memory allocation error"
}
```

---

## 7. Code Structure (MVP)

```
src/
├── main.rs                          # Entry point, server setup
├── lib.rs                           # Library exports
│
├── domain/
│   ├── mod.rs
│   ├── models.rs                    # ExportData, ExportFormat
│   ├── errors.rs                    # DomainError
│   └── validators.rs                # Validate request
│
├── application/
│   ├── mod.rs
│   ├── ports.rs                     # Traits
│   └── use_cases.rs                 # ExportUseCase
│
├── infrastructure/
│   ├── exporters/
│   │   ├── mod.rs
│   │   ├── excel.rs                 # ExcelExporter
│   │   ├── csv.rs                   # CsvExporter
│   │   └── pdf.rs                   # PdfExporter
│   └── auth/
│       ├── mod.rs
│       └── jwt_handler.rs           # JWT token generation/validation
│
└── presentation/
    ├── handlers.rs                  # HTTP handlers
    ├── auth.rs                      # Auth middleware
    └── dto.rs                       # Request/Response DTOs
```

---

## 8. Dependencies (MVP)

```toml
[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
jsonwebtoken = "9"                   # JWT token handling
chrono = "0.4"                       # DateTime
uuid = { version = "1", features = ["v4"] }

# Export libraries
rust_xlsxwriter = "0.66"             # Excel
csv = "1.3"                          # CSV
printpdf = "0.7"                     # PDF

# Utilities
thiserror = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace"] }
```

---

## 9. Environment Configuration

**.env file:**
```bash
# JWT Configuration
JWT_SECRET=your-super-secret-key-change-this-in-production
JWT_EXPIRATION_SECONDS=3600

# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Logging
LOG_LEVEL=info
```

---

## 10. Example Web Client Integration

### JavaScript/TypeScript Example

```javascript
// 1. Get token
const tokenResponse = await fetch('http://localhost:3000/api/auth/token');
const { token } = await tokenResponse.json();

// 2. Prepare export data
const exportData = {
  title: "Sales Report",
  format: "excel",
  headers: ["Product", "Q1", "Q2"],
  rows: [
    ["Product A", "100", "150"],
    ["Product B", "120", "140"]
  ],
  options: {
    freeze_headers: true,
    auto_fit_columns: true,
    header_bold: true
  }
};

// 3. Send export request
const response = await fetch('http://localhost:3000/api/export', {
  method: 'POST',
  headers: {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json'
  },
  body: JSON.stringify(exportData)
});

// 4. Download file
const blob = await response.blob();
const url = window.URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = 'sales-report.xlsx';
a.click();
```

---

## 11. Testing Strategy (MVP)

### Unit Tests

```bash
# Test validators
cargo test validator_tests

# Test exporters
cargo test excel_exporter
cargo test csv_exporter
cargo test pdf_exporter

# Test JWT handling
cargo test jwt_tests

# Test error handling
cargo test error_handling
```

### Integration Tests

```bash
# Full export flow
cargo test integration_tests

# Auth flow
cargo test auth_integration
```

### Manual Testing

```bash
# Start server
cargo run

# Get token
curl http://localhost:3000/api/auth/token

# Export Excel
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test",
    "format": "excel",
    "headers": ["Name", "Value"],
    "rows": [["John", "100"]]
  }' \
  -o export.xlsx
```

---

## 12. Performance Targets (MVP)

| Operation | Target | Notes |
|-----------|--------|-------|
| CSV (1000 rows) | < 50ms | In-memory |
| Excel (1000 rows) | < 200ms | In-memory |
| PDF (1000 rows) | < 500ms | In-memory |
| Token generation | < 10ms | Cached |
| Token validation | < 5ms | Per request |

---

## 13. Error Handling (MVP)

**Domain Errors:**
```rust
pub enum DomainError {
    InvalidFormat(String),
    EmptyData(String),
    ColumnCountMismatch { row: usize, expected: usize, actual: usize },
    InvalidToken,
    TokenExpired,
}
```

**HTTP Status Codes:**
| Status | Meaning | When |
|--------|---------|------|
| 200 | Success | Export generated |
| 400 | Bad Request | Validation error |
| 401 | Unauthorized | Invalid/expired token |
| 500 | Server Error | Export failed |

---

## 14. Deployment (MVP)

### Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/export-service /usr/local/bin/
ENV JWT_SECRET=change-me-in-production
ENV SERVER_PORT=3000
EXPOSE 3000
CMD ["export-service"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  export-service:
    build: .
    ports:
      - "3000:3000"
    environment:
      JWT_SECRET: ${JWT_SECRET:-dev-secret}
      SERVER_PORT: 3000
      LOG_LEVEL: info
```

---

## 15. MVP Checklist

### Phase 1: Core Implementation
- [ ] Project setup with Cargo.toml
- [ ] Domain models and validators
- [ ] JWT token generation/validation
- [ ] CSV exporter
- [ ] Excel exporter (basic)
- [ ] PDF exporter (simple)
- [ ] HTTP handlers and middleware
- [ ] Error handling
- [ ] Basic unit tests

### Phase 2: Polish
- [ ] Integration tests
- [ ] Documentation
- [ ] Docker setup
- [ ] Performance optimization
- [ ] Comprehensive error messages

### Phase 3: Long-term Enhancements
- [ ] Advanced Excel formatting (formulas, conditional)
- [ ] Multiple worksheets
- [ ] PDF advanced features
- [ ] Export templates
- [ ] Async job processing
- [ ] Database persistence
- [ ] User audit logging
- [ ] RBAC support

---

## 16. Next Steps

1. **Setup**: Create Cargo project structure
2. **Auth**: Implement JWT token handling
3. **Exporters**: Start with CSV (simplest)
4. **API**: Create HTTP endpoints
5. **Testing**: Write unit + integration tests
6. **Documentation**: API docs + examples
7. **Deployment**: Docker + deployment guide

---

## Summary

**MVP Scope:**
- ✅ 3 export formats (Excel, CSV, PDF) - basic versions
- ✅ JWT token authentication
- ✅ Direct binary file download
- ✅ No database/persistence
- ✅ Data validation
- ✅ Error handling
- ✅ < 1000 rows support

**Timeline:** Iterative, no rush  
**Complexity:** Medium (auth + 3 exporters)  
**Code Lines:** ~2000-3000 lines (well-structured)
