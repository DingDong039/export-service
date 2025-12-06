# MVP Setup & Testing Guide

## 1. Project Setup

### Create Project
```bash
cargo new export-service
cd export-service
```

### Update Cargo.toml
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
```

### Create Project Structure
```bash
mkdir -p src/{domain,application,infrastructure/{exporters,auth},presentation}
```

### Copy Implementation Files
- Copy code from `MVP_IMPLEMENTATION.rs`
- Split into separate files following the structure above
- Or copy entire content into appropriate modules

---

## 2. Environment Setup

### Create .env file
```bash
# .env
JWT_SECRET=dev-secret-key-change-in-production
JWT_EXPIRATION_SECONDS=3600
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
LOG_LEVEL=info
```

### Load environment variables
```bash
# Install dotenv
cargo add dotenv

# In main.rs:
dotenv::dotenv().ok();
```

---

## 3. Running the Service

### Start the server
```bash
cargo run

# Output:
# ✅ Export Service running on http://127.0.0.1:3000
# 📝 GET  /api/auth/token     - Get JWT token
# 📤 POST /api/export        - Export data (requires token)
```

---

## 4. API Testing

### Option 1: Using cURL

#### Get Token
```bash
curl http://localhost:3000/api/auth/token

# Response:
# {
#   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
#   "expires_in": 3600,
#   "token_type": "Bearer"
# }
```

#### Export as CSV
```bash
TOKEN="<token-from-above>"

curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Sales Report",
    "format": "csv",
    "headers": ["Product", "Q1", "Q2"],
    "rows": [
      ["Product A", "100", "150"],
      ["Product B", "120", "140"]
    ]
  }' \
  -o sales_report.csv

# Output: sales_report.csv
```

#### Export as Excel
```bash
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Sales Report",
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
  }' \
  -o sales_report.xlsx
```

#### Export as PDF
```bash
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Sales Report",
    "format": "pdf",
    "headers": ["Product", "Q1", "Q2"],
    "rows": [
      ["Product A", "100", "150"],
      ["Product B", "120", "140"]
    ]
  }' \
  -o sales_report.pdf
```

### Option 2: Using Postman

1. **GET /api/auth/token**
   - Method: GET
   - URL: `http://localhost:3000/api/auth/token`
   - Copy token from response

2. **POST /api/export**
   - Method: POST
   - URL: `http://localhost:3000/api/export`
   - Headers:
     ```
     Authorization: Bearer <token>
     Content-Type: application/json
     ```
   - Body (JSON):
     ```json
     {
       "title": "Sales Report",
       "format": "excel",
       "headers": ["Product", "Q1", "Q2", "Q3"],
       "rows": [
         ["Product A", "100", "150", "200"],
         ["Product B", "120", "140", "180"],
         ["Product C", "90", "110", "130"]
       ],
       "options": {
         "freeze_headers": true,
         "auto_fit_columns": true,
         "header_bold": true
       }
     }
     ```

### Option 3: Using JavaScript/TypeScript

```javascript
// 1. Get token
async function getToken() {
  const response = await fetch('http://localhost:3000/api/auth/token');
  const data = await response.json();
  return data.token;
}

// 2. Export data
async function exportData(token, format, data) {
  const response = await fetch('http://localhost:3000/api/export', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      title: 'Report',
      format: format,
      headers: data.headers,
      rows: data.rows,
      options: {
        freeze_headers: true,
        auto_fit_columns: true,
        header_bold: true,
      },
    }),
  });

  if (!response.ok) {
    const error = await response.json();
    console.error('Export failed:', error);
    return null;
  }

  return await response.blob();
}

// 3. Download file
function downloadFile(blob, filename) {
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  window.URL.revokeObjectURL(url);
}

// Usage
async function main() {
  const token = await getToken();
  
  const data = {
    headers: ['Product', 'Q1', 'Q2'],
    rows: [
      ['Product A', '100', '150'],
      ['Product B', '120', '140'],
    ],
  };

  const blob = await exportData(token, 'excel', data);
  if (blob) {
    downloadFile(blob, 'report.xlsx');
  }
}

main();
```

---

## 5. Testing

### Unit Tests

Create `tests/unit_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use export_service::domain::validators::ExportValidator;
    use export_service::domain::models::*;

    #[test]
    fn test_validator_rejects_empty_headers() {
        let data = ExportData {
            title: "Test".to_string(),
            format: ExportFormat::Csv,
            headers: vec![],
            rows: vec![vec!["data".to_string()]],
            options: None,
        };

        // Should fail
        assert!(validator.validate(&data).is_err());
    }

    #[test]
    fn test_validator_accepts_valid_data() {
        let data = ExportData {
            title: "Test".to_string(),
            format: ExportFormat::Csv,
            headers: vec!["Name".to_string()],
            rows: vec![vec!["John".to_string()]],
            options: None,
        };

        // Should succeed
        assert!(validator.validate(&data).is_ok());
    }
}
```

### Run Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_validator_rejects_empty_headers

# Run with output
cargo test -- --nocapture

# Run only unit tests
cargo test --lib
```

---

## 6. Build & Deploy

### Build Release
```bash
cargo build --release

# Output: target/release/export-service
```

### Docker Build

**Dockerfile:**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/export-service /usr/local/bin/
ENV JWT_SECRET=production-secret
ENV SERVER_PORT=3000
EXPOSE 3000
CMD ["export-service"]
```

**Build & Run:**
```bash
# Build image
docker build -t export-service:latest .

# Run container
docker run -p 3000:3000 \
  -e JWT_SECRET=your-secret-key \
  export-service:latest
```

---

## 7. Error Testing

### Test Error Cases

#### Invalid Token
```bash
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer invalid_token" \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","format":"csv","headers":["A"],"rows":[["1"]]}'

# Response 401:
# {
#   "error": "Unauthorized",
#   "message": "Invalid or expired token"
# }
```

#### Missing Authorization Header
```bash
curl -X POST http://localhost:3000/api/export \
  -H "Content-Type: application/json" \
  -d '{"title":"Test","format":"csv","headers":["A"],"rows":[["1"]]}'

# Response 401:
# {
#   "error": "Unauthorized",
#   "message": "Missing authorization token"
# }
```

#### Column Count Mismatch
```bash
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test",
    "format": "csv",
    "headers": ["A", "B"],
    "rows": [["1"]]
  }'

# Response 400:
# {
#   "error": "Export failed",
#   "message": "Row 1: column count mismatch (expected 2, got 1)"
# }
```

#### Invalid Format
```bash
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test",
    "format": "json",
    "headers": ["A"],
    "rows": [["1"]]
  }'

# Response 400:
# {
#   "error": "Invalid format",
#   "message": "Invalid format"
# }
```

---

## 8. Performance Testing

### Test with Different Sizes

#### Small (10 rows)
```bash
# Generate test data with 10 rows
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Small Test",
    "format": "excel",
    "headers": ["A", "B", "C"],
    "rows": [["1","2","3"],["4","5","6"],["7","8","9"],["10","11","12"],["13","14","15"],["16","17","18"],["19","20","21"],["22","23","24"],["25","26","27"],["28","29","30"]]
  }' \
  -o test_small.xlsx

time curl -X POST ...  # Measure time
```

#### Medium (500 rows)
```bash
# Similar but with 500 rows
```

#### Large (1000 rows)
```bash
# Similar but with 1000 rows
```

---

## 9. Troubleshooting

### Port already in use
```bash
# Find process using port 3000
lsof -i :3000

# Kill process
kill -9 <PID>

# Or use different port
SERVER_PORT=3001 cargo run
```

### JWT Secret not set
```bash
# Set default in .env or environment
export JWT_SECRET="your-secret-key"
cargo run
```

### Dependencies not found
```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

### Module not found errors
```bash
# Check module declarations in src/lib.rs
# Ensure all modules are properly declared:
# pub mod domain { ... }
# pub mod application { ... }
# pub mod infrastructure { ... }
# pub mod presentation { ... }
```

---

## 10. Development Workflow

### Typical Session
```bash
# 1. Start server in one terminal
cargo run

# 2. Get token in another terminal
TOKEN=$(curl -s http://localhost:3000/api/auth/token | jq -r '.token')

# 3. Test export
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '...' \
  -o test.xlsx

# 4. Check file
file test.xlsx

# 5. Make changes to code
# (modify Rust files)

# 6. Reload (Ctrl+C and run again)
cargo run
```

---

## 11. Next Steps (Phase 2)

After MVP is solid:
- Add async export processing
- Implement export templates
- Add database persistence
- Support for export history
- User audit logging
- Advanced Excel formatting (formulas, conditional)
- Multiple worksheets

---

## Quick Reference

| Task | Command |
|------|---------|
| Start server | `cargo run` |
| Run tests | `cargo test` |
| Build release | `cargo build --release` |
| Get token | `curl http://localhost:3000/api/auth/token` |
| Docker build | `docker build -t export-service .` |
| Docker run | `docker run -p 3000:3000 export-service` |
| Format code | `cargo fmt` |
| Lint code | `cargo clippy` |

