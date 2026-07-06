//! URL → readable plain text via reqwest + dom_smoothie (Readability port).

use dom_smoothie::{Config, Readability, TextMode};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ExtractError {
    #[error("could not fetch the page: {0}")]
    Http(String),
    #[error("could not find a readable article in the page: {0}")]
    Parse(String),
    #[error("the page produced no readable text")]
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Extracted {
    pub title: String,
    pub author: String,
    pub text: String,
}

/// Normalize a readability byline: trim and strip a leading "by" — but only
/// when it is its own word ("By Jane", "by: Jane"), so "Byron Smith" survives.
fn clean_byline(raw: Option<&str>) -> String {
    let byline = raw.unwrap_or("").trim();
    let rest = byline.get(..2).filter(|p| p.eq_ignore_ascii_case("by"));
    match rest.map(|_| &byline[2..]) {
        Some(after) if after.starts_with([' ', '\t', ':']) => {
            after.trim_start_matches([' ', '\t', ':']).trim().to_string()
        }
        _ => byline.to_string(),
    }
}

/// Run readability over already-fetched HTML. `url` resolves relative links.
pub fn extract_from_html(html: &str, url: &str) -> Result<Extracted, ExtractError> {
    let config = Config {
        text_mode: TextMode::Formatted,
        ..Config::default()
    };
    let mut readability = Readability::new(html, Some(url), Some(config))
        .map_err(|e| ExtractError::Parse(e.to_string()))?;
    let article = readability
        .parse()
        .map_err(|e| ExtractError::Parse(e.to_string()))?;
    let text = article.text_content.trim().to_string();
    // Readability falls back to the whole <body> on chrome-only pages,
    // yielding a few stray words; treat that as "nothing readable".
    if text.chars().count() < 25 {
        return Err(ExtractError::Empty);
    }
    Ok(Extracted {
        title: article.title,
        author: clean_byline(article.byline.as_deref()),
        text,
    })
}

const DESKTOP_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
     Chrome/126.0.0.0 Safari/537.36";

/// Fetch `url` with a desktop UA and a timeout, then extract the article.
pub fn extract_url(url: &str) -> Result<Extracted, ExtractError> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(DESKTOP_UA)
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| ExtractError::Http(e.to_string()))?;
    let response = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| ExtractError::Http(e.to_string()))?;
    let html = response.text().map_err(|e| ExtractError::Http(e.to_string()))?;
    extract_from_html(&html, url)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTICLE_HTML: &str = r#"<!doctype html>
<html><head><title>The Rise of Personal Audio — Example News</title></head>
<body>
  <nav><a href="/">Home</a><a href="/sports">Sports</a><a href="/subscribe">Subscribe now</a></nav>
  <article>
    <h1>The Rise of Personal Audio</h1>
    <p class="byline">By Jane Doe</p>
    <p>Listening habits have shifted dramatically over the past decade, and the
    pace shows no sign of slowing. What began as a niche pastime has become a
    daily ritual for millions of commuters around the world.</p>
    <p>Text-to-speech technology has matured to the point where a synthetic
    narrator can carry a long-form article without fatiguing the listener,
    opening written journalism to entirely new audiences.</p>
    <p>Publishers have taken note, and audio editions are now a standard part
    of many newsroom production pipelines rather than an afterthought.</p>
  </article>
  <footer>© Example News. All rights reserved. <a href="/privacy">Privacy</a></footer>
</body></html>"#;

    #[test]
    fn extracts_title_and_body_paragraphs() {
        let got = extract_from_html(ARTICLE_HTML, "https://news.example.com/audio").unwrap();
        assert!(got.title.contains("The Rise of Personal Audio"), "title: {}", got.title);
        assert!(got.text.contains("Listening habits have shifted"));
        assert!(got.text.contains("audio editions are now a standard part"));
    }

    #[test]
    fn keeps_paragraph_breaks() {
        let got = extract_from_html(ARTICLE_HTML, "https://news.example.com/audio").unwrap();
        let first = got.text.find("pace shows no sign").unwrap();
        let second = got.text.find("Text-to-speech technology").unwrap();
        assert!(
            got.text[first..second].contains('\n'),
            "paragraphs must be separated by a line break, got: {}",
            &got.text[first..second]
        );
    }

    #[test]
    fn drops_navigation_chrome() {
        let got = extract_from_html(ARTICLE_HTML, "https://news.example.com/audio").unwrap();
        assert!(!got.text.contains("Subscribe now"), "nav leaked into: {}", got.text);
    }

    #[test]
    fn extracts_author_from_byline_and_keeps_it_out_of_body() {
        let got = extract_from_html(ARTICLE_HTML, "https://news.example.com/audio").unwrap();
        assert_eq!(got.author, "Jane Doe");
        assert!(!got.text.contains("Jane Doe"), "byline leaked into: {}", got.text);
    }

    #[test]
    fn missing_byline_yields_empty_author() {
        let html = ARTICLE_HTML.replace(r#"<p class="byline">By Jane Doe</p>"#, "");
        let got = extract_from_html(&html, "https://news.example.com/audio").unwrap();
        assert_eq!(got.author, "");
    }

    #[test]
    fn clean_byline_strips_by_prefix_variants() {
        assert_eq!(clean_byline(Some("By Jane Doe")), "Jane Doe");
        assert_eq!(clean_byline(Some("by: Jane Doe")), "Jane Doe");
        assert_eq!(clean_byline(Some("BY Jane Doe")), "Jane Doe");
        assert_eq!(clean_byline(Some("  By Jane Doe  ")), "Jane Doe");
        assert_eq!(clean_byline(Some("Byron Smith")), "Byron Smith");
        assert_eq!(clean_byline(Some("Jane Doe")), "Jane Doe");
        assert_eq!(clean_byline(Some("   ")), "");
        assert_eq!(clean_byline(None), "");
    }

    #[test]
    fn unreadable_page_is_an_error() {
        let junk = "<html><body><nav><a href='/'>Home</a></nav></body></html>";
        let got = extract_from_html(junk, "https://news.example.com/empty");
        assert!(got.is_err(), "expected error, got: {got:?}");
    }

    /// Real-network spike: `cargo test -p clip2pod-core -- --ignored`
    #[test]
    #[ignore = "hits a live web page"]
    fn extracts_a_real_article() {
        let got = extract_url("https://en.wikipedia.org/wiki/Text-to-speech").unwrap();
        assert!(got.text.len() > 500, "too short: {} chars", got.text.len());
    }
}
