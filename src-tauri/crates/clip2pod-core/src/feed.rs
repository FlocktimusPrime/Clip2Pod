// Podcast feed logic: scan the output directory into Episodes and render
// RSS 2.0 (iTunes namespace). Pure functions — the HTTP layer lives in the
// Tauri shell.

use id3::TagLike;
use std::path::Path;
use std::time::SystemTime;

pub struct Episode {
    pub title: String,        // ID3 title, fallback: filename stem
    pub filename: String,     // bare filename, no path
    pub size: u64,            // bytes, for enclosure length
    pub modified: SystemTime, // pubDate
    pub artist: Option<String>,
    pub summary: Option<String>,
    pub source_url: Option<String>,
    pub duration_ms: Option<u32>,
}

/// Non-recursive scan for `*.mp3`, newest-first. Tag reads are best-effort:
/// an unreadable tag still yields an episode with a filename-stem title.
pub fn scan_episodes(dir: &Path) -> Vec<Episode> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut episodes: Vec<Episode> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("mp3"))
            {
                return None;
            }
            let meta = entry.metadata().ok()?;
            let filename = path.file_name()?.to_str()?.to_string();
            let stem = path.file_stem()?.to_str()?.to_string();
            let tag = id3::Tag::read_from_path(&path).ok();
            Some(Episode {
                title: tag
                    .as_ref()
                    .and_then(|t| t.title())
                    .map(str::to_string)
                    .unwrap_or(stem),
                // Empty tag strings read as absent: an empty TPE1/COMM/WOAF must
                // not surface as an empty <itunes:author/> etc. in the RSS.
                artist: tag
                    .as_ref()
                    .and_then(|t| t.artist())
                    .filter(|a| !a.is_empty())
                    .map(str::to_string),
                summary: tag
                    .as_ref()
                    .and_then(|t| {
                        t.comments()
                            .find(|c| c.description == "summary")
                            .map(|c| c.text.clone())
                    })
                    .filter(|s| !s.is_empty()),
                source_url: tag
                    .as_ref()
                    .and_then(|t| t.get("WOAF"))
                    .and_then(|f| f.content().link())
                    .filter(|u| !u.is_empty())
                    .map(str::to_string),
                duration_ms: tag.as_ref().and_then(|t| t.duration()),
                size: meta.len(),
                modified: meta.modified().ok()?,
                filename,
            })
        })
        .collect();
    episodes.sort_by(|a, b| b.modified.cmp(&a.modified));
    episodes
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// RFC 3986 path-segment percent-encoding (unreserved chars pass through).
fn encode_path_segment(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn percent_decode(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3)?;
            let hex = std::str::from_utf8(hex).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Percent-decode a requested audio filename; None for malformed encoding or
/// anything that could escape the output directory (the caller answers 400).
pub fn decode_audio_name(raw: &str) -> Option<String> {
    let name = percent_decode(raw)?;
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.is_empty() {
        return None;
    }
    Some(name)
}

fn format_duration(ms: u32) -> String {
    let secs = ms / 1000;
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}

/// Render the full RSS document. `base_url` has no trailing slash,
/// e.g. `http://192.168.1.20:4738`.
pub fn build_rss(episodes: &[Episode], base_url: &str) -> String {
    let mut items = String::new();
    for ep in episodes {
        let pub_date = chrono::DateTime::<chrono::Utc>::from(ep.modified).to_rfc2822();
        items.push_str("<item>");
        items.push_str(&format!("<title>{}</title>", xml_escape(&ep.title)));
        if let Some(summary) = &ep.summary {
            items.push_str(&format!(
                "<description>{}</description>",
                xml_escape(summary)
            ));
        }
        if let Some(url) = &ep.source_url {
            items.push_str(&format!("<link>{}</link>", xml_escape(url)));
        }
        if let Some(artist) = &ep.artist {
            items.push_str(&format!(
                "<itunes:author>{}</itunes:author>",
                xml_escape(artist)
            ));
        }
        if let Some(ms) = ep.duration_ms {
            items.push_str(&format!(
                "<itunes:duration>{}</itunes:duration>",
                format_duration(ms)
            ));
        }
        items.push_str(&format!(
            r#"<enclosure url="{base_url}/audio/{}" length="{}" type="audio/mpeg"/>"#,
            encode_path_segment(&ep.filename),
            ep.size
        ));
        items.push_str(&format!(
            r#"<guid isPermaLink="false">{}</guid>"#,
            xml_escape(&ep.filename)
        ));
        items.push_str(&format!("<pubDate>{pub_date}</pubDate>"));
        items.push_str("</item>");
    }
    format!(
        "{}{}{}{}{}{}{}{}",
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">"#,
        "<channel><title>Clip2Pod</title><description>Articles narrated by Clip2Pod</description>",
        format!("<link>{base_url}/feed.xml</link>"),
        "<language>en</language>",
        format!(r#"<itunes:image href="{base_url}/cover.png"/>"#),
        items,
        "</channel></rss>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};
    use tempfile::tempdir;

    fn ep(title: &str, filename: &str) -> Episode {
        Episode {
            title: title.into(),
            filename: filename.into(),
            size: 1234,
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1_750_000_000),
            artist: None,
            summary: None,
            source_url: None,
            duration_ms: None,
        }
    }

    #[test]
    fn scan_missing_dir_is_empty() {
        assert!(scan_episodes(std::path::Path::new("/no/such/dir")).is_empty());
    }

    #[test]
    fn scan_reads_tags_and_sorts_newest_first() {
        use id3::TagLike;
        let dir = tempdir().unwrap();
        let fake = |name: &str| {
            let path = dir.path().join(name);
            let mut bytes = vec![0xFF, 0xFB, 0x90, 0x00];
            bytes.extend(std::iter::repeat(0u8).take(128));
            std::fs::write(&path, &bytes).unwrap();
            path
        };
        let old = fake("old.mp3");
        let mut tag = id3::Tag::new();
        tag.set_title("Old Article");
        tag.set_artist("Jane");
        tag.set_duration(90_000);
        tag.add_frame(id3::frame::Comment {
            lang: "eng".into(),
            description: "summary".into(),
            text: "The gist.".into(),
        });
        tag.add_frame(id3::Frame::with_content(
            "WOAF",
            id3::Content::Link("https://example.com/old".into()),
        ));
        tag.write_to_path(&old, id3::Version::Id3v24).unwrap();

        fake("untagged.mp3");
        fake("not-audio.txt");
        // ensure distinct mtimes: make the tagged file an hour older
        let past = SystemTime::now() - Duration::from_secs(3600);
        std::fs::File::options()
            .write(true)
            .open(&old)
            .unwrap()
            .set_modified(past)
            .unwrap();

        let eps = scan_episodes(dir.path());
        assert_eq!(eps.len(), 2);
        assert_eq!(eps[0].filename, "untagged.mp3");
        assert_eq!(eps[0].title, "untagged"); // filename stem fallback
        assert!(eps[0].summary.is_none());
        assert!(eps[0].size > 0);
        let old_ep = &eps[1];
        assert_eq!(old_ep.title, "Old Article");
        assert_eq!(old_ep.artist.as_deref(), Some("Jane"));
        assert_eq!(old_ep.summary.as_deref(), Some("The gist."));
        assert_eq!(old_ep.source_url.as_deref(), Some("https://example.com/old"));
        assert_eq!(old_ep.duration_ms, Some(90_000));
    }

    #[test]
    fn scan_treats_empty_tag_strings_as_absent() {
        use id3::TagLike;
        let dir = tempdir().unwrap();
        let path = dir.path().join("no-author.mp3");
        let mut bytes = vec![0xFF, 0xFB, 0x90, 0x00];
        bytes.extend(std::iter::repeat(0u8).take(128));
        std::fs::write(&path, &bytes).unwrap();
        let mut tag = id3::Tag::new();
        tag.set_title("No Author");
        tag.set_artist(""); // empty TPE1 frame must not become <itunes:author/>
        tag.write_to_path(&path, id3::Version::Id3v24).unwrap();

        let eps = scan_episodes(dir.path());
        assert_eq!(eps.len(), 1);
        assert!(eps[0].artist.is_none());
        let xml = build_rss(&eps, "http://192.168.1.5:4738");
        assert!(!xml.contains("itunes:author"));
    }

    #[test]
    fn rss_escapes_and_encodes() {
        let eps = vec![ep("Tom & <Jerry>", "tom & jerry.mp3")];
        let xml = build_rss(&eps, "http://192.168.1.5:4738");
        assert!(xml.contains("<title>Tom &amp; &lt;Jerry&gt;</title>"));
        assert!(xml.contains(r#"url="http://192.168.1.5:4738/audio/tom%20%26%20jerry.mp3""#));
        assert!(xml.contains(r#"length="1234""#));
        assert!(xml.contains(r#"<guid isPermaLink="false">tom &amp; jerry.mp3</guid>"#));
        assert!(xml.contains("<pubDate>"));
        assert!(xml.contains(r#"<itunes:image href="http://192.168.1.5:4738/cover.png"/>"#));
        assert!(xml.contains("<title>Clip2Pod</title>"));
    }

    #[test]
    fn rss_omits_absent_optional_elements() {
        let xml = build_rss(&[ep("T", "t.mp3")], "http://h:4738");
        // channel keeps its required description...
        assert!(xml.contains("<description>Articles narrated by Clip2Pod</description>"));
        // ...but the item (no optional fields set) carries none of the optional elements
        let items = &xml[xml.find("<item>").unwrap()..];
        assert!(!items.contains("<description>"));
        assert!(!items.contains("<link>"));
        assert!(!items.contains("<itunes:author>"));
        assert!(!items.contains("<itunes:duration>"));
    }

    #[test]
    fn rss_renders_optional_elements() {
        let mut e = ep("T", "t.mp3");
        e.artist = Some("Jane".into());
        e.summary = Some("A & B".into());
        e.source_url = Some("https://example.com/x?a=1&b=2".into());
        e.duration_ms = Some(3_723_000); // 1h 2m 3s
        let xml = build_rss(&[e], "http://h:4738");
        assert!(xml.contains("<description>A &amp; B</description>"));
        assert!(xml.contains("<link>https://example.com/x?a=1&amp;b=2</link>"));
        assert!(xml.contains("<itunes:author>Jane</itunes:author>"));
        assert!(xml.contains("<itunes:duration>01:02:03</itunes:duration>"));
    }

    #[test]
    fn empty_feed_is_valid_channel() {
        let xml = build_rss(&[], "http://h:4738");
        assert!(xml.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains("<channel>"));
        assert!(!xml.contains("<item>"));
    }

    #[test]
    fn decode_audio_name_roundtrips_and_rejects_traversal() {
        assert_eq!(
            decode_audio_name("tom%20%26%20jerry.mp3").as_deref(),
            Some("tom & jerry.mp3")
        );
        assert_eq!(decode_audio_name("plain.mp3").as_deref(), Some("plain.mp3"));
        assert!(decode_audio_name("..%2fCargo.toml").is_none());
        assert!(decode_audio_name("a/b.mp3").is_none());
        assert!(decode_audio_name("a%5Cb.mp3").is_none()); // backslash
        assert!(decode_audio_name("..").is_none());
        assert!(decode_audio_name("").is_none());
        assert!(decode_audio_name("bad%zz").is_none()); // invalid hex
    }
}
