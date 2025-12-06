# Export Service

A high-performance REST API service built with Rust and Axum for exporting data to multiple formats (Excel, CSV, PDF) with JWT authentication.

## Features

- **Multiple Export Formats**: Excel (`.xlsx`), CSV (`.csv`), PDF (`.pdf`)
- **JWT Authentication**: Secure API endpoints with JSON Web Tokens
- **Clean Architecture**: Domain-driven design with clear separation of concerns
- **Data Validation**: Comprehensive validation for headers, rows, and cell content
- **High Performance**: Built with Rust for maximum speed and safety

## Architecture

```
src/
├── domain/              # Business logic and models
│   ├── models.rs        # Domain entities (ExportData, ExportFormat, ExportOptions)
│   ├── errors.rs        # Domain errors
│   └── validators.rs    # Business rules validation
├── application/         # Use cases and ports
│   ├── ports.rs         # Service interfaces
│   ├── use_cases.rs     # Export use case
│   └── dto.rs           # Data transfer objects
├── infrastructure/      # External implementations
│   ├── exporters/       # Format-specific exporters
│   │   ├── excel.rs
│   │   ├── csv.rs
│   │   └── pdf.rs
│   └── auth/            # JWT authentication
│       └── jwt_handler.rs
├── presentation/        # HTTP layer
│   ├── handlers.rs      # Request handlers
│   ├── auth.rs          # Auth middleware
│   └── dto.rs           # Response DTOs
└── main.rs              # Application entry point
```

## Getting Started

### Prerequisites

- Rust 1.70+ (edition 2021)
- Cargo

### Installation

1. Clone the repository
2. Build the project:
```bash
cargo build
```

3. Run the server:
```bash
cargo run
```

The server will start on `http://127.0.0.1:3001`

## API Endpoints

### Health Check
```
GET /health
```

Response: `OK`

### Get Authentication Token
```
GET /api/auth/token
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### Export Data
```
POST /api/export
Authorization: Bearer <token>
Content-Type: application/json
```

Request Body:
```json
{
  "title": "Sales Report",
  "format": "excel",  // or "csv", "pdf"
  "headers": ["ID", "Name", "Amount"],
  "rows": [
    ["1", "Product A", "100"],
    ["2", "Product B", "200"],
    ["3", "Product C", "150"]
  ],
  "options": {
    "freeze_headers": true,
    "auto_fit_columns": true,
    "header_bold": true
  }
}
```

Response: Binary file with appropriate Content-Type header

## Usage Examples

### 1. Get Authentication Token
```bash
curl -X GET http://127.0.0.1:3001/api/auth/token
```

### 2. Export to Excel
```bash
curl -X POST http://127.0.0.1:3001/api/export \
  -H "Authorization: Bearer <your-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Sales Report",
    "format": "excel",
    "headers": ["ID", "Name", "Amount"],
    "rows": [
      ["1", "Product A", "100"],
      ["2", "Product B", "200"]
    ]
  }' \
  -o report.xlsx
```

### 3. Export to CSV
```bash
curl -X POST http://127.0.0.1:3001/api/export \
  -H "Authorization: Bearer <your-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Sales Report",
    "format": "csv",
    "headers": ["ID", "Name", "Amount"],
    "rows": [
      ["1", "Product A", "100"],
      ["2", "Product B", "200"]
    ]
  }' \
  -o report.csv
```

### 4. Export to PDF
```bash
curl -X POST http://127.0.0.1:3001/api/export \
  -H "Authorization: Bearer <your-token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Sales Report",
    "format": "pdf",
    "headers": ["ID", "Name", "Amount"],
    "rows": [
      ["1", "Product A", "100"],
      ["2", "Product B", "200"]
    ]
  }' \
  -o report.pdf
```

## Data Validation

The service validates:
- **Headers**: Cannot be empty, max 1000 chars per header
- **Rows**: Cannot be empty, max 10,000 rows
- **Columns**: Must match header count
- **Cells**: Max 1000 chars per cell

## Error Responses

### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Missing authorization token"
}
```

### 400 Bad Request
```json
{
  "error": "Export failed",
  "message": "Row 1: column count mismatch (expected 2, got 3)"
}
```

## Configuration

Environment variables (optional):
- `JWT_SECRET`: Secret key for JWT signing (default: "dev-secret-key")
- `JWT_EXPIRATION_SECONDS`: Token expiration time (default: 3600)

## Testing

The service has been tested with:
- ✅ Health check endpoint
- ✅ JWT token generation
- ✅ Excel export (.xlsx)
- ✅ CSV export (.csv)
- ✅ PDF export (.pdf)
- ✅ Authentication middleware
- ✅ Data validation (empty headers, column mismatch, etc.)

## Technology Stack

- **Web Framework**: Axum 0.7
- **Async Runtime**: Tokio
- **Serialization**: Serde
- **Authentication**: jsonwebtoken
- **Excel**: rust_xlsxwriter
- **CSV**: csv
- **PDF**: printpdf
- **CORS**: tower-http

## License

MIT
