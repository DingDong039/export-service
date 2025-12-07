# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A high-performance REST API service built with Rust and Axum for exporting data to multiple formats (Excel, CSV, PDF) with JWT authentication.

## Development Commands

### Building and Running
- Build the project: `cargo build`
- Run the server: `cargo run` (starts on http://127.0.0.1:3000)
- Run in release mode: `cargo build --release && cargo run --release`

### Testing
- Run all tests: `cargo test`
- Run specific test: `cargo test <test_name>`
- Run with output: `cargo test -- --nocapture`

### Code Quality

- Lint: `cargo clippy`
- Format check: `cargo fmt --check`
- Format fix: `cargo fmt`

## Architecture

This project follows **Clean Architecture** (domain-driven design) with strict separation of concerns:

### Layer Structure (Dependency Rule)

```
Presentation → Application → Domain ← Infrastructure
```

**Critical**: Dependencies only flow inward. Domain has zero dependencies on outer layers.

### Domain Layer (`src/domain/`)
- **Pure business logic** - no framework dependencies
- `models.rs`: Core entities (`ExportData`, `ExportFormat`, `ExportOptions`)
- `validators.rs`: Business rule validation (trait `ExportValidator`)
- `errors.rs`: Domain-specific errors (`DomainError`)
- Validation limits: max 10,000 rows, max 1000 chars per cell/header

### Application Layer (`src/application/`)
- **Use cases** - orchestrates domain logic
- `ports.rs`: Interfaces/traits (e.g., `ExportService` trait)
- `use_cases.rs`: `ExportUseCase` - main export workflow (validate → select service → export)
- `dto.rs`: Data transfer objects for application boundary
- Uses dependency injection via Arc<dyn Trait>

### Infrastructure Layer (`src/infrastructure/`)
- **External implementations** of application ports
- `exporters/`: Format-specific implementations (`ExcelExporter`, `CsvExporter`, `PdfExporter`)
  - Each implements the `ExportService` trait
  - Dependencies: `rust_xlsxwriter`, `csv`, `printpdf`
- `auth/`: JWT handling (`JwtHandler`)
  - Token generation and validation
  - Default expiration: 3600 seconds (configurable)

### Presentation Layer (`src/presentation/`)
- **HTTP interface** - Axum handlers and middleware
- `handlers.rs`: HTTP request handlers (`handle_export`, `health_check`, `get_token`)
- `auth.rs`: Authentication middleware (`auth_middleware`)
- `dto.rs`: HTTP request/response DTOs
- Converts HTTP requests to domain models and vice versa

### Dependency Injection Pattern

The `main.rs` wires everything together:
1. Creates concrete implementations (validators, exporters, JWT handler)
2. Wraps them in `Arc<T>` for thread-safe sharing
3. Injects into `ExportUseCase`
4. Builds `AppState` with use case and JWT handler
5. Passes state to Axum router

## API Endpoints

- `GET /health` - Health check (no auth required)
- `GET /api/auth/token` - Get JWT token (no auth required)
- `POST /api/export` - Export data (requires Bearer token in Authorization header)

## Configuration

Environment variables:
- `JWT_SECRET`: JWT signing key (default: "dev-secret-key")
- `JWT_EXPIRATION_SECONDS`: Token TTL (default: 3600)

## Adding New Export Formats

1. Create new exporter in `src/infrastructure/exporters/` implementing `ExportService` trait
2. Add format variant to `ExportFormat` enum in `src/domain/models.rs`
3. Update `ExportFormat::extension()` and `ExportFormat::mime_type()` methods
4. Register exporter in `main.rs` dependency injection setup
5. Add match arm in `ExportUseCase::execute()` for new format

## Key Design Patterns

- **Trait-based abstractions**: All services defined as traits for testability
- **Arc for shared state**: Thread-safe reference counting for concurrent access
- **Middleware composition**: Axum's layer system for auth and CORS
- **Error propagation**: Using `?` operator with custom domain errors
- **Validation-first**: All data validated before processing

## Code Style

- **Imports**: Group std → external crates → crate:: modules, alphabetically within groups
- **Naming**: snake_case for functions/variables, PascalCase for types/structs/enums
- **Error Handling**: Use `Result<T, E>` with `DomainError`, avoid `unwrap()` in production code
- **Traits**: Define in `ports.rs`, implement in infrastructure layer with `Send + Sync` bounds
- **Formatting**: Default rustfmt (4 spaces, 100 char line limit)

## Code Conventions

- Keep domain models free of serialization concerns (use DTOs)
- Use `Arc<dyn Trait>` for dependency injection
- Keep handlers thin - delegate to use cases
