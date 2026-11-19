use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AppIdParseError {
    #[error("Empty input provided")]
    EmptyInput,
    #[error("Invalid AppID format: {0}. Expected a positive integer or Steam store URL")]
    InvalidFormat(String),
    #[error("AppID must be a positive integer greater than zero")]
    ZeroAppId,
}

static STEAM_URL_RE: OnceLock<Regex> = OnceLock::new();

pub fn parse_app_id_or_url(input: &str) -> Result<u32, AppIdParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppIdParseError::EmptyInput);
    }

    // Direct numeric check
    if let Ok(app_id) = trimmed.parse::<u32>() {
        if app_id == 0 {
            return Err(AppIdParseError::ZeroAppId);
        }
        return Ok(app_id);
    }

    // URL regex check
    let re = STEAM_URL_RE.get_or_init(|| {
        Regex::new(r"(?:store\.steampowered\.com|steamcommunity\.com)/app/([0-9]+)").unwrap()
    });

    if let Some(captures) = re.captures(trimmed) {
        if let Some(matched) = captures.get(1) {
            if let Ok(app_id) = matched.as_str().parse::<u32>() {
                if app_id == 0 {
                    return Err(AppIdParseError::ZeroAppId);
                }
                return Ok(app_id);
            }
        }
    }

    Err(AppIdParseError::InvalidFormat(input.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_numeric() {
        assert_eq!(parse_app_id_or_url("2539430"), Ok(2539430));
        assert_eq!(parse_app_id_or_url("  10  "), Ok(10));
    }

    #[test]
    fn test_parse_steam_urls() {
        assert_eq!(
            parse_app_id_or_url("https://store.steampowered.com/app/2539430/"),
            Ok(2539430)
        );
        assert_eq!(
            parse_app_id_or_url(
                "https://store.steampowered.com/app/2539430/Black_Dragon_Mage_Playtest/"
            ),
            Ok(2539430)
        );
        assert_eq!(
            parse_app_id_or_url("store.steampowered.com/app/730"),
            Ok(730)
        );
    }

    #[test]
    fn test_invalid_inputs() {
        assert_eq!(parse_app_id_or_url(""), Err(AppIdParseError::EmptyInput));
        assert_eq!(parse_app_id_or_url("0"), Err(AppIdParseError::ZeroAppId));
        assert_eq!(
            parse_app_id_or_url("invalid_url"),
            Err(AppIdParseError::InvalidFormat("invalid_url".to_string()))
        );
    }
}
