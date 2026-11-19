use crate::models::finding::FindingCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStatus {
    Calculated,
    InsufficientSample,
    MissingListingData,
    DatasetUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDistribution {
    pub sample_size: usize,
    pub usable_records: usize,
    pub coverage_pct: f64,
    pub listing_value: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub p25: Option<f64>,
    pub median: Option<f64>,
    pub p75: Option<f64>,
    pub percentile_rank: Option<f64>,
    pub share_above: Option<f64>,
    pub share_below: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalBenchmark {
    pub sample_size: usize,
    pub usable_records: usize,
    pub coverage_pct: f64,
    pub listing_has_feature: Option<bool>,
    pub true_count: usize,
    pub false_count: usize,
    pub true_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub metric_id: String,
    pub metric_name: String,
    pub category: FindingCategory,
    pub listing_value_display: String,
    pub status: BenchmarkStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution: Option<BenchmarkDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categorical: Option<CategoricalBenchmark>,
    pub cohort_definition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohortInfo {
    pub cohort_name: String,
    pub total_games_in_cohort: usize,
    pub genres: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_year: Option<i32>,
    pub is_indie_restricted: bool,
    pub fallback_applied: bool,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMethodology {
    pub dataset_source: String,
    pub dataset_license: String,
    pub snapshot_date: String,
    pub total_dataset_games: usize,
    pub min_sample_size_threshold: usize,
    pub percentile_convention: String,
    pub disclaimer: String,
}
