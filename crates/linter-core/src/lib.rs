pub mod dataset;
pub mod models;
pub mod parser;
pub mod report;
pub mod rules;
pub mod steam;

pub use dataset::{
    generate_sample_benchmark_db, BenchmarkDb, BenchmarkGameRow, DatasetDownloader,
    DatasetIngester, DatasetMetadata, DbError, IngestError, IngestionSummary, DEFAULT_DATASET_URL,
};
pub use models::{
    AuditDataQuality, AuditReport, AuditSummary, BenchmarkDistribution, BenchmarkMethodology,
    BenchmarkResult, BenchmarkStatus, CategoricalBenchmark, CohortInfo, Finding, FindingCategory,
    FindingEvidence, ListingSource, Movie, PlatformSupport, PriceInfo, Screenshot, Severity,
    StoreListing, REPORT_SCHEMA_VERSION,
};
pub use parser::{
    count_characters, count_words, decode_html_entities, extract_release_year,
    get_example_listing_json, import_listing_json, normalize_language_list, parse_app_id_or_url,
    parse_steam_app_details, parse_steam_languages, strip_html_tags, AppIdParseError,
    JsonImportError, ParsedLanguages, SteamImportError,
};
pub use report::{escape_html, generate_html_report, generate_json_report, JsonReportError};
pub use rules::{AuditEngine, AuditProfile, RULESET_VERSION};
pub use steam::{SteamClient, SteamClientError};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Runs a deterministic storefront audit on a listing, calculating findings and peer benchmark comparisons.
pub fn run_audit(
    listing: StoreListing,
    db: Option<&BenchmarkDb>,
    profile: AuditProfile,
    min_samples: usize,
) -> AuditReport {
    // 1. Run deterministic rules engine
    let findings = AuditEngine::audit(&listing, profile);

    // 2. Select benchmark cohort and calculate comparisons if DB is available
    let (benchmarks, cohort, total_games, snapshot_date) = if let Some(benchmark_db) = db {
        let meta = benchmark_db
            .get_metadata()
            .ok()
            .flatten()
            .unwrap_or_default();
        let target_genres = listing.genres.clone().unwrap_or_default();
        let target_year = listing.release_year;
        let is_indie = listing.is_indie();

        match benchmark_db.select_cohort(&target_genres, target_year, is_indie, min_samples) {
            Ok((cohort_info, games)) => {
                let metrics = BenchmarkDb::calculate_benchmark_metrics(
                    &listing,
                    &cohort_info,
                    &games,
                    min_samples,
                );
                let snap = meta
                    .ingested_at
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "2024-11".to_string());
                (
                    metrics,
                    Some(cohort_info),
                    meta.eligible_benchmark_count,
                    snap,
                )
            }
            Err(_) => (
                Vec::new(),
                None,
                meta.eligible_benchmark_count,
                "Unavailable".into(),
            ),
        }
    } else {
        (Vec::new(), None, 0, "No dataset loaded".into())
    };

    let methodology = BenchmarkMethodology {
        dataset_source: "FronkonGames/steam-games-dataset".into(),
        dataset_license: "MIT".into(),
        snapshot_date,
        total_dataset_games: total_games,
        min_sample_size_threshold: min_samples,
        percentile_convention: "Linear rank interpolation (percentile rank = [less + 0.5 * equal] / total)".into(),
        disclaimer: "Benchmark distributions are descriptive references based on public Steam games. Listing completeness does not predict sales, commercial revenue, or review sentiment.".into(),
    };

    AuditReport::new(listing, findings, benchmarks, cohort, methodology, profile)
}
