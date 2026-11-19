# HTTP API Reference

The `linter-server` HTTP service exposes versioned endpoints under `/api/v1`.

Base URL: `http://localhost:3000/api/v1`

---

## Endpoints

### 1. Health Check

`GET /api/v1/health`

Response:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 124
}
```

---

### 2. Audit Steam Listing

`POST /api/v1/audits/steam/:app_id`

Audits a listing directly using public Steam API data.

- **URL Parameters:**
  - `app_id`: Numeric Steam AppID or URL-encoded Steam store URL.
- **Query Parameters:**
  - `country` (optional, default: `us`): Country code for store pricing and currency.
  - `language` (optional, default: `english`): Language for store text.
  - `profile` (optional, default: `balanced`): `balanced` or `strict`.
  - `min_samples` (optional, default: `30`): Minimum cohort size for benchmarks.

Response:
Returns the complete `AuditReport` JSON object (see `docs/report-schema.md`).

---

### 3. Audit Imported Metadata JSON

`POST /api/v1/audits/import`

Audits uploaded JSON metadata without connecting to Steam.

- **Headers:** `Content-Type: application/json`
- **Body:** JSON object matching normalized `StoreListing`, Steam `appdetails` format, or benchmark row format.
- **Query Parameters:**
  - `profile` (optional, default: `balanced`): `balanced` or `strict`.
  - `min_samples` (optional, default: `30`): Minimum cohort size.

---

### 4. Retrieve Cached Audit Report

`GET /api/v1/audits/:audit_id`

Returns a previously generated audit report by its UUID.

---

### 5. Export Report

- `GET /api/v1/audits/:audit_id/json`: Downloads machine-readable JSON report file with `Content-Disposition: attachment`.
- `GET /api/v1/audits/:audit_id/html`: Downloads self-contained standalone HTML report file with `Content-Disposition: attachment`.

---

### 6. Example & Sample Reports

- `GET /api/v1/audits/example`: Downloads `example_listing.json`.
- `GET /api/v1/audits/sample-report`: Generates and returns a demonstration audit report.

---

### 7. Benchmarks & Dataset Status

- `GET /api/v1/benchmarks/status`: Returns database path, ingestion timestamp, and row counts.
- `GET /api/v1/benchmarks/genres`: Returns list of available genres and game counts.
- `GET /api/v1/benchmarks/years`: Returns list of release years and game counts.
- `GET /api/v1/benchmarks/metrics`: Returns metadata for all supported benchmark metrics.
- `GET /api/v1/benchmarks/cohort?genre=Action&year=2024&indie=true`: Queries distributions for a specific cohort.
