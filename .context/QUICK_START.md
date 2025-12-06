# Export Service (Rust) - Quick Start Guide

## Overview
Multi-format export service (Excel, CSV, PDF) using **Clean Architecture** with Rust.

### Technology Stack
| Component | Library | Trust Score | Features |
|-----------|---------|-------------|----------|
| Excel | `rust_xlsxwriter` | 9.1 | High-performance, formatting |
| CSV | `csv` (BurntSushi) | 9.1 | Fast, Serde integration |
| PDF | `printpdf` | 9.3 | Comprehensive, flexible |
| Web | `axum` | Modern | High-performance HTTP |
| Async | `tokio` | Standard | Industry standard runtime |

---

## Project Setup

```bash
# 1. Create project
cargo new export-service
cd export-service

# 2. Update Cargo.toml with dependencies from provided file

# 3. Create directory structure
mkdir -p src/{domain/{models,errors,validators},application/{ports,use_cases,dto},infrastructure/{exporters,repositories},presentation/{handlers,dto}}

# 4. Copy files from provided implementation

# 5. Run tests
cargo test

# 6. Start server
cargo run
```

---

## Architecture Layers (Clean Architecture)

```
┌─────────────────────────────────────────────────────┐
│  PRESENTATION LAYER                                 │
│  - HTTP handlers (Axum)                             │
│  - DTOs (Data Transfer Objects)                     │
└────────────────┬────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────┐
│  APPLICATION LAYER                                  │
│  - Use Cases (Orchestration)                        │
│  - Ports (Abstract traits)                          │
│  - DTOs (Domain Contracts)                          │
└────────────────┬────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────┐
│  DOMAIN LAYER                                       │
│  - Models (ExportData, ExportFormat)                │
│  - Value Objects                                    │
│  - Business Rules (Validators)                      │
│  - NO external dependencies                         │
└─────────────────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────┐
│  INFRASTRUCTURE LAYER                               │
│  - Excel Exporter (rust_xlsxwriter)                 │
│  - CSV Exporter (csv)                               │
│  - PDF Exporter (printpdf)                          │
│  - File Repository (persistence)                    │
└─────────────────────────────────────────────────────┘
```

---

## SOLID Principles Applied

### 1️⃣ Single Responsibility
- `ExcelExporter` → Format data to Excel only
- `CsvExporter` → Format data to CSV only  
- `ExportUseCase` → Orchestrate flow only
- `FileRepository` → Save/load exports only

### 2️⃣ Open/Closed
Add new export format WITHOUT modifying existing code:
```rust
pub struct JsonExporter; // New format

impl ExportService for JsonExporter {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>> {
        // Implementation
    }
}
```

### 3️⃣ Liskov Substitution
All exporters implement `ExportService` identically:
```rust
pub trait ExportService: Send + Sync {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>>;
}
```

### 4️⃣ Interface Segregation
- `ExportService` → Export-specific operations only
- `ExportRepository` → Persistence-specific operations only
- `ExportValidator` → Validation-specific operations only

### 5️⃣ Dependency Inversion
- `ExportUseCase` depends on **traits**, not concrete types
- Infrastructure implements traits
- Wiring at application startup (DI)

---

## Key Design Patterns

### Strategy Pattern (Format Selection)
```rust
let service: Arc<dyn ExportService> = match format {
    ExportFormat::Excel => Arc::new(ExcelExporter),
    ExportFormat::Csv => Arc::new(CsvExporter),
    ExportFormat::Pdf => Arc::new(PdfExporter),
};
```

### Factory Pattern (Exporter Creation)
```rust
pub struct ExporterFactory;

impl ExporterFactory {
    pub fn create(format: ExportFormat) -> Arc<dyn ExportService> {
        // Match format and return appropriate exporter
    }
}
```

### Repository Pattern (Data Abstraction)
```rust
pub trait ExportRepository: Send + Sync {
    fn save(&self, data: &ExportData, bytes: &[u8]) -> Result<String>;
    fn get(&self, id: &str) -> Result<Vec<u8>>;
}
```

---

## Example API Usage

### Request
```bash
curl -X POST http://localhost:3000/api/export \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Sales Report",
    "format": "excel",
    "headers": ["Product", "Q1", "Q2", "Q3"],
    "rows": [
      ["Product A", "100", "150", "200"],
      ["Product B", "120", "140", "180"],
      ["Product C", "90", "110", "130"]
    ]
  }'
```

### Response
```json
{
  "success": true,
  "export_id": "550e8400-e29b-41d4-a716-446655440000",
  "error": null
}
```

---

## Testing Strategy

### Unit Tests (No I/O)
```bash
# Test individual exporters
cargo test excel_exporter
cargo test csv_exporter
cargo test pdf_exporter

# Test validators
cargo test validator
```

### Integration Tests
```bash
# Test full pipeline
cargo test use_case

# Test HTTP handlers
cargo test handle_export
```

### Run All Tests
```bash
cargo test --all
```

---

## Code Quality Practices

✅ **Small Functions**: Max 20-30 lines per function  
✅ **Meaningful Names**: `export_data`, not `data` or `d`  
✅ **No Deep Nesting**: Max 2-3 levels  
✅ **No Side Effects**: Pure functions where possible  
✅ **Error Handling**: Explicit error types, no unwrap()  
✅ **Minimal Dependencies**: Only necessary libraries  

---

## Error Handling

### Domain Errors
```rust
pub enum DomainError {
    InvalidFormat(String),
    EmptyData(String),
    MismatchedHeaders(String),
}
```

### Library Errors (Wrapped)
```rust
pub enum ExportError {
    #[error("Excel error: {0}")]
    ExcelError(#[from] XlsxError),
    
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
}
```

---

## Performance Optimization Tips

1. **Buffered I/O**: Use `Vec<u8>` buffer for large files
2. **Chunked Processing**: Process rows in batches
3. **Async Operations**: Use tokio::spawn for parallel exports
4. **Streaming**: For huge datasets, implement streaming
5. **Format Caching**: Reuse Format objects

```rust
// Example: Reusable formats
let header_format = Format::new().set_bold();
for col in 0..headers.len() {
    worksheet.write_with_format(0, col as u16, &header, &header_format)?;
}
```

---

## Deployment

### Docker
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/export-service /usr/bin/
EXPOSE 3000
CMD ["export-service"]
```

### Build & Run
```bash
docker build -t export-service .
docker run -p 3000:3000 export-service
```

---

## File Structure

```
export-service/
├── Cargo.toml
├── src/
│   ├── main.rs                          # Entry point
│   ├── lib.rs                           # Library exports
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── models.rs                    # ExportData, ExportFormat
│   │   ├── errors.rs                    # DomainError
│   │   └── validators.rs                # ExportValidator
│   ├── application/
│   │   ├── mod.rs
│   │   ├── ports.rs                     # Traits (ExportService, Repository)
│   │   ├── use_cases.rs                 # ExportUseCase
│   │   └── dto.rs                       # ExportRequest, ExportResponse
│   ├── infrastructure/
│   │   ├── exporters/
│   │   │   ├── mod.rs
│   │   │   ├── excel.rs                 # ExcelExporter
│   │   │   ├── csv.rs                   # CsvExporter
│   │   │   └── pdf.rs                   # PdfExporter
│   │   └── repositories/
│   │       ├── mod.rs
│   │       └── file.rs                  # FileRepository
│   └── presentation/
│       ├── handlers.rs                  # HTTP handlers
│       └── dto.rs                       # HTTP DTOs
└── tests/
    └── integration.rs
```

---

## Comparison with Other Approaches

### ✅ This Approach (Clean Architecture)
- Testable (no file I/O in tests)
- Maintainable (clear separation)
- Extensible (add formats easily)
- Decoupled (domain independent)
- Professional code structure

### ❌ Monolithic Approach
- Hard to test
- Mixed concerns
- Difficult to extend
- Tightly coupled

---

## Next Steps

1. **Start Simple**: Implement CSV exporter first
2. **Add Tests**: Write unit tests for CSV
3. **Expand**: Add Excel exporter
4. **Complete**: Add PDF exporter
5. **API**: Expose via HTTP endpoints
6. **Deploy**: Containerize and deploy

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [rust_xlsxwriter docs](https://github.com/jmcnamara/rust_xlsxwriter)
- [csv crate docs](https://docs.rs/csv/latest/csv/)
- [printpdf docs](https://docs.rs/printpdf/latest/printpdf/)
- [Axum web framework](https://github.com/tokio-rs/axum)
- [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
