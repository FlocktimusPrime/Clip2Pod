use crate::voices::{Gender, VoiceInfo};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    #[error("Edge TTS request failed: {0}")]
    Service(String),
    #[error("no audio returned for voice {0}")]
    EmptyAudio(String),
    #[error("render cancelled")]
    Cancelled,
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

/// Escape text for embedding in XML (SSML requests and the RSS feed share
/// this). msedge-tts interpolates the text into XML verbatim, and the
/// service answers malformed XML (any bare & < > " ') with an empty turn —
/// no audio, no error.
pub(crate) fn xml_escape(text: &str) -> String {
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

    let mut prev_ended_with_punct = true; // no separator needed before the first sentence
    for (sentence, ends_with_punct) in split_sentences(text) {
        if sentence.chars().count() > MAX_CHUNK_CHARS {
            flush(&mut cur, &mut chunks);
            chunks.extend(word_pack(&sentence));
            prev_ended_with_punct = true;
            continue;
        }
        let sep = usize::from(!cur.is_empty());
        if cur.chars().count() + sep + sentence.chars().count() > MAX_CHUNK_CHARS {
            flush(&mut cur, &mut chunks);
        }
        if !cur.is_empty() {
            // A line break with no terminal punctuation (e.g. a heading) gets
            // a synthetic period so the TTS engine pauses the same way it
            // would after a real sentence, instead of running straight on.
            if !prev_ended_with_punct {
                cur.push('.');
            }
            cur.push(' ');
        }
        cur.push_str(&sentence);
        prev_ended_with_punct = ends_with_punct;
    }
    flush(&mut cur, &mut chunks);
    chunks
}

/// Split on terminal punctuation and newlines, keeping the delimiter with
/// its sentence. Blank spans are dropped. Each segment reports whether it
/// ended because of `. ! ?` (true) or a bare newline (false).
fn split_sentences(text: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.chars() {
        cur.push(c);
        if matches!(c, '.' | '!' | '?' | '\n') {
            let trimmed = cur.trim();
            if !trimmed.is_empty() {
                out.push((trimmed.to_string(), c != '\n'));
            }
            cur.clear();
        }
    }
    let trimmed = cur.trim();
    if !trimmed.is_empty() {
        out.push((trimmed.to_string(), true));
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

/// Attempts per chunk before a render gives up. Edge TTS drops the websocket
/// every so often — Windows surfaces it as "os error 10054" (connection reset
/// by peer), and the Sec-MS-GEC token rotates every five minutes, so a long
/// article is exposed to a reset at any chunk boundary. A reconnect
/// regenerates the token and nearly always succeeds, and re-rendering one
/// 2500-char chunk is far cheaper than discarding a whole article.
const CHUNK_ATTEMPTS: u32 = 4;

/// First backoff step; each later one triples it (400ms, 1.2s, 3.6s).
const RETRY_BASE_DELAY: Duration = Duration::from_millis(400);

/// Run `attempt` until it yields audio, up to `CHUNK_ATTEMPTS` times, waiting
/// `base_delay * 3^(n-1)` between tries so a throttling service gets room
/// rather than a reconnect storm. Returns the last error once the attempts run
/// out, or `Cancelled` if `cancel` flips while we are backing off. `label`
/// names the chunk in the retry notices. Split from the network call so the
/// loop can be tested without a service: tests inject their own `attempt` and
/// a zero `base_delay`.
fn retry_chunk(
    mut attempt: impl FnMut() -> Result<Vec<u8>, TtsError>,
    cancel: &AtomicBool,
    base_delay: Duration,
    label: &str,
) -> Result<Vec<u8>, TtsError> {
    let mut last_err = None;
    for n in 0..CHUNK_ATTEMPTS {
        if n > 0 {
            std::thread::sleep(base_delay * 3u32.pow(n - 1));
            if cancel.load(Ordering::Relaxed) {
                return Err(TtsError::Cancelled);
            }
        }
        match attempt() {
            Ok(audio) => {
                // Silence here would hide both a service that has started
                // failing and one that has been quietly rescuing us.
                if n > 0 {
                    eprintln!("tts: {label} recovered on attempt {}", n + 1);
                }
                return Ok(audio);
            }
            Err(TtsError::Cancelled) => return Err(TtsError::Cancelled),
            Err(e) => {
                if n + 1 < CHUNK_ATTEMPTS {
                    eprintln!("tts: {label} attempt {} failed ({e}), retrying", n + 1);
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.expect("CHUNK_ATTEMPTS >= 1, so a failed loop always set an error"))
}

/// How long a single socket read may stall before we call the turn dead.
///
/// `synthesize` blocks reading frames until the service sends `turn.end`. A
/// link that dies without the peer managing a FIN or RST — wifi switched off,
/// laptop asleep, cable pulled — produces no error at all, so without this the
/// read waits on TCP keepalive: two hours on Windows. Edge TTS streams MP3
/// frames continuously once a turn starts, so a gap this long means the turn is
/// gone, not slow.
const READ_TIMEOUT: Duration = Duration::from_secs(20);

/// One websocket turn: connect, synthesize, insist on audio. A fresh
/// connection every time, because the client's socket state after a turn is
/// not guaranteed reusable — and because reconnecting regenerates the
/// Sec-MS-GEC token.
fn connect_and_synthesize(
    chunk: &str,
    config: &msedge_tts::tts::SpeechConfig,
) -> Result<Vec<u8>, TtsError> {
    let mut client =
        msedge_tts::tts::client::connect().map_err(|e| TtsError::Service(e.to_string()))?;
    // Timing out mid-TLS-record would desync the session, but every error path
    // here drops the client and reconnects, so there is no session to protect.
    client
        .set_read_timeout(Some(READ_TIMEOUT))
        .map_err(|e| TtsError::Service(format!("could not set socket read timeout: {e}")))?;
    let audio = client
        .synthesize(chunk, config)
        .map_err(|e| TtsError::Service(e.to_string()))?;
    if audio.audio_bytes.is_empty() {
        // Treated as retryable: now that text is escaped and chunked before
        // sending, an empty turn is likelier throttling than bad input.
        return Err(TtsError::EmptyAudio(config.voice_name.clone()));
    }
    Ok(audio.audio_bytes)
}

/// Synthesize `text` with the given voice (VoiceInfo::name) to MP3 bytes.
/// Edge TTS emits MP3 frames natively, so no re-encoding happens. Long text
/// is split into per-request chunks and the resulting MP3 frames are
/// concatenated — valid because every chunk uses the same constant-bitrate
/// format.
pub fn synthesize_bytes(voice_name: &str, text: &str) -> Result<Vec<u8>, TtsError> {
    synthesize_bytes_cb(voice_name, text, &AtomicBool::new(false), |_, _| {})
}

/// Like `synthesize_bytes`, but checks `cancel` before each chunk (returning
/// `TtsError::Cancelled`) and calls `on_progress(chunks_done, chunks_total)`
/// after each one.
// ponytail: cancellation lands between chunks, not mid-chunk — a websocket
// turn already in flight finishes before the loop bails. `READ_TIMEOUT` bounds
// that wait: a turn whose socket has gone silent gives up rather than pinning
// the render (and so the Cancel button) on a connection that is never coming
// back.
pub fn synthesize_bytes_cb(
    voice_name: &str,
    text: &str,
    cancel: &AtomicBool,
    on_progress: impl Fn(usize, usize),
) -> Result<Vec<u8>, TtsError> {
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

    let total = chunks.len();
    on_progress(0, total);
    let mut out = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(TtsError::Cancelled);
        }
        let label = format!("chunk {}/{}", i + 1, total);
        let audio = retry_chunk(
            || connect_and_synthesize(chunk, &config),
            cancel,
            RETRY_BASE_DELAY,
            &label,
        )?;
        out.extend_from_slice(&audio);
        on_progress(i + 1, total);
    }
    Ok(out)
}

/// Synthesize straight to an MP3 file.
pub fn synthesize_to_file(voice_name: &str, text: &str, out_path: &Path) -> Result<(), TtsError> {
    synthesize_to_file_cb(voice_name, text, out_path, &AtomicBool::new(false), |_, _| {})
}

/// `synthesize_to_file` with cancellation + progress. The file is written only
/// after every chunk succeeds, so a cancelled render leaves no partial MP3.
pub fn synthesize_to_file_cb(
    voice_name: &str,
    text: &str,
    out_path: &Path,
    cancel: &AtomicBool,
    on_progress: impl Fn(usize, usize),
) -> Result<(), TtsError> {
    let bytes = synthesize_bytes_cb(voice_name, text, cancel, on_progress)?;
    std::fs::write(out_path, &bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use tempfile::tempdir;

    /// Backoff is skipped in these so the retry loop runs at full speed; the
    /// delays themselves are arithmetic on `RETRY_BASE_DELAY`, not behaviour
    /// worth sleeping through.
    const NO_DELAY: Duration = Duration::ZERO;

    #[test]
    fn retry_recovers_from_transient_resets() {
        let calls = Cell::new(0u32);
        let audio = retry_chunk(
            || {
                calls.set(calls.get() + 1);
                if calls.get() < 3 {
                    // What a reset looks like by the time it reaches us.
                    Err(TtsError::Service("os error 10054".into()))
                } else {
                    Ok(vec![0xFF, 0xF3, 0x01])
                }
            },
            &AtomicBool::new(false),
            NO_DELAY,
            "chunk 1/1",
        )
        .unwrap();
        assert_eq!(audio, vec![0xFF, 0xF3, 0x01]);
        assert_eq!(calls.get(), 3, "should stop retrying the moment one succeeds");
    }

    #[test]
    fn retry_gives_up_after_the_attempt_budget_and_keeps_the_last_error() {
        let calls = Cell::new(0u32);
        let err = retry_chunk(
            || {
                calls.set(calls.get() + 1);
                Err(TtsError::Service(format!("reset {}", calls.get())))
            },
            &AtomicBool::new(false),
            NO_DELAY,
            "chunk 2/8",
        )
        .unwrap_err();
        assert_eq!(calls.get(), CHUNK_ATTEMPTS);
        // The surfaced error is the final failure, not the first.
        assert!(matches!(&err, TtsError::Service(m) if m == "reset 4"), "got {err:?}");
    }

    #[test]
    fn retry_bails_when_cancelled_between_attempts() {
        let cancel = AtomicBool::new(false);
        let calls = Cell::new(0u32);
        let err = retry_chunk(
            || {
                calls.set(calls.get() + 1);
                // User hits Cancel while the first attempt is in flight.
                cancel.store(true, Ordering::Relaxed);
                Err(TtsError::Service("os error 10054".into()))
            },
            &cancel,
            NO_DELAY,
            "chunk 1/1",
        )
        .unwrap_err();
        assert!(matches!(err, TtsError::Cancelled), "got {err:?}");
        assert_eq!(calls.get(), 1, "cancel must land before a second attempt");
    }

    #[test]
    fn retry_does_not_retry_a_cancellation() {
        let calls = Cell::new(0u32);
        let err = retry_chunk(
            || {
                calls.set(calls.get() + 1);
                Err(TtsError::Cancelled)
            },
            &AtomicBool::new(false),
            NO_DELAY,
            "chunk 1/1",
        )
        .unwrap_err();
        assert!(matches!(err, TtsError::Cancelled), "got {err:?}");
        assert_eq!(calls.get(), 1);
    }

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
    fn heading_without_punctuation_gets_synthetic_pause() {
        // A subheading with no terminal punctuation, followed by body text,
        // must not run straight into the next line.
        let chunks = chunk_text("Section One\nThe body text starts here.");
        assert_eq!(chunks, vec!["Section One. The body text starts here.".to_string()]);
    }

    #[test]
    fn punctuated_line_break_is_not_double_punctuated() {
        let chunks = chunk_text("Sentence one.\nSentence two.");
        assert_eq!(chunks, vec!["Sentence one. Sentence two.".to_string()]);
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
    fn cancel_flag_stops_before_any_network_call() {
        let cancel = AtomicBool::new(true);
        let seen = std::sync::Mutex::new(Vec::new());
        let err = synthesize_bytes_cb("voice", "One. Two. Three.", &cancel, |d, t| {
            seen.lock().unwrap().push((d, t))
        })
        .unwrap_err();
        assert!(matches!(err, TtsError::Cancelled));
        // total is reported up front; the loop bails before chunk 1.
        assert_eq!(*seen.lock().unwrap(), vec![(0usize, 1usize)]);
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
