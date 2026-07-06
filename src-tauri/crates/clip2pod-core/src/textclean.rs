use serde::Serialize;

/// Metadata auto-filled from cleaned text per the spec's heuristics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Autofill {
    pub title: String,
    pub author: String,
    pub filename_title: String,
}

/// Punctuation that reads naturally aloud; never stripped, and runs of it
/// (e.g. "...", "?!") are left alone.
const SENTENCE_PUNCT: &[char] = &['.', ',', ';', ':', '!', '?', '\'', '"', '(', ')', '-'];

fn is_pictograph(c: char) -> bool {
    matches!(c as u32,
        0x1F000..=0x1FAFF   // emoji, symbols, pictographs
        | 0x2600..=0x27BF   // misc symbols + dingbats (✓ ★ ☀ …)
        | 0x2B00..=0x2BFF   // arrows, stars
        | 0x2190..=0x21FF   // arrows
        | 0x25A0..=0x25FF   // geometric shapes (▶ ●)
        | 0x2500..=0x257F   // box drawing
        | 0xFE00..=0xFE0F   // variation selectors
        | 0x200D..=0x200D   // zero-width joiner
        | 0x2022..=0x2023   // bullets
        | 0x00A9..=0x00A9   // ©
        | 0x00AE..=0x00AE   // ®
        | 0x2122..=0x2122   // ™
    )
}

/// Smart typographic character to plain-ASCII replacement; None = keep as-is.
fn ascii_fold(c: char) -> Option<&'static str> {
    Some(match c {
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{2032}' => "'",
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{2033}' => "\"",
        '\u{2013}' | '\u{2014}' | '\u{2015}' | '\u{2212}' => "-",
        '\u{2026}' => "...",
        '\u{00A0}' | '\u{2009}' | '\u{200A}' | '\u{2002}' | '\u{2003}' => " ",
        _ => return None,
    })
}

fn is_stray_symbol(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace() && !SENTENCE_PUNCT.contains(&c)
}

/// Remove runs of two or more stray symbols, keeping single occurrences
/// (so "50% off" survives but "##@@##" does not).
fn strip_symbol_runs(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < chars.len() {
        if is_stray_symbol(chars[i]) {
            let mut j = i;
            while j < chars.len() && is_stray_symbol(chars[j]) {
                j += 1;
            }
            if j - i == 1 {
                out.push(chars[i]);
            } else {
                out.push(' ');
            }
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Clean raw pasted text for TTS narration.
pub fn clean_for_tts(text: &str) -> String {
    let mut folded = String::with_capacity(text.len());
    for c in text.chars() {
        if is_pictograph(c) {
            continue;
        }
        if let Some(rep) = ascii_fold(c) {
            folded.push_str(rep);
        } else if c == '\n' || c == '\t' || !c.is_control() {
            folded.push(c);
        }
    }

    folded
        .lines()
        .map(strip_symbol_runs)
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Derive Title / Author / Filename-title from already-cleaned text.
/// Title and filename-title come from line 1; author is the shorter of
/// lines 2 and 3 (byline heuristic).
pub fn autofill_from(cleaned: &str) -> Autofill {
    let lines: Vec<&str> = cleaned.lines().collect();
    let title = lines.first().copied().unwrap_or("").to_string();
    let author = match (lines.get(1), lines.get(2)) {
        (Some(a), Some(b)) => {
            if a.chars().count() <= b.chars().count() { a } else { b }
        }
        (Some(a), None) => a,
        _ => "",
    }
    .to_string();
    Autofill {
        filename_title: title.clone(),
        title,
        author,
    }
}

/// Lowercased, trimmed, trailing sentence punctuation stripped — for
/// "is the title already the first line?" comparisons.
fn title_key(s: &str) -> String {
    s.trim()
        .trim_end_matches(['.', '!', '?', ':', '…'])
        .trim()
        .to_lowercase()
}

/// Narration text for TTS: prepend a spoken "Title. / By Author." intro
/// unless the cleaned text already starts with the title (pasted articles
/// usually carry it as line 1; extracted ones never do).
pub fn narration_text(cleaned: &str, title: &str, author: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        return cleaned.to_string();
    }
    let first_line = cleaned.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    if title_key(first_line) == title_key(title) {
        return cleaned.to_string();
    }
    let mut intro = title.to_string();
    if !title.ends_with(['.', '!', '?', ':', '…']) {
        intro.push('.');
    }
    let author = author.trim();
    if !author.is_empty() {
        intro.push_str(&format!("\nBy {author}."));
    }
    format!("{intro}\n\n{cleaned}")
}

/// First ~300 chars of the text with whitespace collapsed — the episode
/// description shown in podcast players.
pub fn summary_snippet(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.chars().count() <= 300 {
        return collapsed;
    }
    let cut: String = collapsed.chars().take(300).collect();
    format!("{}…", cut.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_smart_punctuation_to_ascii() {
        assert_eq!(
            clean_for_tts("\u{201c}Hello\u{201d} \u{2018}world\u{2019} \u{2014} fine \u{2013} ok\u{a0}now\u{2026}"),
            "\"Hello\" 'world' - fine - ok now..."
        );
    }

    #[test]
    fn strips_emoji_and_pictographs() {
        assert_eq!(clean_for_tts("Great news \u{1f600}\u{1f680} today"), "Great news today");
    }

    #[test]
    fn strips_decorative_bullets_and_marks() {
        assert_eq!(
            clean_for_tts("\u{2022} item one \u{2713} done \u{2605} starred \u{a9} 2024 Acme\u{2122}"),
            "item one done starred 2024 Acme"
        );
    }

    #[test]
    fn strips_runs_of_stray_symbols() {
        assert_eq!(clean_for_tts("Read this ##@@## now"), "Read this now");
    }

    #[test]
    fn keeps_normal_sentence_punctuation() {
        assert_eq!(
            clean_for_tts("Wait, really? Yes: it's done (finally). 50% off!"),
            "Wait, really? Yes: it's done (finally). 50% off!"
        );
    }

    #[test]
    fn collapses_whitespace_and_drops_blank_lines() {
        assert_eq!(
            clean_for_tts("First   line\n\n\n  Second\tline  \n\n"),
            "First line\nSecond line"
        );
    }

    #[test]
    fn autofill_title_from_first_line() {
        let a = autofill_from("Big Headline\nBy Jane Doe\nSome long first paragraph of the article body");
        assert_eq!(a.title, "Big Headline");
        assert_eq!(a.filename_title, "Big Headline");
    }

    #[test]
    fn autofill_author_picks_shorter_of_lines_two_and_three() {
        let a = autofill_from("Headline\nA very long subtitle that is clearly not a byline\nBy Jane Doe");
        assert_eq!(a.author, "By Jane Doe");

        let b = autofill_from("Headline\nBy Jane Doe\nA very long first paragraph of body text here");
        assert_eq!(b.author, "By Jane Doe");
    }

    #[test]
    fn autofill_handles_short_texts() {
        let a = autofill_from("Only line");
        assert_eq!(a.title, "Only line");
        assert_eq!(a.author, "");

        let b = autofill_from("Title\nByline");
        assert_eq!(b.author, "Byline");

        let c = autofill_from("");
        assert_eq!(c.title, "");
        assert_eq!(c.author, "");
    }

    #[test]
    fn narration_prepends_title_and_author_when_missing() {
        assert_eq!(
            narration_text("Body starts here.", "My Title", "Jane Doe"),
            "My Title.\nBy Jane Doe.\n\nBody starts here."
        );
    }

    #[test]
    fn narration_omits_author_line_when_blank() {
        assert_eq!(
            narration_text("Body starts here.", "My Title", "  "),
            "My Title.\n\nBody starts here."
        );
    }

    #[test]
    fn narration_skips_intro_when_title_is_first_line() {
        let text = "My Title\nBy Jane Doe\nBody starts here.";
        assert_eq!(narration_text(text, "My Title", "Jane Doe"), text);
        // case and trailing punctuation differences still count as a match
        assert_eq!(narration_text(text, "my title.", "Jane Doe"), text);
        let dotted = "My Title.\nBody";
        assert_eq!(narration_text(dotted, "My Title", ""), dotted);
    }

    #[test]
    fn narration_blank_title_leaves_text_unchanged() {
        assert_eq!(narration_text("Body", "", "Jane Doe"), "Body");
    }

    #[test]
    fn narration_keeps_existing_terminal_punctuation() {
        assert_eq!(
            narration_text("Body", "Why?", ""),
            "Why?\n\nBody"
        );
    }

    #[test]
    fn summary_collapses_whitespace() {
        assert_eq!(
            summary_snippet("Hello\n\n  world\tagain"),
            "Hello world again"
        );
    }

    #[test]
    fn summary_truncates_at_300_chars_with_ellipsis() {
        let long = "word ".repeat(100); // 500 chars collapsed to 499
        let s = summary_snippet(&long);
        assert!(s.chars().count() <= 301); // 300 + ellipsis
        assert!(s.ends_with('…'));
    }

    #[test]
    fn summary_short_text_unchanged() {
        assert_eq!(summary_snippet("short text"), "short text");
    }
}
