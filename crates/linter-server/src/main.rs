use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use linter_core::dataset::BenchmarkDb;
use linter_core::models::listing::StoreListing;
use linter_core::models::report::AuditReport;
use linter_core::parser::json_import::{get_example_listing_json, import_listing_json};
use linter_core::report::{generate_html_report, generate_json_report};
use linter_core::rules::profile::AuditProfile;
use linter_core::steam::SteamClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Clone)]
struct AppState {
    steam_client: SteamClient,
    db_path: PathBuf,
    reports_cache: Arc<RwLock<HashMap<String, AuditReport>>>,
    min_samples: usize,
    start_time: std::time::Instant,
}

#[derive(Deserialize)]
struct AuditQueryParams {
    country: Option<String>,
    language: Option<String>,
    profile: Option<String>,
    min_samples: Option<usize>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
    code: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "INVALID_INPUT" => StatusCode::BAD_REQUEST,
            "PAYLOAD_TOO_LARGE" => StatusCode::PAYLOAD_TOO_LARGE,
            "RATE_LIMITED" => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self)).into_response()
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let db_path = PathBuf::from(
        std::env::var("BENCHMARK_DB_PATH").unwrap_or_else(|_| "data/processed/benchmark.db".into()),
    );
    let min_samples = std::env::var("MIN_SAMPLES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(30);

    let state = AppState {
        steam_client: SteamClient::default(),
        db_path,
        reports_cache: Arc::new(RwLock::new(HashMap::new())),
        min_samples,
        start_time: std::time::Instant::now(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/audits/steam/:app_id", post(audit_steam_handler))
        .route("/audits/import", post(audit_import_handler))
        .route("/audits/:audit_id", get(get_audit_handler))
        .route("/audits/:audit_id/json", get(get_audit_json_handler))
        .route("/audits/:audit_id/html", get(get_audit_html_handler))
        .route("/audits/example", get(get_example_handler))
        .route("/audits/sample-report", get(get_sample_report_handler))
        .route("/benchmarks/status", get(get_benchmark_status_handler))
        .route("/benchmarks/genres", get(get_benchmark_genres_handler))
        .route("/benchmarks/years", get(get_benchmark_years_handler))
        .route("/benchmarks/metrics", get(get_benchmark_metrics_handler))
        .route("/benchmarks/cohort", get(get_benchmark_cohort_handler));

    let web_dist =
        PathBuf::from(std::env::var("WEB_DIST_PATH").unwrap_or_else(|_| "apps/web/dist".into()));
    let mut app = Router::new()
        .nest("/api/v1", api_routes)
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    if web_dist.exists() {
        info!("Serving static web assets from {}", web_dist.display());
        app = app.fallback_service(ServeDir::new(web_dist));
    }

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Valid socket address");

    info!(
        "Steam Storefront Linter server listening on http://{}",
        addr
    );
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: linter_core::version(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

async fn audit_steam_handler(
    State(state): State<AppState>,
    Path(app_id_str): Path<String>,
    Query(params): Query<AuditQueryParams>,
) -> Result<Json<AuditReport>, ApiError> {
    let app_id = linter_core::parse_app_id_or_url(&app_id_str).map_err(|e| ApiError {
        error: format!("Invalid AppID or Steam URL: {}", e),
        code: "INVALID_INPUT".into(),
    })?;

    let cc = params.country.as_deref().unwrap_or("us");
    let lang = params.language.as_deref().unwrap_or("english");
    let profile = params
        .profile
        .as_deref()
        .and_then(AuditProfile::parse)
        .unwrap_or(AuditProfile::Balanced);
    let min_samples = params.min_samples.unwrap_or(state.min_samples);

    info!(
        "Auditing Steam AppID {} (country: {}, language: {})",
        app_id, cc, lang
    );

    let listing = state
        .steam_client
        .fetch_listing(app_id, Some(cc), Some(lang))
        .await
        .map_err(|e| match e {
            linter_core::SteamClientError::RateLimited => ApiError {
                error: "Steam API rate limit reached. Please retry in a few moments.".into(),
                code: "RATE_LIMITED".into(),
            },
            linter_core::SteamClientError::ListingUnavailable(msg) => ApiError {
                error: msg,
                code: "NOT_FOUND".into(),
            },
            other => ApiError {
                error: other.to_string(),
                code: "STEAM_ERROR".into(),
            },
        })?;

    let db = if state.db_path.exists() {
        BenchmarkDb::open(&state.db_path).ok()
    } else {
        None
    };

    let report = linter_core::run_audit(listing, db.as_ref(), profile, min_samples);

    // Save to cache
    {
        let mut cache = state.reports_cache.write().await;
        cache.insert(report.audit_id.clone(), report.clone());
    }

    Ok(Json(report))
}

async fn audit_import_handler(
    State(state): State<AppState>,
    Query(params): Query<AuditQueryParams>,
    body_or_multipart: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<AuditReport>, ApiError> {
    let profile = params
        .profile
        .as_deref()
        .and_then(AuditProfile::parse)
        .unwrap_or(AuditProfile::Balanced);
    let min_samples = params.min_samples.unwrap_or(state.min_samples);

    let listing = match body_or_multipart {
        Ok(Json(val)) => {
            let json_str = serde_json::to_string(&val).unwrap_or_default();
            import_listing_json(&json_str).map_err(|e| ApiError {
                error: format!("Failed to parse JSON payload: {}", e),
                code: "INVALID_INPUT".into(),
            })?
        }
        Err(e) => {
            return Err(ApiError {
                error: format!("Invalid JSON request: {}", e),
                code: "INVALID_INPUT".into(),
            });
        }
    };

    info!(
        "Auditing imported listing: {} ({})",
        listing.name, listing.app_id
    );

    let db = if state.db_path.exists() {
        BenchmarkDb::open(&state.db_path).ok()
    } else {
        None
    };

    let report = linter_core::run_audit(listing, db.as_ref(), profile, min_samples);

    // Save to cache
    {
        let mut cache = state.reports_cache.write().await;
        cache.insert(report.audit_id.clone(), report.clone());
    }

    Ok(Json(report))
}

async fn get_audit_handler(
    State(state): State<AppState>,
    Path(audit_id): Path<String>,
) -> Result<Json<AuditReport>, ApiError> {
    let cache = state.reports_cache.read().await;
    if let Some(report) = cache.get(&audit_id) {
        Ok(Json(report.clone()))
    } else {
        Err(ApiError {
            error: format!("Audit report '{}' not found or expired", audit_id),
            code: "NOT_FOUND".into(),
        })
    }
}

async fn get_audit_json_handler(
    State(state): State<AppState>,
    Path(audit_id): Path<String>,
) -> Result<Response, ApiError> {
    let cache = state.reports_cache.read().await;
    if let Some(report) = cache.get(&audit_id) {
        let json = generate_json_report(report).map_err(|e| ApiError {
            error: format!("JSON generation error: {}", e),
            code: "INTERNAL_ERROR".into(),
        })?;

        let filename = format!("audit-{}-{}.json", report.listing.app_id, audit_id);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
        );

        Ok((headers, json).into_response())
    } else {
        Err(ApiError {
            error: format!("Audit report '{}' not found", audit_id),
            code: "NOT_FOUND".into(),
        })
    }
}

async fn get_audit_html_handler(
    State(state): State<AppState>,
    Path(audit_id): Path<String>,
) -> Result<Response, ApiError> {
    let cache = state.reports_cache.read().await;
    if let Some(report) = cache.get(&audit_id) {
        let html = generate_html_report(report);
        let filename = format!("audit-{}-{}.html", report.listing.app_id, audit_id);
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        headers.insert(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename)).unwrap(),
        );

        Ok((headers, html).into_response())
    } else {
        Err(ApiError {
            error: format!("Audit report '{}' not found", audit_id),
            code: "NOT_FOUND".into(),
        })
    }
}

async fn get_example_handler() -> Response {
    let json = get_example_listing_json();
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"example_listing.json\""),
    );
    (headers, json).into_response()
}

async fn get_sample_report_handler(State(state): State<AppState>) -> Json<AuditReport> {
    let example_json = get_example_listing_json();
    let listing = import_listing_json(example_json).unwrap();
    let db = if state.db_path.exists() {
        BenchmarkDb::open(&state.db_path).ok()
    } else {
        None
    };

    let report = linter_core::run_audit(
        listing,
        db.as_ref(),
        AuditProfile::Balanced,
        state.min_samples,
    );
    {
        let mut cache = state.reports_cache.write().await;
        cache.insert(report.audit_id.clone(), report.clone());
    }
    Json(report)
}

async fn get_benchmark_status_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let exists = state.db_path.exists();
    if exists {
        if let Ok(db) = BenchmarkDb::open(&state.db_path) {
            if let Ok(Some(meta)) = db.get_metadata() {
                return Json(serde_json::json!({
                    "available": true,
                    "database_path": state.db_path.to_string_lossy(),
                    "metadata": meta,
                }));
            }
        }
    }

    Json(serde_json::json!({
        "available": false,
        "database_path": state.db_path.to_string_lossy(),
        "message": "Benchmark database has not been downloaded or ingested yet. Use 'steam-storefront-linter dataset download' and 'ingest' to initialize.",
    }))
}

async fn get_benchmark_genres_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<(String, usize)>>, ApiError> {
    if !state.db_path.exists() {
        return Ok(Json(Vec::new()));
    }
    let db = BenchmarkDb::open(&state.db_path).map_err(|e| ApiError {
        error: e.to_string(),
        code: "DB_ERROR".into(),
    })?;

    let genres = db.get_available_genres().unwrap_or_default();
    Ok(Json(genres))
}

async fn get_benchmark_years_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<(i32, usize)>>, ApiError> {
    if !state.db_path.exists() {
        return Ok(Json(Vec::new()));
    }
    let db = BenchmarkDb::open(&state.db_path).map_err(|e| ApiError {
        error: e.to_string(),
        code: "DB_ERROR".into(),
    })?;

    let years = db.get_available_years().unwrap_or_default();
    Ok(Json(years))
}

async fn get_benchmark_metrics_handler() -> Json<Vec<serde_json::Value>> {
    Json(vec![
        serde_json::json!({
            "id": "screenshots.count",
            "name": "Screenshot Count",
            "type": "distribution",
            "description": "Number of in-game screenshots uploaded to the store listing."
        }),
        serde_json::json!({
            "id": "trailers.presence",
            "name": "Trailer Presence",
            "type": "categorical",
            "description": "Share of comparable games with at least one video trailer."
        }),
        serde_json::json!({
            "id": "trailers.count",
            "name": "Trailer Count",
            "type": "distribution",
            "description": "Count distribution of video trailers."
        }),
        serde_json::json!({
            "id": "description.detailed_words",
            "name": "Detailed Description Word Count",
            "type": "distribution",
            "description": "Readable word count in 'About This Game' after stripping HTML markup."
        }),
        serde_json::json!({
            "id": "description.short_words",
            "name": "Short Description Word Count",
            "type": "distribution",
            "description": "Elevator pitch word count displayed at top-right of store listing."
        }),
        serde_json::json!({
            "id": "localization.interface_languages",
            "name": "Interface Languages Count",
            "type": "distribution",
            "description": "Number of supported interface languages declared."
        }),
        serde_json::json!({
            "id": "localization.full_audio_languages",
            "name": "Full Audio Languages Count",
            "type": "distribution",
            "description": "Number of full-audio voiceover languages declared."
        }),
        serde_json::json!({
            "id": "support.contact_presence",
            "name": "Support Contact Availability",
            "type": "categorical",
            "description": "Share of games declaring a dedicated support URL or support email."
        }),
        serde_json::json!({
            "id": "platforms.linux",
            "name": "Linux Support",
            "type": "categorical",
            "description": "Share of games providing native Linux/SteamOS support."
        }),
        serde_json::json!({
            "id": "platforms.mac",
            "name": "macOS Support",
            "type": "categorical",
            "description": "Share of games providing native macOS support."
        }),
        serde_json::json!({
            "id": "features.categories_count",
            "name": "Declared Steam Categories",
            "type": "distribution",
            "description": "Count distribution of feature categories tagged in Steamworks."
        }),
    ])
}

#[derive(Deserialize)]
struct CohortQuery {
    genre: Option<String>,
    year: Option<i32>,
    indie: Option<bool>,
}

async fn get_benchmark_cohort_handler(
    State(state): State<AppState>,
    Query(query): Query<CohortQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !state.db_path.exists() {
        return Err(ApiError {
            error: "Benchmark database not found".into(),
            code: "NOT_FOUND".into(),
        });
    }

    let db = BenchmarkDb::open(&state.db_path).map_err(|e| ApiError {
        error: e.to_string(),
        code: "DB_ERROR".into(),
    })?;

    let genres = query.genre.map(|g| vec![g]).unwrap_or_default();
    let is_indie = query.indie.unwrap_or(true);

    let (cohort, games) = db
        .select_cohort(&genres, query.year, is_indie, state.min_samples)
        .map_err(|e| ApiError {
            error: e.to_string(),
            code: "DB_ERROR".into(),
        })?;

    // Generate metric summary for this cohort
    let dummy = StoreListing::new(
        0,
        "Inspection Dummy",
        linter_core::models::listing::ListingSource::SampleFixture,
    );
    let metrics =
        BenchmarkDb::calculate_benchmark_metrics(&dummy, &cohort, &games, state.min_samples);

    Ok(Json(serde_json::json!({
        "cohort": cohort,
        "metrics": metrics,
    })))
}
