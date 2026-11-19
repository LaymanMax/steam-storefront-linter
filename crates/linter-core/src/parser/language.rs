use crate::parser::html::strip_html_tags;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedLanguages {
    pub interface_languages: Vec<String>,
    pub full_audio_languages: Option<Vec<String>>,
}

/// Parses a Steam store API language string or raw text representation.
/// E.g.: "English<strong>*</strong>, French, German, Spanish - Spain<strong>*</strong><br><strong>*</strong>languages with full audio support"
pub fn parse_steam_languages(raw: &str) -> ParsedLanguages {
    let raw_lower = raw.to_lowercase();
    let has_audio_footnote = raw_lower.contains("languages with full audio support")
        || raw.contains("<strong>*</strong>")
        || raw.contains("<b>*</b>")
        || raw.contains('*');

    // Remove footnote explanatory text at the end: "<br><strong>*</strong>languages with full audio support"
    let clean_text = if let Some(idx) = raw_lower.find("<br>") {
        &raw[..idx]
    } else if let Some(idx) = raw_lower.find("languages with full audio support") {
        &raw[..idx]
    } else {
        raw
    };

    let mut interface = Vec::new();
    let mut audio = Vec::new();

    // Items separated by comma or semicolon
    for item in clean_text.split([',', ';']) {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if item has audio indicator (*) before stripping tags
        let has_star = trimmed.contains('*');

        // Strip HTML tags and remove asterisk
        let mut lang_name = strip_html_tags(trimmed);
        lang_name = lang_name.replace('*', "").trim().to_string();

        if lang_name.is_empty() || lang_name.eq_ignore_ascii_case("all") {
            continue;
        }

        // Avoid duplicates while preserving order
        if !interface
            .iter()
            .any(|l: &String| l.eq_ignore_ascii_case(&lang_name))
        {
            interface.push(lang_name.clone());
        }

        if has_star
            && !audio
                .iter()
                .any(|l: &String| l.eq_ignore_ascii_case(&lang_name))
        {
            audio.push(lang_name);
        }
    }

    let full_audio = if has_audio_footnote || !audio.is_empty() {
        Some(audio)
    } else {
        // If there was no mention of full audio at all in the text, it is unknown
        None
    };

    ParsedLanguages {
        interface_languages: interface,
        full_audio_languages: full_audio,
    }
}

/// Normalizes a list of language names from structured JSON or arrays.
pub fn normalize_language_list(langs: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for l in langs {
        let clean = strip_html_tags(l).replace('*', "").trim().to_string();
        if !clean.is_empty()
            && !result
                .iter()
                .any(|x: &String| x.eq_ignore_ascii_case(&clean))
        {
            result.push(clean);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_steam_languages_with_audio_footnote() {
        let raw = "English<strong>*</strong>, French, German, Spanish - Spain<strong>*</strong>, Japanese<br><strong>*</strong>languages with full audio support";
        let parsed = parse_steam_languages(raw);

        assert_eq!(parsed.interface_languages.len(), 5);
        assert_eq!(parsed.interface_languages[0], "English");
        assert_eq!(parsed.interface_languages[1], "French");
        assert_eq!(parsed.interface_languages[2], "German");
        assert_eq!(parsed.interface_languages[3], "Spanish - Spain");
        assert_eq!(parsed.interface_languages[4], "Japanese");

        let audio = parsed.full_audio_languages.expect("should have audio");
        assert_eq!(audio.len(), 2);
        assert_eq!(audio[0], "English");
        assert_eq!(audio[1], "Spanish - Spain");
    }

    #[test]
    fn test_parse_steam_languages_plain() {
        let raw = "English, Simplified Chinese, Traditional Chinese";
        let parsed = parse_steam_languages(raw);
        assert_eq!(parsed.interface_languages.len(), 3);
        assert!(
            parsed.full_audio_languages.is_none(),
            "no audio info should be None, not 0"
        );
    }
}
