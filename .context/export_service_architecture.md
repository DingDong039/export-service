# Export Service Architecture - Rust (Clean Architecture)

## Technology Stack Recommendations

### Core Libraries
- **Excel**: `rust_xlsxwriter` (Trust Score: 9.1) - High performance, 388 code snippets
- **CSV**: `csv` (BurntSushi) (Trust Score: 9.1) - Fast & flexible with Serde support
- **PDF**: `printpdf` (Trust Score: 9.3) - Comprehensive PDF creation, 202 code snippets

### Supporting Libraries
- `serde` & `serde_json` - Serialization/Deserialization
- `tokio` - Async runtime
- `thiserror` - Error handling
- `chrono` - Date/Time handling
- `uuid` - Unique identification

---

## Architecture Layers

### 1. Domain Layer (Core Business Logic)
**Purpose**: Entities, Value Objects, Use Cases - NO external dependencies

```rust
// domain/models/export_model.rs
pub struct ExportData {
    id: String,
    format: ExportFormat,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    created_at: DateTime<Utc>,
}

pub enum ExportFormat {
    Excel,
    CSV,
    PDF,
}

// domain/errors/mod.rs
#[derive(Debug)]
pub enum DomainError {
    InvalidFormat,
    EmptyData,
    InvalidHeaderCount,
}
```

### 2. Application Layer (Use Cases)
**Purpose**: Orchestrate domain logic - Depends on Domain only, abstracted from infrastructure

```rust
// application/use_cases/export_use_case.rs
pub trait ExportRepository {
    async fn save(&self, export_data: ExportData) -> Result<(), Error>;
}

pub trait ExportService {
    async fn export(&self, data: ExportData) -> Result<Vec<u8>, Error>;
}

pub struct ExportUseCase<R, S> {
    repository: Arc<R>,
    service: Arc<S>,
}

impl<R, S> ExportUseCase<R, S>
where
    R: ExportRepository,
    S: ExportService,
{
    pub async fn execute(&self, data: ExportData) -> Result<Vec<u8>, Error> {
        // Validate
        // Call service
        // Save to repository
        // Return result
    }
}
```

### 3. Infrastructure Layer (Implementations)
**Purpose**: Concrete implementations of traits - Depends on Application

```rust
// infrastructure/export_services/
//   ├── excel_export.rs
//   ├── csv_export.rs
//   ├── pdf_export.rs
//   └── mod.rs
```

### 4. Presentation Layer (API/Controllers)
**Purpose**: HTTP handlers - Depends on Application, translates requests

```rust
// presentation/api/
//   ├── handlers/export_handler.rs
//   ├── dto/export_dto.rs
//   └── mod.rs
```

---

## Separation of Concerns

### Excel Export Service
- **Single Responsibility**: Format data to Excel only
- **Interface**: `ExportService` trait
- **Dependencies**: `rust_xlsxwriter`, Format trait

### CSV Export Service
- **Single Responsibility**: Format data to CSV only
- **Interface**: `ExportService` trait
- **Dependencies**: `csv` crate

### PDF Export Service
- **Single Responsibility**: Format data to PDF only
- **Interface**: `ExportService` trait
- **Dependencies**: `printpdf` crate

### Data Transformation Layer
- Convert application DTOs → format-specific structures
- Validate data before export
- Handle encoding/encoding issues

---

## Design Patterns Applied

### 1. Strategy Pattern
Different export formats implement same `ExportService` trait

```rust
pub trait ExportService: Send + Sync {
    async fn export(&self, data: &ExportData) -> Result<Vec<u8>, ExportError>;
}

pub struct ExcelExporter;
pub struct CsvExporter;
pub struct PdfExporter;

// Each implements ExportService
```

### 2. Factory Pattern
Create appropriate exporter based on format

```rust
pub struct ExporterFactory;

impl ExporterFactory {
    pub fn create(format: ExportFormat) -> Arc<dyn ExportService> {
        match format {
            ExportFormat::Excel => Arc::new(ExcelExporter),
            ExportFormat::CSV => Arc::new(CsvExporter),
            ExportFormat::PDF => Arc::new(PdfExporter),
        }
    }
}
```

### 3. Repository Pattern
Abstract data persistence

```rust
pub trait ExportRepository: Send + Sync {
    async fn save(&self, export: ExportData) -> Result<String, RepositoryError>;
    async fn get(&self, id: &str) -> Result<ExportData, RepositoryError>;
}
```

---

## SOLID Principles Compliance

### Single Responsibility
- Each service handles ONE format
- ExportUseCase orchestrates flow only
- Repositories handle storage only

### Open/Closed
- Add new export format by implementing `ExportService`
- No modification to existing code
- Factory pattern enables extension

### Liskov Substitution
- All exporters implement `ExportService` identically
- Can substitute any implementation
- Same contract, different behaviors

### Interface Segregation
- `ExportService` - Format conversion only
- `ExportRepository` - Persistence only
- `ExportValidator` - Validation only

### Dependency Inversion
- UseCase depends on traits, not concrete types
- Infrastructure implements traits
- Wiring at application startup (Dependency Injection)

---

## Testability Features

### Unit Tests
- Test each exporter in isolation
- Mock `ExportRepository` and `ExportService`
- No file I/O needed

### Integration Tests
- Test `ExportUseCase` with real implementations
- Verify end-to-end flow

### Mock Example
```rust
pub struct MockExportRepository;

impl ExportRepository for MockExportRepository {
    async fn save(&self, _: ExportData) -> Result<(), Error> {
        Ok(())
    }
}
```

---

## Error Handling

```rust
// application/errors/mod.rs
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("Export format not supported: {0}")]
    UnsupportedFormat(String),
    
    #[error("Data validation failed: {0}")]
    ValidationError(String),
    
    #[error("Export generation failed: {0}")]
    ExportFailed(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}
```

---

## Code Organization

```
src/
├── domain/
│   ├── models/
│   │   └── export_model.rs
│   ├── errors/
│   │   └── mod.rs
│   └── mod.rs
├── application/
│   ├── use_cases/
│   │   └── export_use_case.rs
│   ├── dto/
│   │   └── export_dto.rs
│   ├── traits/
│   │   ├── export_service.rs
│   │   └── export_repository.rs
│   └── mod.rs
├── infrastructure/
│   ├── exporters/
│   │   ├── excel_exporter.rs
│   │   ├── csv_exporter.rs
│   │   ├── pdf_exporter.rs
│   │   └── mod.rs
│   ├── repositories/
│   │   ├── file_repository.rs
│   │   └── mod.rs
│   └── mod.rs
├── presentation/
│   ├── api/
│   │   ├── handlers/
│   │   │   └── export_handler.rs
│   │   └── mod.rs
│   └── mod.rs
└── main.rs
```

---

## Dependency Injection Setup

```rust
// main.rs - Wiring at startup
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let excel_exporter: Arc<dyn ExportService> = Arc::new(ExcelExporter);
    let csv_exporter: Arc<dyn ExportService> = Arc::new(CsvExporter);
    let pdf_exporter: Arc<dyn ExportService> = Arc::new(PdfExporter);
    
    let repository: Arc<dyn ExportRepository> = Arc::new(FileRepository::new());
    
    let use_case = Arc::new(ExportUseCase::new(
        repository.clone(),
        excel_exporter,
        csv_exporter,
        pdf_exporter,
    ));
    
    // Pass to handlers
}
```

---

## Key Benefits

✅ **Testable**: All components are mockable  
✅ **Maintainable**: Clear separation, easy to modify  
✅ **Extensible**: Add new formats without changing existing code  
✅ **Readable**: Small functions, clear names, minimal nesting  
✅ **Flexible**: Swap implementations easily  
✅ **Decoupled**: Domain logic independent of libraries  

---

## Next Steps

1. Create domain models and value objects
2. Define application service interfaces (traits)
3. Implement concrete exporters (Excel, CSV, PDF)
4. Build repository implementations
5. Create UseCase orchestrator
6. Expose via REST API (Actix-web or Axum)
7. Add comprehensive tests
8. Handle error cases and edge cases
