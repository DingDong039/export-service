# ⚡ Export Service - Quick Phase Guide

**One-page reference for rapid implementation**

---

## 🎯 10 Phases at a Glance

| # | Phase | Time | Files | Key Output |
|---|-------|------|-------|------------|
| 1 | Setup | 5m | 1 | Cargo.toml + folders |
| 2 | Domain | 10m | 4 | Models + Validators |
| 3 | Application | 10m | 4 | Use Cases + Ports |
| 4 | Exporters | 15m | 5 | Excel/CSV/PDF |
| 5 | Auth | 10m | 2 | JWT Handler |
| 6 | Presentation | 15m | 4 | HTTP Handlers |
| 7 | Main | 10m | 2 | Server Setup |
| 8 | Config | 2m | 1 | .env file |
| 9 | Build | 5m | 0 | Compile |
| 10 | Test | 5m | 0 | Verify API |
| **Total** | | **90m** | **23** | **Working Service** |

---

## Phase 1: Setup (5 min)

```bash
cargo new export-service && cd export-service
mkdir -p src/{domain,application,infrastructure/{exporters,auth},presentation}
```

**Update Cargo.toml:**
```toml
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
```

---

## Phase 2: Domain (10 min)

**Files:** models.rs, errors.rs, validators.rs, mod.rs

**Key Components:**
- `ExportData` struct (title, format, headers, rows, options)
- `ExportFormat` enum (Excel, Csv, Pdf)
- `DomainError` enum (8 error types)
- `DefaultExportValidator` (validates data)

**Validation Rules:**
- Headers not empty
- Rows not empty
- Max 10,000 rows
- Column count matches
- Max 1,000 chars per cell

---

## Phase 3: Application (10 min)

**Files:** ports.rs, use_cases.rs, dto.rs, mod.rs

**Key Components:**
- `ExportService` trait (interface for exporters)
- `ExportUseCase` (orchestrates validation + export)
- `ExportRequest` DTO (HTTP → Domain conversion)

**Flow:** Request → Validate → Select Exporter → Export → Return Bytes

---

## Phase 4: Exporters (15 min)

**Files:** excel.rs, csv.rs, pdf.rs, mod.rs, infrastructure/mod.rs

### Excel (rust_xlsxwriter)
- Write headers + data
- Set column width
- Freeze panes
- Return .xlsx bytes

### CSV (csv crate)
- Write headers + rows
- UTF-8 encoding
- Quote escaping
- Return .csv bytes

### PDF (printpdf)
- Title + headers
- Data rows (first 30)
- A4 page
- Return .pdf bytes

---

## Phase 5: Auth (10 min)

**Files:** jwt_handler.rs, auth/mod.rs

**JwtHandler:**
- `generate_token()` → JWT string (1 hour expiry)
- `validate_token()` → Claims or error

**Claims:** iss, sub, exp, iat

---

## Phase 6: Presentation (15 min)

**Files:** handlers.rs, auth.rs, dto.rs, mod.rs

**Handlers:**
1. `health_check()` → "✅ OK"
2. `get_token()` → JWT token
3. `handle_export()` → Binary file

**Middleware:**
- `auth_middleware()` → Validate Bearer token

---

## Phase 7: Main (10 min)

**Files:** main.rs, lib.rs

**main.rs:**
```rust
1. Initialize JWT handler
2. Initialize validator
3. Initialize exporters (3)
4. Initialize use case
5. Create app state
6. Build router:
   - /health
   - /api/auth/token
   - /api/export (with auth)
7. Start server on :3000
```

---

## Phase 8: Config (2 min)

**File:** .env

```bash
JWT_SECRET=dev-secret-key
JWT_EXPIRATION_SECONDS=3600
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
LOG_LEVEL=info
```

---

## Phase 9: Build (5 min)

```bash
cargo build
cargo build --release
```

**Fix common errors:**
- Module not found → Check mod.rs
- Trait not implemented → Implement ExportService
- Import not found → Add use statements

---

## Phase 10: Test (5 min)

```bash
# Start server
cargo run

# Get token
TOKEN=$(curl -s http://localhost:3000/api/auth/token | jq -r .token)

# Export Excel
curl -X POST http://localhost:3000/api/export \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Report",
    "format": "excel",
    "headers": ["Name", "Value"],
    "rows": [["John", "100"]]
  }' -o test.xlsx

# Verify
file test.xlsx
```

---

## 📁 File Tree

```
export-service/
├── Cargo.toml
├── .env
└── src/
    ├── main.rs
    ├── lib.rs
    ├── domain/
    │   ├── mod.rs
    │   ├── models.rs
    │   ├── errors.rs
    │   └── validators.rs
    ├── application/
    │   ├── mod.rs
    │   ├── ports.rs
    │   ├── use_cases.rs
    │   └── dto.rs
    ├── infrastructure/
    │   ├── mod.rs
    │   ├── exporters/
    │   │   ├── mod.rs
    │   │   ├── excel.rs
    │   │   ├── csv.rs
    │   │   └── pdf.rs
    │   └── auth/
    │       ├── mod.rs
    │       └── jwt_handler.rs
    └── presentation/
        ├── mod.rs
        ├── handlers.rs
        ├── auth.rs
        └── dto.rs
```

**Total: 23 files**

---

## 🚀 API Reference

### GET /health
Returns: `"✅ OK"`

### GET /api/auth/token
Returns:
```json
{
  "token": "eyJhbG...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### POST /api/export
Headers:
```
Authorization: Bearer <token>
Content-Type: application/json
```

Body:
```json
{
  "title": "Report",
  "format": "excel|csv|pdf",
  "headers": ["Col1", "Col2"],
  "rows": [["val1", "val2"]],
  "options": {
    "freeze_headers": true,
    "auto_fit_columns": true,
    "header_bold": true
  }
}
```

Returns: Binary file (Excel/CSV/PDF)

---

## 🔍 Quick Troubleshooting

**Server won't start:**
- Check port 3000 free
- Verify JWT_SECRET in .env

**Compilation fails:**
- Run `cargo clean && cargo build`
- Check all mod.rs declarations
- Verify imports

**Export fails:**
- Check token is valid
- Verify data format
- Check row/column counts

**401 Unauthorized:**
- Token missing or invalid
- Get new token

**400 Bad Request:**
- Data validation failed
- Check error message

---

## 💡 Quick Tips

1. **Start Simple:** Test with 1-2 rows first
2. **Token:** Save token in variable for testing
3. **Format:** Use `file` command to verify output
4. **Debug:** Check server logs for errors
5. **Test Order:** Health → Token → Export

---

## 📊 Architecture Layers

```
┌─────────────────────────┐
│    Presentation         │  HTTP, Auth
├─────────────────────────┤
│    Application          │  Use Cases
├─────────────────────────┤
│    Domain               │  Models, Rules
├─────────────────────────┤
│    Infrastructure       │  Exporters, JWT
└─────────────────────────┘
```

---

## ⏱️ Time Breakdown

- **Critical Path:** Phases 1-7 (60 min)
- **Configuration:** Phase 8 (2 min)
- **Verification:** Phases 9-10 (10 min)
- **Buffer:** 18 min for debugging

---

## ✅ Success Criteria

After completion:
- [ ] Server runs on :3000
- [ ] Token endpoint works
- [ ] Excel export works
- [ ] CSV export works
- [ ] PDF export works
- [ ] Files open correctly
- [ ] Auth protects /api/export
- [ ] Errors return proper codes

---

## 🎯 Next Steps After Completion

1. Add unit tests
2. Create Dockerfile
3. Write documentation
4. Integrate with web client
5. Deploy to production

---

## 📚 Key Concepts

**Clean Architecture:**
- Domain = Business logic (no dependencies)
- Application = Use cases (orchestration)
- Infrastructure = External libs (Excel, JWT)
- Presentation = HTTP layer (Axum)

**Dependency Direction:**
```
Presentation → Application → Domain ← Infrastructure
```

**Dependency Injection:**
All dependencies created in main.rs and passed via Arc<T>

**Error Handling:**
Domain errors → HTTP status codes (400, 401, 500)

---

## 🔗 Reference Documents

- **Full Guide:** PHASE_IMPLEMENTATION_PLAN.md
- **Checklist:** PHASE_CHECKLIST.md
- **Spec:** MVP_SPECIFICATION.md
- **Setup:** MVP_SETUP_GUIDE.md

---

## 💪 You Got This!

Follow the phases, check items off the checklist, and you'll have a working service in 90 minutes!

**Remember:**
1. One phase at a time
2. Compile often
3. Test as you go
4. Ask for help if stuck

**Good luck! 🚀**
