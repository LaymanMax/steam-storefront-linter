use crate::models::listing::StoreListing;
use crate::parser::steam_import::{parse_steam_app_details, SteamImportError};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum SteamClientError {
    #[error("Invalid AppID: {0}. AppID must be a positive integer.")]
    InvalidAppId(u32),
    #[error("Steam API rate limit reached (HTTP 429). Please wait a moment before trying again.")]
    RateLimited,
    #[error("Steam API request timed out after {0:?}.")]
    Timeout(Duration),
    #[error("Steam server error (HTTP {0}). Steam may be experiencing downtime.")]
    ServerError(u16),
    #[error("Network connection error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Listing not found or unavailable on Steam: {0}")]
    ListingUnavailable(String),
    #[error("Failed to parse Steam response: {0}")]
    ParseError(#[from] SteamImportError),
}

#[derive(Clone)]
struct CachedResponse {
    json: String,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct SteamClient {
    client: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, CachedResponse>>>,
    cache_ttl: Duration,
    request_timeout: Duration,
}

impl Default for SteamClient {
    fn default() -> Self {
        Self::new(Duration::from_secs(10), Duration::from_secs(300))
    }
}

impl SteamClient {
    pub fn new(request_timeout: Duration, cache_ttl: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(request_timeout)
            .user_agent("SteamStorefrontLinter/0.1.0 (https://github.com/LaymanMax/steam-storefront-linter)")
            .build()
            .unwrap_or_default();

        Self {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_ttl,
            request_timeout,
        }
    }

    pub async fn fetch_listing(
        &self,
        app_id: u32,
        country: Option<&str>,
        language: Option<&str>,
    ) -> Result<StoreListing, SteamClientError> {
        if app_id == 0 {
            return Err(SteamClientError::InvalidAppId(app_id));
        }

        let cc = country.unwrap_or("us");
        let l = language.unwrap_or("english");
        let cache_key = format!("{}:{}:{}", app_id, cc, l);

        // Check in-memory cache
        {
            let cache = self.cache.lock().await;
            if let Some(entry) = cache.get(&cache_key) {
                if entry.fetched_at.elapsed() < self.cache_ttl {
                    info!("Cache hit for Steam listing {}", cache_key);
                    return parse_steam_app_details(
                        app_id,
                        &entry.json,
                        Some(cc.to_string()),
                        Some(l.to_string()),
                    )
                    .map_err(SteamClientError::ParseError);
                }
            }
        }

        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={}&cc={}&l={}",
            app_id, cc, l
        );

        let mut attempts = 0;
        let max_attempts = 3;
        let mut last_error = None;

        while attempts < max_attempts {
            attempts += 1;
            info!(
                "Fetching Steam listing for AppID {} (attempt {}/{})",
                app_id, attempts, max_attempts
            );

            let res = self.client.get(&url).send().await;

            match res {
                Ok(resp) => {
                    let status = resp.status();
                    if status.as_u16() == 429 {
                        warn!(
                            "Received HTTP 429 rate limit from Steam API on AppID {}",
                            app_id
                        );
                        return Err(SteamClientError::RateLimited);
                    }

                    if status.is_server_error() {
                        warn!(
                            "Steam API returned server error {} on attempt {}",
                            status, attempts
                        );
                        last_error = Some(SteamClientError::ServerError(status.as_u16()));
                        tokio::time::sleep(Duration::from_millis(500 * attempts)).await;
                        continue;
                    }

                    if !status.is_success() {
                        return Err(SteamClientError::ListingUnavailable(format!(
                            "Steam returned HTTP {}",
                            status
                        )));
                    }

                    let text = resp.text().await?;

                    // Parse listing
                    let listing = parse_steam_app_details(
                        app_id,
                        &text,
                        Some(cc.to_string()),
                        Some(l.to_string()),
                    )
                    .map_err(|err| match err {
                        SteamImportError::AppNotFoundOrUnavailable => {
                            SteamClientError::ListingUnavailable(format!(
                                "Listing for AppID {} is either unreleased, region-restricted, or removed from Steam",
                                app_id
                            ))
                        }
                        other => SteamClientError::ParseError(other),
                    })?;

                    // Save to cache
                    {
                        let mut cache = self.cache.lock().await;
                        cache.insert(
                            cache_key,
                            CachedResponse {
                                json: text,
                                fetched_at: Instant::now(),
                            },
                        );
                    }

                    return Ok(listing);
                }
                Err(err) => {
                    if err.is_timeout() {
                        last_error = Some(SteamClientError::Timeout(self.request_timeout));
                    } else {
                        last_error = Some(SteamClientError::Network(err));
                    }
                    tokio::time::sleep(Duration::from_millis(500 * attempts)).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            SteamClientError::ListingUnavailable(format!(
                "Failed to fetch listing for AppID {} after {} attempts",
                app_id, max_attempts
            ))
        }))
    }
}
