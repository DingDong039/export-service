# Agent Guide for Export Service

## Build/Test/Lint Commands
- **Build**: `cargo build` (dev) or `cargo build --release` (production)
- **Run**: `cargo run` (starts server on http://127.0.0.1:3000)
- **Test**: `cargo test` (run all tests)
- **Test single**: `cargo test <test_name>` (e.g., `cargo test validate`)
- **Lint**: `cargo clippy` (static analysis)
- **Format check**: `cargo fmt --check`
- **Format fix**: `cargo fmt`

## Architecture
- **Clean Architecture**: domain → application → infrastructure → presentation
- **Layers**: domain/ (models, errors, validators), application/ (use_cases, ports, dto), infrastructure/ (exporters, auth), presentation/ (handlers, auth middleware)

## Code Style
- **Imports**: Group std → external crates → crate:: modules, alphabetically within groups
- **Naming**: snake_case for functions/variables, PascalCase for types/structs/enums, SCREAMING_SNAKE_CASE for constants
- **Types**: Use explicit types for public APIs, leverage type inference internally
- **Error Handling**: Use Result<T, E> with custom error types (DomainError), never use unwrap() in production code
- **Traits**: Define traits in ports.rs, implement in infrastructure layer with Send + Sync bounds
- **Comments**: Use `///` for public API docs, `//` for internal notes
- **Formatting**: Default rustfmt (4 spaces, 100 char line limit)
