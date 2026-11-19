use crate::dataset::models::DatasetMetadata;
use crate::models::benchmark::{
    BenchmarkDistribution, BenchmarkResult, BenchmarkStatus, CategoricalBenchmark, CohortInfo,
};
use crate::models::finding::FindingCategory;
use crate::models::listing::StoreListing;
use rusqlite::{params, Connection};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Failed to serialize or deserialize JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct BenchmarkGameRow {
    pub app_id: u32,
    pub name: String,
    pub release_year: Option<i32>,
    pub is_indie: bool,
    pub genres: Vec<String>,
    pub screenshots_count: Option<usize>,
    pub movies_count: Option<usize>,
    pub has_trailer: Option<bool>,
    pub short_desc_words: Option<usize>,
    pub short_desc_chars: Option<usize>,
    pub detailed_desc_words: Option<usize>,
    pub detailed_desc_chars: Option<usize>,
    pub interface_languages_count: Option<usize>,
    pub full_audio_languages_count: Option<usize>,
    pub has_support_contact: Option<bool>,
    pub has_support_url: Option<bool>,
    pub has_support_email: Option<bool>,
    pub supports_windows: Option<bool>,
    pub supports_mac: Option<bool>,
    pub supports_linux: Option<bool>,
    pub categories_count: Option<usize>,
    pub is_excluded: bool,
    pub exclusion_reason: Option<String>,
}

pub struct BenchmarkDb {
    conn: Connection,
}

impl BenchmarkDb {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn init_schema(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS dataset_metadata (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                data TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS games (
                app_id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                release_year INTEGER,
                is_indie BOOLEAN NOT NULL,
                genres TEXT NOT NULL,
                tags TEXT NOT NULL,
                categories TEXT NOT NULL,
                screenshots_count INTEGER,
                movies_count INTEGER,
                has_trailer BOOLEAN,
                short_desc_words INTEGER,
                short_desc_chars INTEGER,
                detailed_desc_words INTEGER,
                detailed_desc_chars INTEGER,
                interface_languages_count INTEGER,
                full_audio_languages_count INTEGER,
                has_support_contact BOOLEAN,
                has_support_url BOOLEAN,
                has_support_email BOOLEAN,
                supports_windows BOOLEAN,
                supports_mac BOOLEAN,
                supports_linux BOOLEAN,
                categories_count INTEGER,
                is_excluded BOOLEAN NOT NULL,
                exclusion_reason TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_games_cohort ON games (is_excluded, is_indie, release_year);
            CREATE INDEX IF NOT EXISTS idx_games_year ON games (release_year);
            "#,
        )?;
        Ok(())
    }

    pub fn save_metadata(&self, meta: &DatasetMetadata) -> Result<(), DbError> {
        let json = serde_json::to_string(meta)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO dataset_metadata (id, data) VALUES (1, ?1)",
            params![json],
        )?;
        Ok(())
    }

    pub fn get_metadata(&self) -> Result<Option<DatasetMetadata>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT data FROM dataset_metadata WHERE id = 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            let meta: DatasetMetadata = serde_json::from_str(&json)?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }

    pub fn insert_game(&self, game: &BenchmarkGameRow) -> Result<(), DbError> {
        let genres_json = serde_json::to_string(&game.genres)?;
        let tags_json = "[]";
        let categories_json = "[]";

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO games (
                app_id, name, release_year, is_indie, genres, tags, categories,
                screenshots_count, movies_count, has_trailer,
                short_desc_words, short_desc_chars, detailed_desc_words, detailed_desc_chars,
                interface_languages_count, full_audio_languages_count,
                has_support_contact, has_support_url, has_support_email,
                supports_windows, supports_mac, supports_linux,
                categories_count, is_excluded, exclusion_reason
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10,
                ?11, ?12, ?13, ?14,
                ?15, ?16,
                ?17, ?18, ?19,
                ?20, ?21, ?22,
                ?23, ?24, ?25
            )
            "#,
            params![
                game.app_id,
                game.name,
                game.release_year,
                game.is_indie,
                genres_json,
                tags_json,
                categories_json,
                game.screenshots_count.map(|v| v as i64),
                game.movies_count.map(|v| v as i64),
                game.has_trailer,
                game.short_desc_words.map(|v| v as i64),
                game.short_desc_chars.map(|v| v as i64),
                game.detailed_desc_words.map(|v| v as i64),
                game.detailed_desc_chars.map(|v| v as i64),
                game.interface_languages_count.map(|v| v as i64),
                game.full_audio_languages_count.map(|v| v as i64),
                game.has_support_contact,
                game.has_support_url,
                game.has_support_email,
                game.supports_windows,
                game.supports_mac,
                game.supports_linux,
                game.categories_count.map(|v| v as i64),
                game.is_excluded,
                game.exclusion_reason,
            ],
        )?;
        Ok(())
    }

    pub fn get_available_genres(&self) -> Result<Vec<(String, usize)>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT genres FROM games WHERE is_excluded = 0")?;
        let rows = stmt.query_map([], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;

        let mut counts = std::collections::HashMap::new();
        for r in rows {
            let json = r?;
            if let Ok(genres) = serde_json::from_str::<Vec<String>>(&json) {
                for g in genres {
                    *counts.entry(g).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<(String, usize)> = counts.into_iter().collect();
        result.sort_by_key(|a| std::cmp::Reverse(a.1));
        Ok(result)
    }

    pub fn get_available_years(&self) -> Result<Vec<(i32, usize)>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT release_year, COUNT(*) FROM games WHERE is_excluded = 0 AND release_year IS NOT NULL GROUP BY release_year ORDER BY release_year DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let year: i32 = row.get(0)?;
            let count: usize = row.get(1)?;
            Ok((year, count))
        })?;

        let mut result = Vec::new();
        for r in rows {
            result.push(r?);
        }
        Ok(result)
    }

    /// Selects a cohort of comparable games based on genres, release year, and indie status,
    /// applying a multi-stage documented fallback policy when samples are limited.
    pub fn select_cohort(
        &self,
        target_genres: &[String],
        target_year: Option<i32>,
        target_is_indie: bool,
        min_samples: usize,
    ) -> Result<(CohortInfo, Vec<BenchmarkGameRow>), DbError> {
        // Fetch all non-excluded games once into memory for fast multi-stage cohort evaluation
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                app_id, name, release_year, is_indie, genres,
                screenshots_count, movies_count, has_trailer,
                short_desc_words, short_desc_chars, detailed_desc_words, detailed_desc_chars,
                interface_languages_count, full_audio_languages_count,
                has_support_contact, has_support_url, has_support_email,
                supports_windows, supports_mac, supports_linux,
                categories_count, is_excluded, exclusion_reason
            FROM games
            WHERE is_excluded = 0
            "#,
        )?;

        let game_rows = stmt.query_map([], |row| {
            let genres_json: String = row.get(4)?;
            let genres: Vec<String> = serde_json::from_str(&genres_json).unwrap_or_default();
            Ok(BenchmarkGameRow {
                app_id: row.get(0)?,
                name: row.get(1)?,
                release_year: row.get(2)?,
                is_indie: row.get(3)?,
                genres,
                screenshots_count: row.get::<_, Option<i64>>(5)?.map(|v| v as usize),
                movies_count: row.get::<_, Option<i64>>(6)?.map(|v| v as usize),
                has_trailer: row.get(7)?,
                short_desc_words: row.get::<_, Option<i64>>(8)?.map(|v| v as usize),
                short_desc_chars: row.get::<_, Option<i64>>(9)?.map(|v| v as usize),
                detailed_desc_words: row.get::<_, Option<i64>>(10)?.map(|v| v as usize),
                detailed_desc_chars: row.get::<_, Option<i64>>(11)?.map(|v| v as usize),
                interface_languages_count: row.get::<_, Option<i64>>(12)?.map(|v| v as usize),
                full_audio_languages_count: row.get::<_, Option<i64>>(13)?.map(|v| v as usize),
                has_support_contact: row.get(14)?,
                has_support_url: row.get(15)?,
                has_support_email: row.get(16)?,
                supports_windows: row.get(17)?,
                supports_mac: row.get(18)?,
                supports_linux: row.get(19)?,
                categories_count: row.get::<_, Option<i64>>(20)?.map(|v| v as usize),
                is_excluded: row.get(21)?,
                exclusion_reason: row.get(22)?,
            })
        })?;

        let mut all_games = Vec::new();
        for g in game_rows {
            all_games.push(g?);
        }

        let shares_genre = |game: &BenchmarkGameRow| -> bool {
            if target_genres.is_empty() {
                return true;
            }
            game.genres
                .iter()
                .any(|g| target_genres.iter().any(|tg| tg.eq_ignore_ascii_case(g)))
        };

        // Multi-stage fallback:
        // Stage 1: Same genre(s) + same release year + Indie (if target is indie)
        let mut cohort_games: Vec<BenchmarkGameRow> = all_games
            .iter()
            .filter(|g| {
                shares_genre(g)
                    && (target_year.is_none() || g.release_year == target_year)
                    && (!target_is_indie || g.is_indie)
            })
            .cloned()
            .collect();

        let mut stage_name = format!(
            "{}{} games sharing genre(s) [{}]",
            if target_is_indie { "Indie " } else { "" },
            target_year
                .map(|y| format!("released in {} ", y))
                .unwrap_or_default(),
            target_genres.join(", ")
        );
        let mut fallback_applied = false;
        let mut explanation = format!(
            "Specific cohort matching target genres ({}) and release year ({:?}).",
            target_genres.join(", "),
            target_year
        );

        // Stage 2: Relax indie requirement if count < min_samples
        if cohort_games.len() < min_samples && target_is_indie {
            let stage2: Vec<BenchmarkGameRow> = all_games
                .iter()
                .filter(|g| {
                    shares_genre(g) && (target_year.is_none() || g.release_year == target_year)
                })
                .cloned()
                .collect();
            if stage2.len() >= min_samples || stage2.len() > cohort_games.len() {
                cohort_games = stage2;
                fallback_applied = true;
                stage_name = format!(
                    "All games sharing genre(s) [{}] {}",
                    target_genres.join(", "),
                    target_year
                        .map(|y| format!("released in {}", y))
                        .unwrap_or_default()
                );
                explanation = "Broadened cohort: relaxed Indie restriction to achieve sufficient sample size.".into();
            }
        }

        // Stage 3: Relax release year if count < min_samples
        if cohort_games.len() < min_samples && !target_genres.is_empty() {
            let stage3: Vec<BenchmarkGameRow> = all_games
                .iter()
                .filter(|g| shares_genre(g))
                .cloned()
                .collect();
            if stage3.len() >= min_samples || stage3.len() > cohort_games.len() {
                cohort_games = stage3;
                fallback_applied = true;
                stage_name = format!(
                    "Games sharing genre(s) [{}] across all years",
                    target_genres.join(", ")
                );
                explanation = "Broadened cohort: included games across all release years sharing primary genres.".into();
            }
        }

        // Stage 4: If still too small, use all indie games in same release year
        if let Some(year) = target_year {
            if cohort_games.len() < min_samples {
                let stage4: Vec<BenchmarkGameRow> = all_games
                    .iter()
                    .filter(|g| g.release_year == Some(year))
                    .cloned()
                    .collect();
                if stage4.len() >= min_samples {
                    cohort_games = stage4;
                    fallback_applied = true;
                    stage_name = format!("All games released in {}", year);
                    explanation = "Broadened cohort: sample broadened to all games released in the same year due to niche genres.".into();
                }
            }
        }

        // Stage 5: Broad fallback (all eligible games)
        if cohort_games.len() < min_samples && all_games.len() >= min_samples {
            cohort_games = all_games.clone();
            fallback_applied = true;
            stage_name = "Broad Steam dataset benchmark cohort".into();
            explanation =
                "Broad cohort: using entire eligible benchmark dataset as a baseline comparison."
                    .into();
        }

        let cohort_info = CohortInfo {
            cohort_name: stage_name,
            total_games_in_cohort: cohort_games.len(),
            genres: target_genres.to_vec(),
            release_year: target_year,
            is_indie_restricted: target_is_indie && !fallback_applied,
            fallback_applied,
            explanation,
        };

        Ok((cohort_info, cohort_games))
    }

    /// Calculates comparative metrics for a listing against a cohort of games.
    pub fn calculate_benchmark_metrics(
        listing: &StoreListing,
        cohort: &CohortInfo,
        games: &[BenchmarkGameRow],
        min_samples: usize,
    ) -> Vec<BenchmarkResult> {
        let _sample_size = games.len();
        let mut results = Vec::new();

        // 1. Screenshot count
        results.push(Self::calc_distribution_metric(
            "screenshots.count",
            "Screenshot Count",
            FindingCategory::Screenshots,
            listing.screenshot_count().map(|v| v as f64),
            games,
            |g| g.screenshots_count.map(|v| v as f64),
            cohort,
            min_samples,
            "Screenshots",
        ));

        // 2. Trailer presence (categorical)
        results.push(Self::calc_categorical_metric(
            "trailers.presence",
            "Trailer Presence",
            FindingCategory::Trailers,
            listing.movie_count().map(|c| c > 0),
            games,
            |g| g.has_trailer,
            cohort,
            min_samples,
            "Listing has at least 1 video trailer",
        ));

        // 3. Trailer count
        results.push(Self::calc_distribution_metric(
            "trailers.count",
            "Video Trailer Count",
            FindingCategory::Trailers,
            listing.movie_count().map(|v| v as f64),
            games,
            |g| g.movies_count.map(|v| v as f64),
            cohort,
            min_samples,
            "Trailers",
        ));

        // 4. Detailed description word count
        let detailed_words = listing.detailed_description.as_ref().map(|d| {
            let clean = crate::parser::html::strip_html_tags(d);
            crate::parser::html::count_words(&clean) as f64
        });
        results.push(Self::calc_distribution_metric(
            "description.detailed_words",
            "Detailed Description Word Count",
            FindingCategory::Description,
            detailed_words,
            games,
            |g| g.detailed_desc_words.map(|v| v as f64),
            cohort,
            min_samples,
            "Words",
        ));

        // 5. Short description word count
        let short_words = listing.short_description.as_ref().map(|d| {
            let clean = crate::parser::html::strip_html_tags(d);
            crate::parser::html::count_words(&clean) as f64
        });
        results.push(Self::calc_distribution_metric(
            "description.short_words",
            "Short Description Word Count",
            FindingCategory::Description,
            short_words,
            games,
            |g| g.short_desc_words.map(|v| v as f64),
            cohort,
            min_samples,
            "Words",
        ));

        // 6. Supported interface languages
        results.push(Self::calc_distribution_metric(
            "localization.interface_languages",
            "Supported Interface Languages",
            FindingCategory::Localization,
            listing.supported_language_count().map(|v| v as f64),
            games,
            |g| g.interface_languages_count.map(|v| v as f64),
            cohort,
            min_samples,
            "Languages",
        ));

        // 7. Full audio languages
        results.push(Self::calc_distribution_metric(
            "localization.full_audio_languages",
            "Full Audio Languages",
            FindingCategory::Localization,
            listing.full_audio_language_count().map(|v| v as f64),
            games,
            |g| g.full_audio_languages_count.map(|v| v as f64),
            cohort,
            min_samples,
            "Audio Languages",
        ));

        // 8. Support contact presence (categorical)
        let has_support = if listing.support_url.is_some() || listing.support_email.is_some() {
            Some(true)
        } else if listing.provenance.contains_key("support_url")
            || listing.provenance.contains_key("support_email")
        {
            Some(false)
        } else {
            None
        };
        results.push(Self::calc_categorical_metric(
            "support.contact_presence",
            "Support Contact Availability",
            FindingCategory::Support,
            has_support,
            games,
            |g| g.has_support_contact,
            cohort,
            min_samples,
            "Has dedicated support URL or email",
        ));

        // 9. Linux platform support (categorical)
        let has_linux = listing.platforms.as_ref().map(|p| p.linux);
        results.push(Self::calc_categorical_metric(
            "platforms.linux",
            "Linux Native Support",
            FindingCategory::Platforms,
            has_linux,
            games,
            |g| g.supports_linux,
            cohort,
            min_samples,
            "Supports Linux / SteamOS natively",
        ));

        // 10. macOS platform support (categorical)
        let has_mac = listing.platforms.as_ref().map(|p| p.mac);
        results.push(Self::calc_categorical_metric(
            "platforms.mac",
            "macOS Native Support",
            FindingCategory::Platforms,
            has_mac,
            games,
            |g| g.supports_mac,
            cohort,
            min_samples,
            "Supports macOS natively",
        ));

        // 11. Feature categories count
        results.push(Self::calc_distribution_metric(
            "features.categories_count",
            "Declared Steam Categories",
            FindingCategory::Features,
            listing.category_count().map(|v| v as f64),
            games,
            |g| g.categories_count.map(|v| v as f64),
            cohort,
            min_samples,
            "Categories",
        ));

        results
    }

    #[allow(clippy::too_many_arguments)]
    fn calc_distribution_metric<F>(
        metric_id: &str,
        metric_name: &str,
        category: FindingCategory,
        listing_val: Option<f64>,
        games: &[BenchmarkGameRow],
        extractor: F,
        cohort: &CohortInfo,
        min_samples: usize,
        unit: &str,
    ) -> BenchmarkResult
    where
        F: Fn(&BenchmarkGameRow) -> Option<f64>,
    {
        let sample_size = games.len();
        let mut usable_values: Vec<f64> = games.iter().filter_map(&extractor).collect();
        let usable_records = usable_values.len();
        let coverage_pct = if sample_size > 0 {
            ((usable_records as f64 / sample_size as f64) * 100.0 * 10.0).round() / 10.0
        } else {
            0.0
        };

        let listing_value_display = match listing_val {
            Some(v) => format!("{} {}", v as i64, unit),
            None => "Unknown / Unavailable".to_string(),
        };

        if sample_size == 0 {
            return BenchmarkResult {
                metric_id: metric_id.to_string(),
                metric_name: metric_name.to_string(),
                category,
                listing_value_display,
                status: BenchmarkStatus::DatasetUnavailable,
                distribution: None,
                categorical: None,
                cohort_definition: cohort.cohort_name.clone(),
                explanation: Some(
                    "Benchmark dataset is not yet downloaded or ingested.".to_string(),
                ),
            };
        }

        if usable_records < min_samples {
            return BenchmarkResult {
                metric_id: metric_id.to_string(),
                metric_name: metric_name.to_string(),
                category,
                listing_value_display,
                status: BenchmarkStatus::InsufficientSample,
                distribution: Some(BenchmarkDistribution {
                    sample_size,
                    usable_records,
                    coverage_pct,
                    listing_value: listing_val,
                    min: None,
                    max: None,
                    p25: None,
                    median: None,
                    p75: None,
                    percentile_rank: None,
                    share_above: None,
                    share_below: None,
                }),
                categorical: None,
                cohort_definition: cohort.cohort_name.clone(),
                explanation: Some(format!(
                    "Only {} usable record(s) available in this cohort (minimum required threshold is {}).",
                    usable_records, min_samples
                )),
            };
        }

        if listing_val.is_none() {
            return BenchmarkResult {
                metric_id: metric_id.to_string(),
                metric_name: metric_name.to_string(),
                category,
                listing_value_display,
                status: BenchmarkStatus::MissingListingData,
                distribution: None,
                categorical: None,
                cohort_definition: cohort.cohort_name.clone(),
                explanation: Some("This metric could not be compared because the listing did not provide this data.".to_string()),
            };
        }

        let l_val = listing_val.unwrap();
        usable_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = usable_values.first().copied();
        let max = usable_values.last().copied();

        let median = calc_percentile(&usable_values, 0.50);
        let p25 = calc_percentile(&usable_values, 0.25);
        let p75 = calc_percentile(&usable_values, 0.75);

        // Percentile rank: share of usable values below + half of equals
        let count_less = usable_values.iter().filter(|&&x| x < l_val).count();
        let count_equal = usable_values
            .iter()
            .filter(|&&x| (x - l_val).abs() < f64::EPSILON)
            .count();
        let count_greater = usable_values.iter().filter(|&&x| x > l_val).count();

        let percentile_rank = (((count_less as f64 + 0.5 * count_equal as f64)
            / usable_records as f64)
            * 100.0
            * 10.0)
            .round()
            / 10.0;
        let share_above =
            ((count_greater as f64 / usable_records as f64) * 100.0 * 10.0).round() / 10.0;
        let share_below =
            ((count_less as f64 / usable_records as f64) * 100.0 * 10.0).round() / 10.0;

        BenchmarkResult {
            metric_id: metric_id.to_string(),
            metric_name: metric_name.to_string(),
            category,
            listing_value_display,
            status: BenchmarkStatus::Calculated,
            distribution: Some(BenchmarkDistribution {
                sample_size,
                usable_records,
                coverage_pct,
                listing_value: Some(l_val),
                min,
                max,
                p25: Some(p25),
                median: Some(median),
                p75: Some(p75),
                percentile_rank: Some(percentile_rank),
                share_above: Some(share_above),
                share_below: Some(share_below),
            }),
            categorical: None,
            cohort_definition: cohort.cohort_name.clone(),
            explanation: Some(format!(
                "Your listing's {} ({:.0}) is at the {:.0}th percentile. Approximately {:.0}% of comparable games have a higher value.",
                metric_name.to_lowercase(),
                l_val,
                percentile_rank,
                share_above
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn calc_categorical_metric<F>(
        metric_id: &str,
        metric_name: &str,
        category: FindingCategory,
        listing_val: Option<bool>,
        games: &[BenchmarkGameRow],
        extractor: F,
        cohort: &CohortInfo,
        min_samples: usize,
        _explanation_label: &str,
    ) -> BenchmarkResult
    where
        F: Fn(&BenchmarkGameRow) -> Option<bool>,
    {
        let sample_size = games.len();
        let usable_pairs: Vec<bool> = games.iter().filter_map(&extractor).collect();
        let usable_records = usable_pairs.len();
        let coverage_pct = if sample_size > 0 {
            ((usable_records as f64 / sample_size as f64) * 100.0 * 10.0).round() / 10.0
        } else {
            0.0
        };

        let listing_value_display = match listing_val {
            Some(true) => "Yes / Supported".to_string(),
            Some(false) => "No / Not Supported".to_string(),
            None => "Unknown".to_string(),
        };

        if sample_size == 0 {
            return BenchmarkResult {
                metric_id: metric_id.to_string(),
                metric_name: metric_name.to_string(),
                category,
                listing_value_display,
                status: BenchmarkStatus::DatasetUnavailable,
                distribution: None,
                categorical: None,
                cohort_definition: cohort.cohort_name.clone(),
                explanation: Some(
                    "Benchmark dataset is not yet downloaded or ingested.".to_string(),
                ),
            };
        }

        if usable_records < min_samples {
            return BenchmarkResult {
                metric_id: metric_id.to_string(),
                metric_name: metric_name.to_string(),
                category,
                listing_value_display,
                status: BenchmarkStatus::InsufficientSample,
                distribution: None,
                categorical: None,
                cohort_definition: cohort.cohort_name.clone(),
                explanation: Some(format!(
                    "Only {} usable records available (minimum required: {}).",
                    usable_records, min_samples
                )),
            };
        }

        let true_count = usable_pairs.iter().filter(|&&v| v).count();
        let false_count = usable_records - true_count;
        let true_percentage =
            ((true_count as f64 / usable_records as f64) * 100.0 * 10.0).round() / 10.0;

        BenchmarkResult {
            metric_id: metric_id.to_string(),
            metric_name: metric_name.to_string(),
            category,
            listing_value_display,
            status: BenchmarkStatus::Calculated,
            distribution: None,
            categorical: Some(CategoricalBenchmark {
                sample_size,
                usable_records,
                coverage_pct,
                listing_has_feature: listing_val,
                true_count,
                false_count,
                true_percentage,
            }),
            cohort_definition: cohort.cohort_name.clone(),
            explanation: Some(format!(
                "{:.0}% of comparable games have this feature ({}/{} usable games).",
                true_percentage, true_count, usable_records
            )),
        }
    }
}

fn calc_percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = (sorted.len() - 1) as f64 * pct;
    let lower = idx.floor() as usize;
    let upper = idx.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = idx - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_calculations() {
        let values = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert_eq!(calc_percentile(&values, 0.50), 6.0);
        assert_eq!(calc_percentile(&values, 0.0), 2.0);
        assert_eq!(calc_percentile(&values, 1.0), 10.0);
    }

    #[test]
    fn test_benchmark_in_memory() {
        let db = BenchmarkDb::open_in_memory().unwrap();
        for i in 1..=50 {
            let game = BenchmarkGameRow {
                app_id: i,
                name: format!("Game {}", i),
                release_year: Some(2024),
                is_indie: true,
                genres: vec!["Action".into(), "Indie".into()],
                screenshots_count: Some(((i % 15) + 1) as usize),
                movies_count: Some(if i % 3 == 0 { 0 } else { 1 }),
                has_trailer: Some(i % 3 != 0),
                short_desc_words: Some(25),
                short_desc_chars: Some(150),
                detailed_desc_words: Some(200),
                detailed_desc_chars: Some(1200),
                interface_languages_count: Some(if i % 2 == 0 { 5 } else { 1 }),
                full_audio_languages_count: Some(1),
                has_support_contact: Some(true),
                has_support_url: Some(true),
                has_support_email: Some(true),
                supports_windows: Some(true),
                supports_mac: Some(false),
                supports_linux: Some(i % 4 == 0),
                categories_count: Some(4),
                is_excluded: false,
                exclusion_reason: None,
            };
            db.insert_game(&game).unwrap();
        }

        let (cohort, games) = db
            .select_cohort(&["Action".into()], Some(2024), true, 30)
            .unwrap();
        assert_eq!(cohort.total_games_in_cohort, 50);

        let mut listing = StoreListing::new(
            9999,
            "My Test Game",
            crate::models::listing::ListingSource::JsonUpload,
        );
        listing.screenshots = Some(vec![
            crate::models::listing::Screenshot {
                id: 0,
                url_thumbnail: "".into(),
                url_full: "".into(),
            },
            crate::models::listing::Screenshot {
                id: 1,
                url_thumbnail: "".into(),
                url_full: "".into(),
            },
        ]);

        let results = BenchmarkDb::calculate_benchmark_metrics(&listing, &cohort, &games, 30);
        let shot_result = results
            .iter()
            .find(|r| r.metric_id == "screenshots.count")
            .unwrap();
        assert_eq!(shot_result.status, BenchmarkStatus::Calculated);
        let dist = shot_result.distribution.as_ref().unwrap();
        assert_eq!(dist.listing_value, Some(2.0));
        assert!(dist.percentile_rank.is_some());
    }
}
