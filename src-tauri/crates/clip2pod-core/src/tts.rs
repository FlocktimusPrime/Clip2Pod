use crate::voices::{Gender, VoiceInfo};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    #[error("Edge TTS request failed: {0}")]
    Service(String),
    #[error("no audio returned for voice {0}")]
    EmptyAudio(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn parse_gender(g: &Option<String>) -> Gender {
    match g.as_deref() {
        Some("Female") => Gender::Female,
        _ => Gender::Male,
    }
}

/// Fetch the full Edge TTS voice catalog and map it to our VoiceInfo.
pub fn fetch_voices() -> Result<Vec<VoiceInfo>, TtsError> {
    let raw = msedge_tts::voice::get_voices_list().map_err(|e| TtsError::Service(e.to_string()))?;
    Ok(raw
        .into_iter()
        .map(|v| {
            let locale = v.locale.clone().unwrap_or_default();
            let mut parts = locale.split('-');
            let language = parts.next().unwrap_or_default().to_string();
            let country = parts.next().unwrap_or_default().to_string();
            let category = v
                .voice_tag
                .as_ref()
                .and_then(|t| t.content_categories.as_ref())
                .map(|c| c.join(", "))
                .unwrap_or_default();
            VoiceInfo {
                short_name: v.short_name.clone().unwrap_or_else(|| v.name.clone()),
                name: v.name,
                gender: parse_gender(&v.gender),
                locale,
                language,
                country,
                category,
            }
        })
        .collect())
}

/// Escape text for embedding in the SSML request. msedge-tts interpolates
/// the text into XML verbatim, and the service answers malformed XML (any
/// bare & < > " ') with an empty turn — no audio, no error.
fn xml_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// Max characters per Edge TTS request. A single websocket turn silently
/// returns zero audio frames when the text is too long, so long articles
/// must be split. Kept well under the observed failure threshold.
const MAX_CHUNK_CHARS: usize = 2500;

/// Break narration text into request-sized chunks. Splits on sentence and
/// line boundaries; a single oversized sentence falls back to word packing.
/// Never cuts mid-word.
fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut cur = String::new();

    let flush = |cur: &mut String, chunks: &mut Vec<String>| {
        if !cur.is_empty() {
            chunks.push(std::mem::take(cur));
        }
    };

    for sentence in split_sentences(text) {
        if sentence.chars().count() > MAX_CHUNK_CHARS {
            flush(&mut cur, &mut chunks);
            chunks.extend(word_pack(&sentence));
            continue;
        }
        let sep = usize::from(!cur.is_empty());
        if cur.chars().count() + sep + sentence.chars().count() > MAX_CHUNK_CHARS {
            flush(&mut cur, &mut chunks);
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(&sentence);
    }
    flush(&mut cur, &mut chunks);
    chunks
}

/// Split on terminal punctuation and newlines, keeping the delimiter with
/// its sentence. Blank spans are dropped.
fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        cur.push(c);
        if matches!(c, '.' | '!' | '?' | '\n') {
            let trimmed = cur.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
            cur.clear();
        }
    }
    let trimmed = cur.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
    out
}

/// Greedily pack whitespace-separated words into under-budget chunks. Last
/// resort for a sentence with no usable internal boundary.
fn word_pack(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let sep = usize::from(!cur.is_empty());
        if !cur.is_empty()
            && cur.chars().count() + sep + word.chars().count() > MAX_CHUNK_CHARS
        {
            chunks.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        chunks.push(cur);
    }
    chunks
}

/// Synthesize `text` with the given voice (VoiceInfo::name) to MP3 bytes.
/// Edge TTS emits MP3 frames natively, so no re-encoding happens. Long text
/// is split into per-request chunks and the resulting MP3 frames are
/// concatenated — valid because every chunk uses the same constant-bitrate
/// format.
pub fn synthesize_bytes(voice_name: &str, text: &str) -> Result<Vec<u8>, TtsError> {
    let config = msedge_tts::tts::SpeechConfig {
        voice_name: voice_name.to_string(),
        audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
        pitch: 0,
        rate: 0,
        volume: 0,
    };

    // Escape before chunking so the per-request budget counts the escaped
    // length actually sent. Entities contain no whitespace, so chunk
    // boundaries can never split one.
    let text = xml_escape(text);
    let chunks = chunk_text(&text);
    if chunks.is_empty() {
        return Err(TtsError::EmptyAudio(voice_name.to_string()));
    }

    let mut out = Vec::new();
    for chunk in &chunks {
        // Fresh connection per chunk: the client returns once a turn ends and
        // its socket state afterward is not guaranteed reusable.
        let mut client =
            msedge_tts::tts::client::connect().map_err(|e| TtsError::Service(e.to_string()))?;
        let audio = client
            .synthesize(chunk, &config)
            .map_err(|e| TtsError::Service(e.to_string()))?;
        if audio.audio_bytes.is_empty() {
            return Err(TtsError::EmptyAudio(voice_name.to_string()));
        }
        out.extend_from_slice(&audio.audio_bytes);
    }
    Ok(out)
}

/// Synthesize straight to an MP3 file.
pub fn synthesize_to_file(voice_name: &str, text: &str, out_path: &Path) -> Result<(), TtsError> {
    let bytes = synthesize_bytes(voice_name, text)?;
    std::fs::write(out_path, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn xml_escape_replaces_all_five_specials() {
        assert_eq!(
            xml_escape("AT&T says x < y > z, \"quoted\" and it's"),
            "AT&amp;T says x &lt; y &gt; z, &quot;quoted&quot; and it&apos;s"
        );
    }

    #[test]
    fn xml_escape_leaves_clean_text_untouched() {
        let text = "Plain sentence. No specials here!";
        assert_eq!(xml_escape(text), text);
    }

    #[test]
    fn xml_escape_does_not_double_escape_output() {
        // '&amp;' escaped again would be '&amp;amp;' — input '&' maps once.
        assert_eq!(xml_escape("&"), "&amp;");
        assert_eq!(xml_escape("&&"), "&amp;&amp;");
    }

    #[test]
    fn short_text_is_a_single_chunk() {
        let chunks = chunk_text("Hello world. This is fine.");
        assert_eq!(chunks, vec!["Hello world. This is fine.".to_string()]);
    }

    #[test]
    fn empty_text_yields_no_chunks() {
        assert!(chunk_text("").is_empty());
        assert!(chunk_text("   \n  ").is_empty());
    }

    #[test]
    fn long_text_splits_under_budget_without_losing_words() {
        // ~40k chars of sentences, forcing many chunks.
        let sentence = "The quick brown fox jumps over the lazy dog. ";
        let text = sentence.repeat(900);
        let chunks = chunk_text(&text);

        assert!(chunks.len() > 1, "expected multiple chunks");
        for c in &chunks {
            assert!(c.chars().count() <= MAX_CHUNK_CHARS, "chunk over budget: {}", c.chars().count());
            assert!(!c.is_empty());
        }

        // Every word survives the split.
        let original_words: usize = text.split_whitespace().count();
        let chunked_words: usize = chunks.iter().map(|c| c.split_whitespace().count()).sum();
        assert_eq!(original_words, chunked_words);
    }

    #[test]
    fn oversized_single_sentence_falls_back_to_word_packing() {
        // One sentence with no terminal punctuation, longer than the budget.
        let text = "word ".repeat(1000);
        let chunks = chunk_text(&text);
        assert!(chunks.len() > 1);
        for c in &chunks {
            assert!(c.chars().count() <= MAX_CHUNK_CHARS);
        }
    }

    #[test]
    #[ignore = "hits the live Edge TTS service"]
    fn synthesizes_long_multichunk_article() {
        let voices = fetch_voices().unwrap();
        let voice = voices
            .iter()
            .find(|v| v.locale == "en-US" && v.gender == Gender::Male)
            .expect("an en-US male voice exists");

        // ~8k chars -> forces several chunks past MAX_CHUNK_CHARS.
        let para = "Modern weight loss medications have transformed obesity treatment, helping many people lose significant amounts of weight. But these drugs often come with an important drawback. They can also reduce muscle mass. ";
        let text = para.repeat(40);
        assert!(text.chars().count() > MAX_CHUNK_CHARS * 2);

        let bytes = synthesize_bytes(&voice.name, &text).unwrap();
        // Concatenated frames should be a sizable MP3 with the classic
        // frame sync bits (0xFF 0xEx / 0xFx) somewhere in the head.
        assert!(bytes.len() > 100_000, "audio too small: {} bytes", bytes.len());
        assert!(bytes.windows(2).any(|w| w[0] == 0xFF && (w[1] & 0xE0) == 0xE0));
    }

    #[test]
    #[ignore = "hits the live Edge TTS service"]
    fn synthesizes_text_with_xml_special_chars() {
        let voices = fetch_voices().unwrap();
        let voice = voices
            .iter()
            .find(|v| v.locale == "en-US" && v.gender == Gender::Male)
            .expect("an en-US male voice exists");

        // Symbols that survive clean_for_tts and would break the SSML XML
        // if sent unescaped.
        let text = "AT&T and Q&A sessions. We know x < y and y > z. \
                    She said \"hello\" and it's fine.";
        let bytes = synthesize_bytes(&voice.name, text).unwrap();
        assert!(bytes.len() > 1000, "audio too small: {} bytes", bytes.len());
    }

    /// Real-network spike: `cargo test -p clip2pod-core -- --ignored`
    #[test]
    #[ignore = "hits the live Edge TTS service"]
    fn synthesizes_real_mp3_and_tags_it() {
        let voices = fetch_voices().unwrap();
        assert!(voices.len() > 50, "expected a large catalog, got {}", voices.len());
        let voice = voices
            .iter()
            .find(|v| v.locale == "en-US" && v.gender == Gender::Male)
            .expect("an en-US male voice exists");

        let dir = tempdir().unwrap();
        let path = dir.path().join("spike.mp3");
        synthesize_to_file(&voice.name, "Clip2Pod version two test narration.", &path).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() > 1000, "audio too small: {} bytes", bytes.len());

        crate::tagging::tag_mp3(&path, "Spike", "Tester", &voice.short_name, "", None, None).unwrap();
        let tag = id3::Tag::read_from_path(&path).unwrap();
        use id3::TagLike;
        assert_eq!(tag.album(), Some("Clip2Pod"));
    }
}
