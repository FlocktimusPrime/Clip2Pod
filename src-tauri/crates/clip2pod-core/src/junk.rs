/// Default junk phrases from the spec. Matching is case-insensitive substring;
/// a `*` in a phrase matches any run of characters (segments must appear in
/// order on the line).
pub const DEFAULT_JUNK_PHRASES: [&str; 17] = [
    "credit:",
    "getty images",
    "http",
    "image by author",
    "image credit",
    "listen to article",
    "member-only",
    "min read",
    "photo from",
    "read more",
    "read it free",
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

/// Locate `needle` (already trimmed + lowercased) within `line` (already
/// lowercased chars). A `*` matches any run of characters, so the non-empty
/// segments between `*`s must appear in order. Returns the char range spanning
/// the whole match, or None. A needle of only `*`s matches nothing.
fn match_on_line(line: &[char], needle: &str) -> Option<(usize, usize)> {
    let segments: Vec<Vec<char>> = needle
        .split('*')
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().collect())
        .collect();
    let (first, rest) = segments.split_first()?;

    let find_from = |seg: &[char], from: usize| -> Option<usize> {
        if seg.is_empty() || seg.len() > line.len() || from > line.len() - seg.len() {
            return None;
        }
        (from..=line.len() - seg.len()).find(|&i| &line[i..i + seg.len()] == seg)
    };

    let start = find_from(first, 0)?;
    let mut cursor = start + first.len();
    for seg in rest {
        cursor = find_from(seg, cursor)? + seg.len();
    }
    Some((start, cursor))
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
        let mut best: Option<(usize, usize, &String)> = None;
        for needle in &needles {
            if let Some((start, end)) = match_on_line(&lower, needle) {
                if best.is_none_or(|(b, ..)| start < b) {
                    best = Some((start, end, needle));
                }
            }
        }
        if let Some((start, end, needle)) = best {
            return Some(JunkMatch {
                line_idx,
                match_start: start,
                match_end: end,
                phrase: needle.clone(),
            });
        }
    }
    None
}

/// Line-start signals that a short line is page furniture, not prose.
const BOILERPLATE_PREFIXES: [&str; 14] = [
    "published",
    "updated ",
    "by ",
    "photo",
    "image",
    "illustration",
    "advertisement",
    "sponsored",
    "share this",
    "subscribe",
    "sign up",
    "follow us",
    "related",
    "trending",
];

const MONTHS: [&str; 12] = [
    "january", "february", "march", "april", "may", "june", "july", "august",
    "september", "october", "november", "december",
];

fn looks_like_date(line: &str) -> bool {
    let l = line.to_lowercase();
    let words: Vec<&str> = l.split(|c: char| !c.is_alphabetic()).collect();
    let has_month = words.iter().any(|w| MONTHS.contains(w));
    let has_year = l
        .split(|c: char| !c.is_ascii_digit())
        .any(|w| w.len() == 4 && !w.is_empty());
    if has_month && has_year {
        return true;
    }
    // Pure numeric date: digits plus exactly two of / - .
    let seps = l.chars().filter(|c| "/-.".contains(*c)).count();
    let only_date_chars = !l.is_empty()
        && l.chars().all(|c| c.is_ascii_digit() || "/-. ".contains(c))
        && l.chars().any(|c| c.is_ascii_digit());
    only_date_chars && seps == 2
}

fn is_all_caps(line: &str) -> bool {
    let letters: Vec<char> = line.chars().filter(|c| c.is_alphabetic()).collect();
    letters.len() >= 3 && letters.iter().all(|c| c.is_uppercase())
}

fn has_min_read(line: &str) -> bool {
    let l = line.to_lowercase();
    let Some(idx) = l.find("min read") else { return false };
    l[..idx]
        .trim_end()
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_digit())
}

/// Scan `text` for lines that read like page furniture and aren't already
/// covered by `existing` junk phrases. Returns candidate phrases (lowercased,
/// trimmed, document order, deduped, capped at 15). The caller reviews them.
// ponytail: fixed prefix/signal heuristic; if hit rate is poor, learn signals
// from what the user actually adds instead of hand-maintaining the list.
pub fn suggest_phrases(text: &str, existing: &[String]) -> Vec<String> {
    let needles: Vec<String> = existing
        .iter()
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect();
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 60 {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if out.contains(&lower) {
            continue;
        }
        let lower_chars: Vec<char> = lower.chars().collect();
        if needles
            .iter()
            .any(|n| match_on_line(&lower_chars, n).is_some())
        {
            continue;
        }
        let is_boiler = has_min_read(trimmed)
            || BOILERPLATE_PREFIXES.iter().any(|p| lower.starts_with(p))
            || is_all_caps(trimmed)
            || looks_like_date(trimmed);
        if is_boiler {
            out.push(lower);
            if out.len() >= 15 {
                break;
            }
        }
    }
    out
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
        // a wildcard phrase added on top of the defaults is not "default"
        let mut wild = default_phrases();
        wild.push("photo*getty".to_string());
        assert!(!is_default(&wild));
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

    #[test]
    fn wildcard_spans_the_whole_run_and_respects_order() {
        let phrases = vec!["photo*getty".to_string()];
        let m = find_next("Photo credit: Getty Images", 0, &phrases).unwrap();
        assert_eq!(m.match_start, 0);
        assert_eq!(m.match_end, "Photo credit: Getty".len());
        // segments out of order: no match
        assert!(find_next("Getty then a photo", 0, &phrases).is_none());
    }

    #[test]
    fn wildcard_at_edges_is_plain_substring() {
        let phrases = vec!["*getty*".to_string()];
        let m = find_next("a getty b", 0, &phrases).unwrap();
        assert_eq!((m.match_start, m.match_end), (2, 7));
        // a needle of only stars matches nothing
        assert!(find_next("anything", 0, &vec!["*".to_string()]).is_none());
    }

    #[test]
    fn suggest_flags_boilerplate_lines_only() {
        let text = "The Real Headline\n\
                    By Jane Reporter\n\
                    Published March 3, 2024\n\
                    7 min read\n\
                    SHARE ON TWITTER\n\
                    This is the actual first paragraph of the article and it is quite long.\n\
                    Subscribe to our newsletter\n\
                    A normal sentence that happens to be short.";
        let got = suggest_phrases(text, &[]);
        assert_eq!(
            got,
            vec![
                "by jane reporter",
                "published march 3, 2024",
                "7 min read",
                "share on twitter",
                "subscribe to our newsletter",
            ]
        );
    }

    #[test]
    fn suggest_skips_lines_already_covered() {
        let text = "7 min read\nSubscribe now";
        let existing = vec!["min read".to_string()];
        let got = suggest_phrases(text, &existing);
        assert_eq!(got, vec!["subscribe now"]);
    }
}
