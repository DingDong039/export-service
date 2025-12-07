# Entity-Relationship Diagram

## System Architecture Overview

This project uses **Clean Architecture** with clear separation between layers. The ER diagram below shows the relationships between domain entities and how they flow through the system.

---

## Domain Model ER Diagram

```mermaid
erDiagram
    ExportRequest ||--|| ExportData : "converts to"
    ExportData ||--|| ExportFormat : "has"
    ExportData ||--o| ExportOptions : "optionally has"
    ExportData ||--o{ ColumnMetadata : "optionally has many"

    ExportRequest {
        string title
        string format
        array headers
        array rows
        ExportOptions options
        array column_metadata
    }

    ExportData {
        string title
        ExportFormat format
        array headers
        array rows
        ExportOptions options
        array column_metadata
    }

    ExportFormat {
        enum type "Excel|Csv|Pdf"
        string extension
        string mime_type
    }

    ExportOptions {
        bool freeze_headers
        bool auto_fit_columns
        bool header_bold
        string header_background
        bool include_header_row
        string delimiter
    }

    ColumnMetadata {
        ColumnType column_type
        float width_hint
    }

    ColumnType {
        enum type "Text|Number|Currency|Percentage|Date"
        bool is_right_aligned
    }
```

---

## Data Flow Diagram

```mermaid
flowchart TB
    Client[Client/Postman]

    subgraph Presentation["Presentation Layer"]
        Handler[HTTP Handler]
        AuthMW[Auth Middleware]
        DTO[Request DTO]
    end

    subgraph Application["Application Layer"]
        UseCase[Export Use Case]
        Validator[Validator Port]
        ServicePort[Export Service Port]
    end

    subgraph Domain["Domain Layer"]
        ExportData[Export Data Model]
        ExportFormat[Export Format Enum]
        ExportOptions[Export Options]
        ColumnMeta[Column Metadata]
        DomainError[Domain Errors]
    end

    subgraph Infrastructure["Infrastructure Layer"]
        ExcelExporter[Excel Exporter]
        CsvExporter[CSV Exporter]
        PdfExporter[PDF Exporter]
        JwtHandler[JWT Handler]
    end

    Client -->|POST /api/export| AuthMW
    AuthMW -->|Validate Token| JwtHandler
    AuthMW -->|Authorized| Handler
    Handler -->|Parse JSON| DTO
    DTO -->|Convert to Domain| ExportData

    Handler -->|Execute| UseCase
    UseCase -->|Validate| Validator
    Validator -->|Check Rules| ExportData

    UseCase -->|Select Service| ServicePort
    ServicePort -.->|Excel Format| ExcelExporter
    ServicePort -.->|CSV Format| CsvExporter
    ServicePort -.->|PDF Format| PdfExporter

    ExcelExporter -->|Generate File| Client
    CsvExporter -->|Generate File| Client
    PdfExporter -->|Generate File| Client

    ExportData -->|Uses| ExportFormat
    ExportData -->|Uses| ExportOptions
    ExportData -->|Uses| ColumnMeta
    Validator -->|Throws| DomainError
```

---

## Clean Architecture Layer Relationships

```mermaid
graph TB
    subgraph External["External World"]
        HTTP[HTTP Requests]
        FileSystem[File System]
    end

    subgraph Presentation["Presentation Layer (src/presentation/)"]
        Handlers[handlers.rs]
        AuthMiddleware[auth.rs]
        PresentationDTO[dto.rs]
    end

    subgraph Application["Application Layer (src/application/)"]
        UseCases[use_cases.rs]
        Ports[ports.rs - Traits]
        AppDTO[dto.rs]
    end

    subgraph Domain["Domain Layer (src/domain/)"]
        Models[models.rs]
        Validators[validators.rs]
        Errors[errors.rs]
    end

    subgraph Infrastructure["Infrastructure Layer (src/infrastructure/)"]
        Exporters[exporters/]
        Auth[auth/]
    end

    HTTP -->|Request| Handlers
    Handlers -->|Use| UseCases
    UseCases -->|Define| Ports
    Ports -->|Implemented by| Exporters
    Ports -->|Implemented by| Auth

    UseCases -->|Use| Models
    UseCases -->|Use| Validators
    Validators -->|Validate| Models
    Validators -->|Throw| Errors

    Exporters -->|Write to| FileSystem

    style Domain fill:#e1f5e1
    style Application fill:#e3f2fd
    style Infrastructure fill:#fff3e0
    style Presentation fill:#f3e5f5
```

---

## Detailed Entity Descriptions

### 1. ExportRequest (Presentation DTO)
**Location:** `src/presentation/dto.rs` → `src/application/dto.rs`

HTTP request from client that gets converted to domain model.

**Attributes:**
- `title`: Export file name
- `format`: Export format as string ("excel", "csv", "pdf")
- `headers`: Column headers array
- `rows`: Data rows (2D array)
- `options`: Optional formatting options
- `column_metadata`: Optional column type/width information

**Validations:**
- Format must be valid ("excel", "csv", "pdf")
- Converts to `ExportData` domain model

---

### 2. ExportData (Domain Model)
**Location:** `src/domain/models.rs`

Core business entity representing export data.

**Attributes:**
- `title`: String - export title
- `format`: ExportFormat enum
- `headers`: Vec<String> - column headers
- `rows`: Vec<Vec<String>> - data rows
- `options`: Option<ExportOptions>
- `column_metadata`: Option<Vec<ColumnMetadata>>

**Business Rules:**
- Maximum 10,000 rows
- Maximum 1,000 characters per header
- Maximum 1,000 characters per cell
- All rows must have same column count as headers

---

### 3. ExportFormat (Domain Enum)
**Location:** `src/domain/models.rs`

**Values:**
- `Excel` → extension: "xlsx", mime: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
- `Csv` → extension: "csv", mime: "text/csv"
- `Pdf` → extension: "pdf", mime: "application/pdf"

**Methods:**
- `extension()`: Returns file extension
- `mime_type()`: Returns MIME type for HTTP response

---

### 4. ExportOptions (Domain Model)
**Location:** `src/domain/models.rs`

Optional formatting configuration for exports.

**Attributes:**
- `freeze_headers`: Option<bool> - freeze header row (Excel only)
- `auto_fit_columns`: Option<bool> - auto-fit column widths (Excel only)
- `header_bold`: Option<bool> - make headers bold
- `header_background`: Option<String> - header background color (hex)
- `include_header_row`: Option<bool> - include headers in export
- `delimiter`: Option<String> - column delimiter (CSV only)

---

### 5. ColumnMetadata (Domain Model)
**Location:** `src/domain/models.rs`

Metadata for individual columns to control formatting and alignment.

**Attributes:**
- `column_type`: ColumnType enum
- `width_hint`: Option<f32> - suggested column width

**Factory Methods:**
- `text()`, `number()`, `currency()`, `percentage()`, `date()`
- `with_width(width)` - builder method for width

---

### 6. ColumnType (Domain Enum)
**Location:** `src/domain/models.rs`

**Values:**
- `Text` - left-aligned text (default)
- `Number` - right-aligned numbers
- `Currency` - right-aligned currency format
- `Percentage` - right-aligned percentage
- `Date` - date format

**Methods:**
- `is_right_aligned()`: Returns true for Number, Currency, Percentage

---

## Dependency Injection Flow

```mermaid
sequenceDiagram
    participant Main
    participant Validator
    participant Exporters
    participant UseCase
    participant AppState
    participant Router

    Main->>Validator: Create BasicValidator
    Main->>Exporters: Create ExcelExporter, CsvExporter, PdfExporter
    Main->>UseCase: Inject Arc<dyn Validator>
    Main->>UseCase: Inject Arc<dyn ExportService> for each format
    Main->>AppState: Create with UseCase + JwtHandler
    Main->>Router: Inject AppState
    Router->>Router: Bind handlers with state
```

---

## Validation Rules Diagram

```mermaid
flowchart TD
    Start[ExportData Input]

    CheckRows{rows.len() > 0?}
    CheckMaxRows{rows.len() <= 10000?}
    CheckHeaders{headers.len() > 0?}
    CheckHeaderLen{header.len() <= 1000?}
    CheckRowCols{row.len() == headers.len()?}
    CheckCellLen{cell.len() <= 1000?}

    Valid[Validation Passed]
    Error[Throw DomainError]

    Start --> CheckRows
    CheckRows -->|No| Error
    CheckRows -->|Yes| CheckMaxRows
    CheckMaxRows -->|No| Error
    CheckMaxRows -->|Yes| CheckHeaders
    CheckHeaders -->|No| Error
    CheckHeaders -->|Yes| CheckHeaderLen
    CheckHeaderLen -->|No| Error
    CheckHeaderLen -->|Yes| CheckRowCols
    CheckRowCols -->|No| Error
    CheckRowCols -->|Yes| CheckCellLen
    CheckCellLen -->|No| Error
    CheckCellLen -->|Yes| Valid
```

---

## Export Service Implementations

Each format has its own implementation of the `ExportService` trait:

### Interface (Port)
```rust
pub trait ExportService: Send + Sync {
    fn export(&self, data: &ExportData) -> Result<Vec<u8>, DomainError>;
}
```

### Implementations

1. **ExcelExporter** (`src/infrastructure/exporters/excel.rs`)
   - Uses `rust_xlsxwriter` crate
   - Supports: freeze_headers, auto_fit_columns, header styling

2. **CsvExporter** (`src/infrastructure/exporters/csv.rs`)
   - Uses `csv` crate
   - Supports: custom delimiter

3. **PdfExporter** (`src/infrastructure/exporters/pdf.rs`)
   - Uses `printpdf` crate
   - Supports: header styling, column alignment, Thai fonts

---

## Authentication Flow

```mermaid
sequenceDiagram
    participant Client
    participant GetToken[GET /api/auth/token]
    participant JwtHandler
    participant Export[POST /api/export]
    participant AuthMW[Auth Middleware]

    Client->>GetToken: Request token
    GetToken->>JwtHandler: generate_token()
    JwtHandler-->>GetToken: JWT token (expires in 3600s)
    GetToken-->>Client: {token, expires_in, token_type}

    Client->>Export: POST with Bearer token
    Export->>AuthMW: Validate request
    AuthMW->>JwtHandler: validate_token(token)

    alt Valid Token
        JwtHandler-->>AuthMW: Claims
        AuthMW-->>Export: Proceed
    else Invalid/Expired Token
        JwtHandler-->>AuthMW: Error
        AuthMW-->>Client: 401 Unauthorized
    end
```

---

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| **HTTP Server** | Axum, Tokio |
| **Authentication** | jsonwebtoken |
| **Excel Export** | rust_xlsxwriter |
| **CSV Export** | csv |
| **PDF Export** | printpdf |
| **Serialization** | serde, serde_json |
| **Architecture** | Clean Architecture, DDD |

---

## Database Consideration

**Note:** This application is currently **stateless** and does not use a database. All data is provided via API requests.

### Future Database Schema (if needed)

```mermaid
erDiagram
    Users ||--o{ ExportJobs : creates
    ExportJobs ||--|| ExportFormats : has
    ExportJobs ||--o{ ExportLogs : generates

    Users {
        uuid id PK
        string username
        string email
        timestamp created_at
    }

    ExportJobs {
        uuid id PK
        uuid user_id FK
        string title
        enum format
        jsonb options
        jsonb column_metadata
        enum status
        timestamp created_at
        timestamp completed_at
    }

    ExportFormats {
        int id PK
        string name
        string extension
        string mime_type
    }

    ExportLogs {
        uuid id PK
        uuid job_id FK
        string message
        enum level
        timestamp created_at
    }
```

---

## References

- Clean Architecture: [docs/CLAUDE.md](../CLAUDE.md)
- API Specification: [docs/API_SPEC.md](./API_SPEC.md)
- Source Code: [src/](../src/)
