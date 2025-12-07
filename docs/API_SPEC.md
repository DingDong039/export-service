# API Specification

## Base URL
```
http://127.0.0.1:3000
```

## Authentication
All endpoints except `/health` and `/api/auth/token` require JWT authentication.

**Header:**
```
Authorization: Bearer <token>
```

---

## Endpoints

### 1. Health Check

**Endpoint:** `GET /health`

**Description:** Check API service status

**Authentication:** Not required

**Response:**
```json
{
  "status": "ok"
}
```

**Status Codes:**
- `200 OK`: Service is healthy

---

### 2. Get Authentication Token

**Endpoint:** `GET /api/auth/token`

**Description:** Generate JWT token for API authentication

**Authentication:** Not required

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

**Status Codes:**
- `200 OK`: Token generated successfully
- `500 Internal Server Error`: Token generation failed

---

### 3. Export Data

**Endpoint:** `POST /api/export`

**Description:** Export data to Excel, CSV, or PDF format

**Authentication:** Required (Bearer Token)

**Request Headers:**
```
Content-Type: application/json
Authorization: Bearer <token>
```

**Request Body:**
```json
{
  "title": "รายงานพนักงาน",
  "format": "pdf",
  "headers": ["รหัส", "ชื่อ", "แผนก", "เงินเดือน"],
  "rows": [
    ["EMP001", "สมชาย ใจดี", "IT", "85000"],
    ["EMP002", "สมหญิง รักงาน", "HR", "75000"]
  ],
  "options": {
    "header_bold": true,
    "header_background": "#4472C4",
    "include_header_row": true,
    "freeze_headers": false,
    "auto_fit_columns": true,
    "delimiter": ","
  },
  "column_metadata": [
    {"column_type": "text"},
    {"column_type": "text"},
    {"column_type": "text"},
    {"column_type": "currency"}
  ]
}
```

**Request Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Export file title/name |
| `format` | string | Yes | Export format: `excel`, `csv`, or `pdf` |
| `headers` | array[string] | Yes | Column headers (max 1000 chars each) |
| `rows` | array[array[string]] | Yes | Data rows (max 10,000 rows, max 1000 chars per cell) |
| `options` | object | No | Export formatting options |
| `column_metadata` | array[object] | No | Column type and width hints |

**Options Object:**

| Field | Type | Default | Description | Applicable To |
|-------|------|---------|-------------|---------------|
| `header_bold` | boolean | false | Make headers bold | All formats |
| `header_background` | string | null | Header background color (hex: `#RRGGBB`) | Excel, PDF |
| `include_header_row` | boolean | true | Include header row in export | All formats |
| `freeze_headers` | boolean | false | Freeze header row | Excel only |
| `auto_fit_columns` | boolean | false | Auto-fit column widths | Excel only |
| `delimiter` | string | `,` | Column delimiter character | CSV only |

**Column Metadata Object:**

| Field | Type | Description |
|-------|------|-------------|
| `column_type` | string | Data type: `text`, `number`, `currency`, `percentage`, `date` |
| `width_hint` | number | Optional column width (pixels or percentage) |

**Column Types:**
- `text`: Left-aligned text (default)
- `number`: Right-aligned numbers
- `currency`: Right-aligned currency format
- `percentage`: Right-aligned percentage format
- `date`: Date format

**Response:**
```
Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet  (Excel)
Content-Type: text/csv                                                           (CSV)
Content-Type: application/pdf                                                    (PDF)
Content-Disposition: attachment; filename="<title>.<extension>"

[Binary file data]
```

**Status Codes:**
- `200 OK`: Export successful, file returned
- `400 Bad Request`: Invalid request data or validation failed
- `401 Unauthorized`: Missing or invalid authentication token
- `500 Internal Server Error`: Export processing failed

**Error Response:**
```json
{
  "error": "Validation failed: Too many rows (max: 10000)"
}
```

---

## Validation Rules

### Data Limits
- **Maximum rows:** 10,000
- **Maximum header length:** 1,000 characters
- **Maximum cell length:** 1,000 characters
- **Minimum rows:** 1 (at least one data row required)
- **Minimum headers:** 1 (at least one column required)

### Format Validation
- Headers and rows must have matching column counts
- Format must be one of: `excel`, `csv`, `pdf` (case-insensitive)
- All rows must have the same number of columns as headers

### Authentication
- JWT token expires after 3600 seconds (1 hour) by default
- Token must be included in `Authorization` header as `Bearer <token>`
- Invalid or expired tokens return `401 Unauthorized`

---

## Example Requests

### Get Token (cURL)
```bash
curl -X GET http://127.0.0.1:3000/api/auth/token
```

### Export to PDF (cURL)
```bash
curl -X POST http://127.0.0.1:3000/api/export \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "title": "Employee Report",
    "format": "pdf",
    "headers": ["ID", "Name", "Department", "Salary"],
    "rows": [
      ["EMP001", "John Doe", "IT", "85000"],
      ["EMP002", "Jane Smith", "HR", "75000"]
    ],
    "options": {
      "header_bold": true,
      "header_background": "#4472C4"
    },
    "column_metadata": [
      {"column_type": "text"},
      {"column_type": "text"},
      {"column_type": "text"},
      {"column_type": "currency"}
    ]
  }' \
  --output report.pdf
```

### Export to Excel (cURL)
```bash
curl -X POST http://127.0.0.1:3000/api/export \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "title": "Sales Data",
    "format": "excel",
    "headers": ["Date", "Product", "Quantity", "Revenue"],
    "rows": [
      ["2024-01-01", "Product A", "100", "50000"],
      ["2024-01-02", "Product B", "150", "75000"]
    ],
    "options": {
      "freeze_headers": true,
      "auto_fit_columns": true,
      "header_bold": true
    }
  }' \
  --output sales.xlsx
```

### Export to CSV (cURL)
```bash
curl -X POST http://127.0.0.1:3000/api/export \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{
    "title": "Customer List",
    "format": "csv",
    "headers": ["ID", "Name", "Email"],
    "rows": [
      ["001", "John Doe", "john@example.com"],
      ["002", "Jane Smith", "jane@example.com"]
    ],
    "options": {
      "delimiter": ","
    }
  }' \
  --output customers.csv
```

---

## Postman Collection Example

### Complete Request (PDF with Thai Language)
```json
{
  "title": "รายงานพนักงาน",
  "format": "pdf",
  "headers": ["รหัสพนักงาน", "ชื่อ-นามสกุล", "แผนก", "ตำแหน่ง", "เงินเดือน", "วันที่เริ่มงาน", "อีเมล", "เบอร์โทร", "สถานะ"],
  "rows": [
    ["EMP001", "สมชาย ใจดี", "IT", "Senior Developer", "85000", "2020-01-15", "somchai@company.com", "081-234-5678", "Active"],
    ["EMP002", "สมหญิง รักงาน", "HR", "HR Manager", "75000", "2019-03-20", "somying@company.com", "082-345-6789", "Active"],
    ["EMP003", "วิชัย เก่งมาก", "Finance", "Accountant", "55000", "2021-06-01", "wichai@company.com", "083-456-7890", "Active"],
    ["EMP004", "นภา สวยงาม", "Marketing", "Marketing Lead", "70000", "2018-09-10", "napa@company.com", "084-567-8901", "Active"],
    ["EMP005", "ประยุทธ์ ขยันดี", "IT", "DevOps Engineer", "80000", "2020-11-25", "prayut@company.com", "085-678-9012", "Active"]
  ],
  "options": {
    "header_bold": true,
    "header_background": "#4472C4",
    "include_header_row": true
  },
  "column_metadata": [
    {"column_type": "text"},
    {"column_type": "text"},
    {"column_type": "text"},
    {"column_type": "text"},
    {"column_type": "currency"},
    {"column_type": "date"},
    {"column_type": "text"},
    {"column_type": "text"},
    {"column_type": "text"}
  ]
}
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `JWT_SECRET` | Secret key for JWT signing | `dev-secret-key` |
| `JWT_EXPIRATION_SECONDS` | Token expiration time in seconds | `3600` |

---

## Error Codes

| Status Code | Description |
|-------------|-------------|
| 200 | Success - File returned |
| 400 | Bad Request - Invalid input data |
| 401 | Unauthorized - Missing or invalid token |
| 500 | Internal Server Error - Processing failed |

---

## Rate Limits
Currently no rate limiting is implemented. Consider adding rate limiting for production use.

---

## CORS Policy
CORS is enabled for all origins in development mode. Restrict origins in production.
