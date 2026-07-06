use std::path::Path;

/// Write ID3v2.4 tags: Title, Artist, Album (always "Clip2Pod"), a Comment
/// recording the narrating voice, and — when available — a summary comment,
/// source-URL WOAF frame and TLEN duration for the podcast feed.
/// Best-effort — callers log and continue on error.
pub fn tag_mp3(
    path: &Path,
    title: &str,
    artist: &str,
    voice: &str,
    summary: &str,
    source_url: Option<&str>,
    duration_ms: Option<u32>,
) -> Result<(), id3::Error> {
    use id3::TagLike;
    let mut tag = id3::Tag::read_from_path(path).unwrap_or_default();
    tag.set_title(title);
    tag.set_artist(artist);
    tag.set_album("Clip2Pod");
    tag.add_frame(id3::frame::Comment {
        lang: "eng".to_string(),
        description: "voice".to_string(),
        text: format!("Narrated by {voice}"),
    });
    if !summary.is_empty() {
        tag.add_frame(id3::frame::Comment {
            lang: "eng".to_string(),
            description: "summary".to_string(),
            text: summary.to_string(),
        });
    }
    if let Some(url) = source_url {
        tag.add_frame(id3::Frame::with_content(
            "WOAF",
            id3::Content::Link(url.to_string()),
        ));
    }
    if let Some(ms) = duration_ms {
        tag.set_duration(ms);
    }
    tag.write_to_path(path, id3::Version::Id3v24)
}

/// Frame-accurate MP3 duration in milliseconds; None on any failure
/// (measurement must never block tagging).
pub fn mp3_duration_ms(path: &Path) -> Option<u32> {
    mp3_duration::from_path(path)
        .ok()
        .map(|d| d.as_millis().min(u128::from(u32::MAX)) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use id3::TagLike;
    use tempfile::tempdir;

    fn fake_mp3(dir: &std::path::Path) -> std::path::PathBuf {
        let path = dir.join("song.mp3");
        // minimal fake MP3: a single MPEG frame sync header + padding
        let mut bytes = vec![0xFF, 0xFB, 0x90, 0x00];
        bytes.extend(std::iter::repeat(0u8).take(128));
        std::fs::write(&path, &bytes).unwrap();
        path
    }

    #[test]
    fn writes_and_reads_back_tags() {
        let dir = tempdir().unwrap();
        let path = fake_mp3(dir.path());

        tag_mp3(
            &path,
            "My Title",
            "Jane Doe",
            "en-US-GuyNeural",
            "A short summary.",
            Some("https://example.com/article"),
            Some(83_000),
        )
        .unwrap();

        let tag = id3::Tag::read_from_path(&path).unwrap();
        assert_eq!(tag.title(), Some("My Title"));
        assert_eq!(tag.artist(), Some("Jane Doe"));
        assert_eq!(tag.album(), Some("Clip2Pod"));
        let comments: Vec<_> = tag.comments().collect();
        assert!(comments.iter().any(|c| c.text.contains("en-US-GuyNeural")));
        assert!(comments
            .iter()
            .any(|c| c.description == "summary" && c.text == "A short summary."));
        let woaf = tag.get("WOAF").and_then(|f| f.content().link());
        assert_eq!(woaf, Some("https://example.com/article"));
        assert_eq!(tag.duration(), Some(83_000));
    }

    #[test]
    fn optional_frames_omitted_when_absent() {
        let dir = tempdir().unwrap();
        let path = fake_mp3(dir.path());

        tag_mp3(&path, "T", "A", "v", "", None, None).unwrap();

        let tag = id3::Tag::read_from_path(&path).unwrap();
        assert!(tag.get("WOAF").is_none());
        assert!(tag.duration().is_none());
        assert!(!tag.comments().any(|c| c.description == "summary"));
    }
}
