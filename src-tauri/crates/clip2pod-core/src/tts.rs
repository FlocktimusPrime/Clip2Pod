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

/// Synthesize `text` with the given voice (VoiceInfo::name) to MP3 bytes.
/// Edge TTS emits MP3 frames natively, so no re-encoding happens.
pub fn synthesize_bytes(voice_name: &str, text: &str) -> Result<Vec<u8>, TtsError> {
    let config = msedge_tts::tts::SpeechConfig {
        voice_name: voice_name.to_string(),
        audio_format: "audio-24khz-48kbitrate-mono-mp3".to_string(),
        pitch: 0,
        rate: 0,
        volume: 0,
    };
    let mut client =
        msedge_tts::tts::client::connect().map_err(|e| TtsError::Service(e.to_string()))?;
    let audio = client
        .synthesize(text, &config)
        .map_err(|e| TtsError::Service(e.to_string()))?;
    if audio.audio_bytes.is_empty() {
        return Err(TtsError::EmptyAudio(voice_name.to_string()));
    }
    Ok(audio.audio_bytes)
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
