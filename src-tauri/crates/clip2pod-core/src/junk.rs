/// Default junk phrases from the spec; matching is case-insensitive substring.
pub const DEFAULT_JUNK_PHRASES: [&str; 12] = [
    "credit:",
    "getty images",
    "http",
    "listen to article",
    "min read",
    "read more",
    "read this article for free",
    "related links",
    "related stories",
    "unsplash",
    "view image in full size",
    "view original",
];

pub fn default_phrases() -> Vec<String> {
    DEFAULT_JUNK_PHRASES.iter().map(|s| s.to_string()).collect()
}

/// True when a custom list is exactly the defaults (so config can store a
/// "using defaults" marker instead of a redundant copy).
pub fn is_default(phrases: &[String]) -> bool {
    let mut norm: Vec<String> = phrases.iter().map(|s| s.trim().to_lowercase()).collect();
    norm.sort();
    let mut defaults = default_phrases();
    defaults.sort();
    norm == defaults
}

/// A junk phrase found in the text. Offsets are character (not byte)
/// positions within the matched line.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct JunkMatch {
    pub line_idx: usize,
    pub match_start: usize,
    pub match_end: usize,
    pub phrase: String,
}

/// Find the first junk phrase at or after `from_line`. No wrap-around:
/// returns None past the last match; the caller decides whether to restart
/// from line 0.
pub fn find_next(text: &str, from_line: usize, phrases: &[String]) -> Option<JunkMatch> {
    let needles: Vec<String> = phrases
        .iter()
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect();
    for (line_idx, line) in text.lines().enumerate().skip(from_line) {
        let lower: Vec<char> = line.to_lowercase().chars().collect();
        let mut best: Option<(usize, &String)> = None;
        for needle in &needles {
            let n: Vec<char> = needle.chars().collect();
            if n.is_empty() || n.len() > lower.len() {
                continue;
            }
            if let Some(pos) = lower.windows(n.len()).position(|w| w == n.as_slice()) {
                if best.is_none_or(|(b, _)| pos < b) {
                    best = Some((pos, needle));
                }
            }
        }
        if let Some((pos, needle)) = best {
            return Some(JunkMatch {
                line_idx,
                match_start: pos,
                match_end: pos + needle.chars().count(),
                phrase: needle.clone(),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_default_ignores_order_and_case() {
        assert!(is_default(&default_phrases()));
        let mut shuffled = default_phrases();
        shuffled.reverse();
        assert!(is_default(&shuffled));
        let upper: Vec<String> = default_phrases().iter().map(|s| s.to_uppercase()).collect();
        assert!(is_default(&upper));
        assert!(!is_default(&["custom".to_string()]));
        let mut extra = default_phrases();
        extra.push("extra".to_string());
        assert!(!is_default(&extra));
    }

    #[test]
    fn finds_first_match_case_insensitively() {
        let text = "Headline\nPhoto: GETTY IMAGES\nBody text";
        let m = find_next(text, 0, &default_phrases()).unwrap();
        assert_eq!(m.line_idx, 1);
        assert_eq!(m.phrase, "getty images");
        assert_eq!(m.match_start, 7);
        assert_eq!(m.match_end, 19);
    }

    #[test]
    fn respects_from_line_and_does_not_wrap() {
        let text = "5 min read\nBody\nRead more here";
        let first = find_next(text, 0, &default_phrases()).unwrap();
        assert_eq!(first.line_idx, 0);
        let second = find_next(text, 1, &default_phrases()).unwrap();
        assert_eq!(second.line_idx, 2);
        assert_eq!(second.phrase, "read more");
        assert!(find_next(text, 3, &default_phrases()).is_none());
    }

    #[test]
    fn returns_none_when_clean() {
        assert!(find_next("Just a normal article line", 0, &default_phrases()).is_none());
    }

    #[test]
    fn match_offsets_are_char_based() {
        // é is 2 bytes but 1 char; offsets must count chars.
        let text = "Café credit: someone";
        let m = find_next(text, 0, &default_phrases()).unwrap();
        assert_eq!(m.match_start, 5);
        assert_eq!(m.match_end, 12);
    }
}
