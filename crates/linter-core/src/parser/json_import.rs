use crate::models::listing::{ListingSource, Movie, PlatformSupport, Screenshot, StoreListing};
use crate::parser::language::parse_steam_languages;
use crate::parser::steam_import::{extract_release_year, parse_steam_app_details};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JsonImportError {
    #[error("File exceeds maximum allowed upload size ({0} bytes)")]
    PayloadTooLarge(usize),
    #[error("Failed to parse JSON: {0}")]
    MalformedJson(#[from] serde_json::Error),
    #[error("Unrecognized JSON format: could not identify Steam Store Linter schema, Steam appdetails, or dataset record format")]
    UnrecognizedFormat,
    #[error("Missing required field '{0}'")]
    MissingRequiredField(&'static str),
}

pub const MAX_JSON_UPLOAD_BYTES: usize = 10 * 1024 * 1024; // 10 MB

pub fn import_listing_json(raw_json: &str) -> Result<StoreListing, JsonImportError> {
    if raw_json.len() > MAX_JSON_UPLOAD_BYTES {
        return Err(JsonImportError::PayloadTooLarge(MAX_JSON_UPLOAD_BYTES));
    }

    let root: Value = serde_json::from_str(raw_json)?;

    // Case 1: Already normalized StoreListing format
    if let Ok(listing) = serde_json::from_value::<StoreListing>(root.clone()) {
        if listing.app_id > 0 && !listing.name.is_empty() {
            let mut l = listing;
            l.source = ListingSource::JsonUpload;
            return Ok(l);
        }
    }

    // Case 2: Standard Steam API appdetails envelope: { "<appid>": { "success": true, "data": { ... } } }
    if let Some(obj) = root.as_object() {
        for (key, val) in obj {
            if let Ok(app_id) = key.parse::<u32>() {
                if val.get("success").is_some() || val.get("data").is_some() {
                    if let Ok(listing) = parse_steam_app_details(app_id, raw_json, None, None) {
                        let mut l = listing;
                        l.source = ListingSource::JsonUpload;
                        return Ok(l);
                    }
                }
            }
        }
    }

    // Case 3: Raw Steam "data" object alone or dataset row format
    if let Some(obj) = root.as_object() {
        let app_id = obj
            .get("app_id")
            .or_else(|| obj.get("appID"))
            .or_else(|| obj.get("steam_appid"))
            .and_then(|v| v.as_u64())
            .map(|id| id as u32)
            .unwrap_or(0);

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        if app_id > 0 && !name.is_empty() {
            let mut listing = StoreListing::new(app_id, name, ListingSource::JsonUpload);

            if let Some(short) = obj.get("short_description").and_then(|v| v.as_str()) {
                listing.short_description = Some(short.to_string());
                listing.track_provenance("short_description", true, None);
            }
            if let Some(detailed) = obj.get("detailed_description").and_then(|v| v.as_str()) {
                listing.detailed_description = Some(detailed.to_string());
                listing.track_provenance("detailed_description", true, None);
            }
            if let Some(about) = obj.get("about_the_game").and_then(|v| v.as_str()) {
                listing.about_the_game = Some(about.to_string());
            }

            if let Some(rel) = obj.get("release_date").and_then(|v| v.as_str()) {
                listing.release_date = Some(rel.to_string());
                listing.release_year = extract_release_year(rel);
                listing.track_provenance("release_date", true, None);
            }

            if let Some(img) = obj.get("header_image").and_then(|v| v.as_str()) {
                listing.header_image = Some(img.to_string());
                listing.track_provenance("header_image", true, None);
            }

            // Developers and Publishers
            if let Some(devs) = obj.get("developers").and_then(|v| v.as_array()) {
                listing.developers = devs
                    .iter()
                    .filter_map(|d| d.as_str().map(|s| s.to_string()))
                    .collect();
                listing.track_provenance("developers", !listing.developers.is_empty(), None);
            }
            if let Some(pubs) = obj.get("publishers").and_then(|v| v.as_array()) {
                listing.publishers = pubs
                    .iter()
                    .filter_map(|p| p.as_str().map(|s| s.to_string()))
                    .collect();
                listing.track_provenance("publishers", !listing.publishers.is_empty(), None);
            }

            // Platforms
            let mut windows = obj
                .get("windows")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mut mac = obj.get("mac").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut linux = obj.get("linux").and_then(|v| v.as_bool()).unwrap_or(false);

            if let Some(plat) = obj.get("platforms").and_then(|v| v.as_object()) {
                if let Some(w) = plat.get("windows").and_then(|v| v.as_bool()) {
                    windows = w;
                }
                if let Some(m) = plat.get("mac").and_then(|v| v.as_bool()) {
                    mac = m;
                }
                if let Some(l) = plat.get("linux").and_then(|v| v.as_bool()) {
                    linux = l;
                }
            }

            if obj.contains_key("windows") || obj.contains_key("platforms") {
                listing.platforms = Some(PlatformSupport {
                    windows,
                    mac,
                    linux,
                });
                listing.track_provenance("platforms", true, None);
            }

            // Categories
            if let Some(cats) = obj.get("categories").and_then(|v| v.as_array()) {
                listing.categories = Some(
                    cats.iter()
                        .filter_map(|c| c.as_str().map(|s| s.to_string()))
                        .collect(),
                );
                listing.track_provenance("categories", true, None);
            }

            // Genres
            if let Some(genres) = obj.get("genres").and_then(|v| v.as_array()) {
                listing.genres = Some(
                    genres
                        .iter()
                        .filter_map(|g| g.as_str().map(|s| s.to_string()))
                        .collect(),
                );
                listing.track_provenance("genres", true, None);
            }

            // Screenshots
            if let Some(shots) = obj.get("screenshots") {
                if let Some(arr) = shots.as_array() {
                    let mut screenshot_list = Vec::new();
                    for (i, item) in arr.iter().enumerate() {
                        if let Some(url) = item.as_str() {
                            screenshot_list.push(Screenshot {
                                id: i as u64,
                                url_thumbnail: url.to_string(),
                                url_full: url.to_string(),
                            });
                        } else if let Some(o) = item.as_object() {
                            let url_t = o
                                .get("path_thumbnail")
                                .or_else(|| o.get("url_thumbnail"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let url_f = o
                                .get("path_full")
                                .or_else(|| o.get("url_full"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            screenshot_list.push(Screenshot {
                                id: i as u64,
                                url_thumbnail: url_t,
                                url_full: url_f,
                            });
                        }
                    }
                    listing.screenshots = Some(screenshot_list);
                    listing.track_provenance("screenshots", true, None);
                }
            }

            // Movies
            if let Some(movies) = obj.get("movies") {
                if let Some(arr) = movies.as_array() {
                    let mut movie_list = Vec::new();
                    for (i, item) in arr.iter().enumerate() {
                        if let Some(name) = item.as_str() {
                            movie_list.push(Movie {
                                id: i as u64,
                                name: name.to_string(),
                                thumbnail_url: None,
                                mp4_url: None,
                                webm_url: None,
                            });
                        } else if let Some(o) = item.as_object() {
                            let name = o
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Trailer")
                                .to_string();
                            movie_list.push(Movie {
                                id: i as u64,
                                name,
                                thumbnail_url: None,
                                mp4_url: None,
                                webm_url: None,
                            });
                        }
                    }
                    listing.movies = Some(movie_list);
                    listing.track_provenance("movies", true, None);
                }
            }

            // Languages
            if let Some(lang_val) = obj.get("supported_languages") {
                if let Some(s) = lang_val.as_str() {
                    let parsed = parse_steam_languages(s);
                    listing.supported_languages = Some(parsed.interface_languages);
                    listing.full_audio_languages = parsed.full_audio_languages;
                } else if let Some(arr) = lang_val.as_array() {
                    listing.supported_languages = Some(
                        arr.iter()
                            .filter_map(|l| l.as_str().map(|s| s.to_string()))
                            .collect(),
                    );
                }
                listing.track_provenance("supported_languages", true, None);
            }

            if let Some(audio_val) = obj.get("full_audio_languages") {
                if let Some(arr) = audio_val.as_array() {
                    listing.full_audio_languages = Some(
                        arr.iter()
                            .filter_map(|l| l.as_str().map(|s| s.to_string()))
                            .collect(),
                    );
                    listing.track_provenance("full_audio_languages", true, None);
                }
            }

            // Support info
            if let Some(url) = obj.get("support_url").and_then(|v| v.as_str()) {
                if !url.trim().is_empty() {
                    listing.support_url = Some(url.to_string());
                }
            }
            if let Some(email) = obj.get("support_email").and_then(|v| v.as_str()) {
                if !email.trim().is_empty() {
                    listing.support_email = Some(email.to_string());
                }
            }
            if let Some(site) = obj.get("website").and_then(|v| v.as_str()) {
                if !site.trim().is_empty() {
                    listing.website = Some(site.to_string());
                }
            }

            return Ok(listing);
        }
    }

    Err(JsonImportError::UnrecognizedFormat)
}

/// Returns a well-formatted example JSON string suitable for download by users.
pub fn get_example_listing_json() -> &'static str {
    r#"{
  "app_id": 2539430,
  "name": "Dragon Forge: Chronicles",
  "short_description": "Forge legendary gear, delve into ancient dungeons, and tame dragons in this indie action-RPG crafting adventure.",
  "detailed_description": "<h2>Enter the Realm of the Dragon Forge</h2><p>Embark on an epic journey through procedural dungeons, gathering ancient ores and mastering blacksmithing techniques forgotten for centuries.</p><h3>Key Features</h3><ul><li>Deep crafting system with over 300 craftable weapons and armor sets.</li><li>Challenging dungeon crawler combat with responsive dodge and parry mechanics.</li><li>Co-op multiplayer support for up to 4 players.</li></ul>",
  "developers": ["Emberstone Studios"],
  "publishers": ["Emberstone Studios"],
  "header_image": "https://example.com/capsule_header.jpg",
  "release_date": "24 Oct, 2024",
  "release_year": 2024,
  "genres": ["Action", "Adventure", "Indie", "RPG"],
  "categories": ["Single-player", "Online Co-op", "Steam Achievements", "Full controller support"],
  "tags": ["Indie", "Crafting", "Action RPG", "Dungeon Crawler"],
  "supported_languages": ["English", "French", "German", "Spanish - Spain", "Japanese", "Simplified Chinese"],
  "full_audio_languages": ["English"],
  "platforms": {
    "windows": true,
    "mac": true,
    "linux": true
  },
  "screenshots": [
    {
      "id": 1,
      "url_thumbnail": "https://example.com/screens/thumb1.jpg",
      "url_full": "https://example.com/screens/full1.jpg"
    },
    {
      "id": 2,
      "url_thumbnail": "https://example.com/screens/thumb2.jpg",
      "url_full": "https://example.com/screens/full2.jpg"
    },
    {
      "id": 3,
      "url_thumbnail": "https://example.com/screens/thumb3.jpg",
      "url_full": "https://example.com/screens/full3.jpg"
    },
    {
      "id": 4,
      "url_thumbnail": "https://example.com/screens/thumb4.jpg",
      "url_full": "https://example.com/screens/full4.jpg"
    },
    {
      "id": 5,
      "url_thumbnail": "https://example.com/screens/thumb5.jpg",
      "url_full": "https://example.com/screens/full5.jpg"
    },
    {
      "id": 6,
      "url_thumbnail": "https://example.com/screens/thumb6.jpg",
      "url_full": "https://example.com/screens/full6.jpg"
    }
  ],
  "movies": [
    {
      "id": 1,
      "name": "Official Gameplay Reveal Trailer"
    }
  ],
  "website": "https://emberstone.example.com/dragonforge",
  "support_url": "https://emberstone.example.com/support",
  "support_email": "support@emberstone.example.com"
}"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_example_json() {
        let example = get_example_listing_json();
        let listing = import_listing_json(example).unwrap();
        assert_eq!(listing.app_id, 2539430);
        assert_eq!(listing.name, "Dragon Forge: Chronicles");
        assert_eq!(listing.screenshot_count(), Some(6));
        assert_eq!(listing.movie_count(), Some(1));
        assert!(listing.is_indie());
    }
}
