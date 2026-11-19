use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingSource {
    SteamStoreApi,
    JsonUpload,
    BenchmarkDataset,
    SampleFixture,
}

impl ListingSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            ListingSource::SteamStoreApi => "steam_store_api",
            ListingSource::JsonUpload => "json_upload",
            ListingSource::BenchmarkDataset => "benchmark_dataset",
            ListingSource::SampleFixture => "sample_fixture",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldProvenance {
    pub field: String,
    pub known: bool,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformSupport {
    pub windows: bool,
    pub mac: bool,
    pub linux: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    pub id: u64,
    pub url_thumbnail: String,
    pub url_full: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movie {
    pub id: u64,
    pub name: String,
    pub thumbnail_url: Option<String>,
    pub mp4_url: Option<String>,
    pub webm_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInfo {
    pub currency: String,
    pub initial_cents: u64,
    pub final_cents: u64,
    pub discount_percent: u32,
    pub formatted_initial: String,
    pub formatted_final: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreListing {
    pub app_id: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coming_soon: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_free: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<PriceInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_age: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about_the_game: Option<String>,

    pub developers: Vec<String>,
    pub publishers: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_languages: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_audio_languages: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshots: Option<Vec<Screenshot>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub movies: Option<Vec<Movie>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_email: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<PlatformSupport>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub achievements_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_url: Option<String>,

    pub source: ListingSource,
    pub provenance: HashMap<String, FieldProvenance>,
    pub fetched_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl StoreListing {
    pub fn new(app_id: u32, name: impl Into<String>, source: ListingSource) -> Self {
        Self {
            app_id,
            name: name.into(),
            listing_type: None,
            release_date: None,
            release_year: None,
            coming_soon: None,
            is_free: None,
            price: None,
            required_age: None,
            short_description: None,
            detailed_description: None,
            about_the_game: None,
            developers: Vec::new(),
            publishers: Vec::new(),
            supported_languages: None,
            full_audio_languages: None,
            screenshots: None,
            movies: None,
            website: None,
            support_url: None,
            support_email: None,
            platforms: None,
            categories: None,
            genres: None,
            tags: None,
            achievements_count: None,
            header_image: None,
            store_url: Some(format!("https://store.steampowered.com/app/{}/", app_id)),
            source,
            provenance: HashMap::new(),
            fetched_at: Some(Utc::now()),
            country: None,
            language: None,
        }
    }

    pub fn track_provenance(&mut self, field: &str, known: bool, notes: Option<String>) {
        self.provenance.insert(
            field.to_string(),
            FieldProvenance {
                field: field.to_string(),
                known,
                source: self.source.as_str().to_string(),
                notes,
            },
        );
    }

    pub fn is_indie(&self) -> bool {
        if let Some(genres) = &self.genres {
            if genres.iter().any(|g| g.eq_ignore_ascii_case("indie")) {
                return true;
            }
        }
        if let Some(tags) = &self.tags {
            if tags.iter().any(|t| t.eq_ignore_ascii_case("indie")) {
                return true;
            }
        }
        false
    }

    pub fn screenshot_count(&self) -> Option<usize> {
        self.screenshots.as_ref().map(|s| s.len())
    }

    pub fn movie_count(&self) -> Option<usize> {
        self.movies.as_ref().map(|m| m.len())
    }

    pub fn supported_language_count(&self) -> Option<usize> {
        self.supported_languages.as_ref().map(|l| l.len())
    }

    pub fn full_audio_language_count(&self) -> Option<usize> {
        self.full_audio_languages.as_ref().map(|l| l.len())
    }

    pub fn category_count(&self) -> Option<usize> {
        self.categories.as_ref().map(|c| c.len())
    }
}
