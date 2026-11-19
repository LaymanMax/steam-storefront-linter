use regex::Regex;
use std::sync::OnceLock;

static TAG_RE: OnceLock<Regex> = OnceLock::new();
static BR_P_RE: OnceLock<Regex> = OnceLock::new();
static MULTI_SPACE_RE: OnceLock<Regex> = OnceLock::new();

pub fn strip_html_tags(input: &str) -> String {
    let br_p_re = BR_P_RE.get_or_init(|| {
        Regex::new(r"(?i)<(br|/p|/div|li|/li|ul|/ul|ol|/ol|/h[1-6])\s*/?>").unwrap()
    });
    let tag_re = TAG_RE.get_or_init(|| Regex::new(r"<[^>]*>").unwrap());
    let multi_space_re = MULTI_SPACE_RE.get_or_init(|| Regex::new(r"\s+").unwrap());

    // Replace break/block tags with space
    let with_spaces = br_p_re.replace_all(input, " ");
    // Strip remaining tags
    let stripped = tag_re.replace_all(&with_spaces, "");
    // Decode entities
    let decoded = decode_html_entities(&stripped);
    // Normalize spaces
    multi_space_re.replace_all(&decoded, " ").trim().to_string()
}

pub fn decode_html_entities(input: &str) -> String {
    let mut result = input.to_string();
    result = result.replace("&nbsp;", " ");
    result = result.replace("&quot;", "\"");
    result = result.replace("&apos;", "'");
    result = result.replace("&#39;", "'");
    result = result.replace("&amp;", "&");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&trade;", "™");
    result = result.replace("&reg;", "®");
    result = result.replace("&copy;", "©");
    result = result.replace("&mdash;", "—");
    result = result.replace("&ndash;", "–");
    result = result.replace("&bull;", "•");
    result
}

pub fn count_words(text: &str) -> usize {
    text.split_whitespace().filter(|w| !w.is_empty()).count()
}

pub fn count_characters(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        let html = "<p>Welcome to <b>The Dungeon</b>!</p><br>Features:<ul><li>Monsters &amp; Loot</li></ul>";
        let clean = strip_html_tags(html);
        assert_eq!(clean, "Welcome to The Dungeon! Features: Monsters & Loot");
        assert_eq!(count_words(&clean), 8);
    }

    #[test]
    fn test_decode_entities() {
        let text = "Rock &amp; Roll &#39;Legends&#39; &quot;Deluxe&quot;";
        let decoded = decode_html_entities(text);
        assert_eq!(decoded, "Rock & Roll 'Legends' \"Deluxe\"");
    }
}
