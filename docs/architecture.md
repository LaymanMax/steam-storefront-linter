# Architecture Overview — Steam Storefront Linter

Steam Storefront Linter is structured as a modular Rust workspace with a decoupled React + TypeScript frontend.

```text
steam-storefront-linter/
├── apps/
│   └── web/                   # React 19 + TypeScript + Vite frontend
├── crates/
│   ├── linter-core/           # Domain models, parser, rules engine, benchmarking, SQLite DB, report generation
│   ├── linter-server/         # Axum HTTP API & static asset server with rate-limiting & caching
│   └── linter-cli/            # Command-line interface (`steam-storefront-linter`)
├── data/
│   ├── raw/                   # (gitignored) Raw Parquet dataset from Hugging Face
│   ├── processed/             # (gitignored) Ingested SQLite benchmark database & manifest
│   └── fixtures/              # Tracked test fixtures (JSON & mocked responses)
└── docs/                      # Architectural, benchmark, API, and schema documentation
```

---

## 1. `linter-core` (Shared Domain & Logic)

The core crate encapsulates all business logic and is completely independent of HTTP routing or CLI flags.

- **`models`**:
  - `StoreListing`: Normalized Steam store listing model. Preserves distinctions between present values, present-but-empty values, and unknown values (never converts missing data to zero).
  - `Finding`: Structured findings with severity (`Critical`, `Warning`, `Info`, `Pass`), category, title, message, evidence, and actionable recommendation.
  - `BenchmarkResult`: Quantitative distributions (min, max, median, IQR, percentile rank, share above/below) and categorical adoption statistics.
  - `AuditReport`: Versioned report schema containing listing metadata, audit summary, findings, peer benchmarks, cohort definitions, and dataset methodology.
- **`parser`**:
  - `html.rs`: Tag stripping, block-level spacing, entity decoding, and word/character count extraction.
  - `language.rs`: Steam language metadata parsing, separating interface languages from audio footnotes (`<strong>*</strong>`).
  - `steam_import.rs`: Parses the live Steam Store API `appdetails` JSON envelope.
  - `json_import.rs`: Auto-detects and normalizes uploaded JSON formats with strict 10MB limits.
  - `steam_url.rs`: Validates numeric AppIDs and extracts them from Steam store URLs.
- **`rules`**:
  - `engine.rs`: Deterministic evaluation across 8 categories: Description, Screenshots, Trailers, Localization, Support, Features, Platform Support, and General Metadata.
  - `profile.rs`: Configurable audit profiles (`Balanced` vs `Strict`).
- **`dataset`**:
  - `db.rs`: SQLite storage with index optimization for cohort queries.
  - `download.rs`: Resilient streaming downloader for the Parquet dataset from Hugging Face.
  - `ingest.rs`: Streaming Arrow/Parquet reader, deduplication by AppID, and exclusion filtering.
  - `fixture.rs`: Offline sample benchmark database generator for zero-configuration testing.
- **`report`**:
  - `json.rs`: Stable JSON serializer.
  - `html.rs`: Standalone, self-contained HTML report generator with dark-theme styling, responsive design, and safe HTML escaping.

---

## 2. `linter-server` (HTTP API)

Built using **Axum** and **Tower-HTTP**:

- Exposes versioned endpoints under `/api/v1/`.
- Implements request validation, size limits (10MB body limit), and CORS for development.
- Wraps `SteamClient` with in-memory TTL caching and retry logic for transient failures.
- Directly serves the compiled production React assets from `apps/web/dist` when available.

---

## 3. `linter-cli` (Command-Line Interface)

Built using **Clap**:

- Binary: `steam-storefront-linter`
- Subcommands:
  - `audit <APP_ID_OR_URL>`: Direct terminal audit or report file export (`json` / `html`).
  - `dataset download | ingest | inspect | validate | sample`: Dataset ingestion and inspection.
  - `benchmark status | inspect`: Cohort and distribution inspection.
- Exit Codes:
  - `0`: Audit clean (no critical issues)
  - `1`: Critical findings detected (ideal for CI/CD gates)
  - `2`: Runtime error or invalid inputs.

---

## 4. `apps/web` (React Frontend)

Built using **React 19**, **TypeScript**, and **Vite**:

- Clean developer tool UI with custom CSS variable design tokens.
- Interactive filtering by severity and category.
- Rich benchmark distribution visualizations (quartiles, median, and percentile pin).
- Fully decoupled from raw Steam scraping; consumes normalized backend models.
