use crate::models::listing::{
    ListingSource, Movie, PlatformSupport, PriceInfo, Screenshot, StoreListing,
};
use crate::parser::language::parse_steam_languages;
use chrono::Utc;
use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SteamImportError {
    #[error("Steam API returned invalid JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Steam API response indicated failure: listing not found, unpublished, or restricted (success: false)")]
    AppNotFoundOrUnavailable,
    #[error("Missing data field in Steam API response for AppID {0}")]
    MissingDataField(u32),
    #[error("Steam API response missing AppID key {0}")]
    MissingAppIdKey(u32),
}

static YEAR_RE: OnceLock<Regex> = OnceLock::new();

pub fn extract_release_year(date_str: &str) -> Option<i32> {
    let re = YEAR_RE.get_or_init(|| Regex::new(r"\b(19[89][0-9]|20[0-3][0-9])\b").unwrap());
    re.captures(date_str)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<i32>().ok())
}

pub fn parse_steam_app_details(
    app_id: u32,
    raw_json: &str,
    country: Option<String>,
    language: Option<String>,
) -> Result<StoreListing, SteamImportError> {
    let root: Value = serde_json::from_str(raw_json)?;

    // Steam response format is keyed by AppID string: { "2539430": { "success": true, "data": { ... } } }
    let app_key = app_id.to_string();
    let app_obj = root
        .get(&app_key)
        .or_else(|| {
            // Fallback: if root itself has "success" and "data"
            if root.get("success").is_some() {
                Some(&root)
            } else {
                root.as_object().and_then(|o| o.values().next())
            }
        })
        .ok_or(SteamImportError::MissingAppIdKey(app_id))?;

    let success = app_obj
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !success {
        return Err(SteamImportError::AppNotFoundOrUnavailable);
    }

    let data = app_obj
        .get("data")
        .ok_or(SteamImportError::MissingDataField(app_id))?;

    let name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled Game")
        .to_string();

    let mut listing = StoreListing::new(app_id, name, ListingSource::SteamStoreApi);
    listing.country = country;
    listing.language = language;
    listing.fetched_at = Some(Utc::now());

    // Listing type
    if let Some(t) = data.get("type").and_then(|v| v.as_str()) {
        listing.listing_type = Some(t.to_string());
        listing.track_provenance("listing_type", true, None);
    } else {
        listing.track_provenance("listing_type", false, Some("Not provided".into()));
    }

    // Required age
    if let Some(age) = data.get("required_age").and_then(|v| v.as_u64()) {
        listing.required_age = Some(age as u32);
        listing.track_provenance("required_age", true, None);
    }

    // Is free
    if let Some(is_free) = data.get("is_free").and_then(|v| v.as_bool()) {
        listing.is_free = Some(is_free);
        listing.track_provenance("is_free", true, None);
    }

    // Descriptions
    if let Some(short_desc) = data.get("short_description").and_then(|v| v.as_str()) {
        listing.short_description = Some(short_desc.to_string());
        listing.track_provenance("short_description", true, None);
    } else {
        listing.track_provenance(
            "short_description",
            false,
            Some("Missing short description".into()),
        );
    }

    if let Some(detailed_desc) = data.get("detailed_description").and_then(|v| v.as_str()) {
        listing.detailed_description = Some(detailed_desc.to_string());
        listing.track_provenance("detailed_description", true, None);
    } else {
        listing.track_provenance(
            "detailed_description",
            false,
            Some("Missing detailed description".into()),
        );
    }

    if let Some(about) = data.get("about_the_game").and_then(|v| v.as_str()) {
        listing.about_the_game = Some(about.to_string());
        listing.track_provenance("about_the_game", true, None);
    }

    // Header image
    if let Some(img) = data.get("header_image").and_then(|v| v.as_str()) {
        listing.header_image = Some(img.to_string());
        listing.track_provenance("header_image", true, None);
    }

    // Website
    if let Some(site) = data.get("website").and_then(|v| v.as_str()) {
        if !site.trim().is_empty() {
            listing.website = Some(site.trim().to_string());
            listing.track_provenance("website", true, None);
        } else {
            listing.track_provenance("website", false, Some("Empty website field".into()));
        }
    }

    // Support info
    if let Some(support) = data.get("support_info") {
        if let Some(url) = support.get("url").and_then(|v| v.as_str()) {
            if !url.trim().is_empty() {
                listing.support_url = Some(url.trim().to_string());
                listing.track_provenance("support_url", true, None);
            }
        }
        if let Some(email) = support.get("email").and_then(|v| v.as_str()) {
            if !email.trim().is_empty() {
                listing.support_email = Some(email.trim().to_string());
                listing.track_provenance("support_email", true, None);
            }
        }
    }

    // Developers and Publishers
    if let Some(devs) = data.get("developers").and_then(|v| v.as_array()) {
        listing.developers = devs
            .iter()
            .filter_map(|d| d.as_str().map(|s| s.to_string()))
            .collect();
        listing.track_provenance("developers", !listing.developers.is_empty(), None);
    }

    if let Some(pubs) = data.get("publishers").and_then(|v| v.as_array()) {
        listing.publishers = pubs
            .iter()
            .filter_map(|p| p.as_str().map(|s| s.to_string()))
            .collect();
        listing.track_provenance("publishers", !listing.publishers.is_empty(), None);
    }

    // Platforms
    if let Some(plat) = data.get("platforms") {
        let windows = plat
            .get("windows")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let mac = plat.get("mac").and_then(|v| v.as_bool()).unwrap_or(false);
        let linux = plat.get("linux").and_then(|v| v.as_bool()).unwrap_or(false);
        listing.platforms = Some(PlatformSupport {
            windows,
            mac,
            linux,
        });
        listing.track_provenance("platforms", true, None);
    } else {
        listing.track_provenance("platforms", false, Some("Platforms object missing".into()));
    }

    // Categories
    if let Some(cats) = data.get("categories").and_then(|v| v.as_array()) {
        let category_names: Vec<String> = cats
            .iter()
            .filter_map(|c| {
                c.get("description")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        listing.categories = Some(category_names);
        listing.track_provenance("categories", true, None);
    } else {
        listing.track_provenance(
            "categories",
            false,
            Some("Categories missing from response".into()),
        );
    }

    // Genres
    if let Some(genres) = data.get("genres").and_then(|v| v.as_array()) {
        let genre_names: Vec<String> = genres
            .iter()
            .filter_map(|g| {
                g.get("description")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string())
            })
            .collect();
        listing.genres = Some(genre_names);
        listing.track_provenance("genres", true, None);
    } else {
        listing.track_provenance("genres", false, Some("Genres missing from response".into()));
    }

    // Screenshots
    if let Some(screenshots) = data.get("screenshots").and_then(|v| v.as_array()) {
        let shots: Vec<Screenshot> = screenshots
            .iter()
            .filter_map(|s| {
                let id = s.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                let url_thumbnail = s
                    .get("path_thumbnail")
                    .and_then(|v| v.as_str())?
                    .to_string();
                let url_full = s.get("path_full").and_then(|v| v.as_str())?.to_string();
                Some(Screenshot {
                    id,
                    url_thumbnail,
                    url_full,
                })
            })
            .collect();
        listing.screenshots = Some(shots);
        listing.track_provenance("screenshots", true, None);
    } else {
        listing.track_provenance(
            "screenshots",
            false,
            Some("Screenshots array missing".into()),
        );
    }

    // Movies
    if let Some(movies) = data.get("movies").and_then(|v| v.as_array()) {
        let movs: Vec<Movie> = movies
            .iter()
            .map(|m| {
                let id = m.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                let name = m
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Trailer")
                    .to_string();
                let thumbnail_url = m
                    .get("thumbnail")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let mp4_url = m
                    .get("mp4")
                    .and_then(|mp4| mp4.get("max").or_else(|| mp4.get("480")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let webm_url = m
                    .get("webm")
                    .and_then(|webm| webm.get("max").or_else(|| webm.get("480")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Movie {
                    id,
                    name,
                    thumbnail_url,
                    mp4_url,
                    webm_url,
                }
            })
            .collect();
        listing.movies = Some(movs);
        listing.track_provenance("movies", true, None);
    } else {
        listing.track_provenance("movies", false, Some("Movies array missing".into()));
    }

    // Languages
    if let Some(lang_str) = data.get("supported_languages").and_then(|v| v.as_str()) {
        let parsed = parse_steam_languages(lang_str);
        listing.supported_languages = Some(parsed.interface_languages);
        listing.full_audio_languages = parsed.full_audio_languages;
        listing.track_provenance("supported_languages", true, None);
        listing.track_provenance(
            "full_audio_languages",
            listing.full_audio_languages.is_some(),
            if listing.full_audio_languages.is_none() {
                Some(
                    "Full audio languages not explicitly indicated in Steam language string".into(),
                )
            } else {
                None
            },
        );
    } else {
        listing.track_provenance(
            "supported_languages",
            false,
            Some("Missing supported_languages field".into()),
        );
        listing.track_provenance(
            "full_audio_languages",
            false,
            Some("Missing supported_languages field".into()),
        );
    }

    // Release date
    if let Some(rel) = data.get("release_date") {
        if let Some(coming) = rel.get("coming_soon").and_then(|v| v.as_bool()) {
            listing.coming_soon = Some(coming);
        }
        if let Some(date_str) = rel.get("date").and_then(|v| v.as_str()) {
            listing.release_date = Some(date_str.to_string());
            listing.release_year = extract_release_year(date_str);
            listing.track_provenance("release_date", true, None);
        }
    }

    // Price overview
    if let Some(po) = data.get("price_overview") {
        let currency = po
            .get("currency")
            .and_then(|v| v.as_str())
            .unwrap_or("USD")
            .to_string();
        let initial_cents = po.get("initial").and_then(|v| v.as_u64()).unwrap_or(0);
        let final_cents = po.get("final").and_then(|v| v.as_u64()).unwrap_or(0);
        let discount_percent = po
            .get("discount_percent")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let formatted_initial = po
            .get("initial_formatted")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let formatted_final = po
            .get("final_formatted")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        listing.price = Some(PriceInfo {
            currency,
            initial_cents,
            final_cents,
            discount_percent,
            formatted_initial,
            formatted_final,
        });
        listing.track_provenance("price", true, None);
    }

    Ok(listing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_release_year() {
        assert_eq!(extract_release_year("15 Aug, 2024"), Some(2024));
        assert_eq!(extract_release_year("Oct 2023"), Some(2023));
        assert_eq!(extract_release_year("2025"), Some(2025));
        assert_eq!(extract_release_year("Coming soon"), None);
    }

    #[test]
    fn test_parse_steam_app_details_success() {
        let json = r#"{
            "2539430": {
                "success": true,
                "data": {
                    "type": "game",
                    "name": "Black Dragon Mage",
                    "steam_appid": 2539430,
                    "short_description": "A dark retro roguelite bullet hell.",
                    "detailed_description": "<p>Fight hordes of enemies in this dark fantasy world!</p>",
                    "header_image": "https://cdn.akamai.steamstatic.com/steam/apps/2539430/header.jpg",
                    "developers": ["Morbidware"],
                    "publishers": ["Morbidware"],
                    "platforms": { "windows": true, "mac": false, "linux": false },
                    "categories": [{"id": 2, "description": "Single-player"}],
                    "genres": [{"id": "1", "description": "Action"}, {"id": "23", "description": "Indie"}],
                    "screenshots": [
                        {"id": 0, "path_thumbnail": "https://thumb1.jpg", "path_full": "https://full1.jpg"},
                        {"id": 1, "path_thumbnail": "https://thumb2.jpg", "path_full": "https://full2.jpg"}
                    ],
                    "movies": [
                        {"id": 1, "name": "Trailer", "thumbnail": "https://movie.jpg"}
                    ],
                    "supported_languages": "English<strong>*</strong>, German<br><strong>*</strong>languages with full audio support",
                    "release_date": { "coming_soon": false, "date": "15 Aug, 2024" }
                }
            }
        }"#;

        let listing =
            parse_steam_app_details(2539430, json, Some("us".into()), Some("english".into()))
                .unwrap();
        assert_eq!(listing.name, "Black Dragon Mage");
        assert_eq!(listing.release_year, Some(2024));
        assert_eq!(listing.screenshot_count(), Some(2));
        assert_eq!(listing.movie_count(), Some(1));
        assert!(listing.is_indie());
        assert_eq!(listing.supported_language_count(), Some(2));
        assert_eq!(listing.full_audio_language_count(), Some(1));
    }

    #[test]
    fn test_parse_steam_app_details_failure() {
        let json = r#"{"9999999": {"success": false}}"#;
        let res = parse_steam_app_details(9999999, json, None, None);
        assert!(res.is_err());
    }
}
