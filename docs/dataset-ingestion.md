# Dataset Ingestion Pipeline

## Source Dataset

- **Name:** FronkonGames/steam-games-dataset
- **Source URL:** [https://huggingface.co/datasets/FronkonGames/steam-games-dataset](https://huggingface.co/datasets/FronkonGames/steam-games-dataset)
- **Artifact:** `data/train-00000-of-00001.parquet` (~191.98 MB)
- **License:** MIT License
- **Attribution:** Maintained by Fronkon Games (MIT), generated via Steam Store API & SteamSpy.

---

## Ingestion Architecture

```text
[ Hugging Face Parquet ]
           │
           ▼
[ DatasetDownloader ] (Streaming download with retry & resume)
           │
           ▼  data/raw/train-00000-of-00001.parquet
[ DatasetIngester ] (Arrow RecordBatch reader)
    ├── Validation (AppID > 0)
    ├── Deduplication (Set of unique AppIDs)
    ├── Quality Filtering:
    │     ├── Playtest Exclusion (e.g. AppID 2539430 "Black Dragon Mage Playtest")
    │     └── Sparse Metadata Exclusion (missing short & detailed descriptions)
    └── Feature Normalization:
          ├── Language Footnote Parsing (Interface vs Audio)
          ├── Date / Year Extraction (e.g. "15 Aug, 2024" -> 2024)
          ├── Description Word & Character Counts (HTML stripped)
          └── Media / Screenshot Arrays
           │
           ▼
[ BenchmarkDb (SQLite) ]  data/processed/benchmark.db
[ Ingestion Manifest ]    data/processed/ingestion_manifest.json
```

---

## Quality Exclusion Criteria

1. **`title_contains_playtest`**:
   - Records whose name contains the substring `"playtest"` (case-insensitive).
   - Reason: Steam Playtests are auxiliary transient listings with stripped metadata, unpopulated trailers, and zero owners. Including them skews benchmark distributions downward.
2. **`sparse_metadata`**:
   - Records lacking both short description and detailed description.
   - Reason: Unusable for comparison metrics.
3. **`duplicate_appid`**:
   - When multiple records share the same AppID, only the first encountered row is indexed.
4. **`invalid_appid`**:
   - Non-positive or non-numeric AppIDs.

All excluded records are tracked with their exact exclusion reason in the database (`is_excluded = 1`, `exclusion_reason = '...'`) and summarized in the manifest.

---

## CLI Ingestion Commands

```bash
# 1. Download Parquet dataset from Hugging Face
steam-storefront-linter dataset download

# 2. Ingest into SQLite database
steam-storefront-linter dataset ingest

# 3. Inspect dataset metadata and row counts
steam-storefront-linter dataset inspect

# 4. Validate database integrity
steam-storefront-linter dataset validate

# 5. Initialize sample benchmark database for offline testing
steam-storefront-linter dataset sample
```
