use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    TitleContainsPlaytest,
    SparseMetadata,
    InvalidAppId,
    DuplicateAppId,
}

impl ExclusionReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExclusionReason::TitleContainsPlaytest => "title_contains_playtest",
            ExclusionReason::SparseMetadata => "sparse_metadata",
            ExclusionReason::InvalidAppId => "invalid_appid",
            ExclusionReason::DuplicateAppId => "duplicate_appid",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadata {
    pub source_url: String,
    pub dataset_id: String,
    pub revision: Option<String>,
    pub downloaded_at: Option<DateTime<Utc>>,
    pub ingested_at: Option<DateTime<Utc>>,
    pub source_row_count: usize,
    pub unique_appid_count: usize,
    pub duplicate_count: usize,
    pub excluded_count: usize,
    pub eligible_benchmark_count: usize,
    pub file_size_bytes: u64,
    pub license: String,
    pub attribution: String,
}

impl Default for DatasetMetadata {
    fn default() -> Self {
        Self {
            source_url: "https://huggingface.co/datasets/FronkonGames/steam-games-dataset".into(),
            dataset_id: "FronkonGames/steam-games-dataset".into(),
            revision: Some("6c21457b9ebf0a6a7f5b91e11281d65d456ea30e".into()),
            downloaded_at: None,
            ingested_at: None,
            source_row_count: 0,
            unique_appid_count: 0,
            duplicate_count: 0,
            excluded_count: 0,
            eligible_benchmark_count: 0,
            file_size_bytes: 0,
            license: "MIT".into(),
            attribution: "FronkonGames/steam-games-dataset (MIT License), created using Steam Store API & SteamSpy".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionSummary {
    pub metadata: DatasetMetadata,
    pub exclusions_by_reason: HashMap<String, usize>,
    pub games_by_year: HashMap<i32, usize>,
    pub top_genres: Vec<(String, usize)>,
    pub data_quality_notes: Vec<String>,
}
