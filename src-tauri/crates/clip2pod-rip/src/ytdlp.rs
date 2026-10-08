// yt-dlp invocation logic: turn the user's args template into a safe argv,
// and parse `--newline` stdout into progress events. Pure functions — the
// process spawning lives in the Tauri shell.

use std::path::Path;

/// The user's proven command, adapted opus → mp3 (libopus postprocessor arg
/// dropped; the volume boost stays). `-P` and the URL are appended by
/// `build_args`, never stored in the template, so the feed dir can't drift.
pub const DEFAULT_ARGS: &str = "--restrict-filenames --trim-filenames 150 --no-overwrites --embed-thumbnail --embed-metadata --embed-chapters --sponsorblock-remove default --extract-audio --audio-format mp3 --audio-quality 0 --postprocessor-args \"ExtractAudio:-af volume=1.5\" -o \"%(title)s.%(ext)s\"";

/// Hosts where "there is an audio/video track worth ripping" is the sensible
/// default when the browser extension captures a page. Not exhaustive — yt-dlp
/// supports ~1800 sites; the extension's "Capture as video" override reaches the
/// rest. Matched as an exact host or a subdomain of one of these.
pub const RIPPABLE_HOSTS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "vimeo.com",
    "soundcloud.com",
    "twitch.tv",
    "dailymotion.com",
    "bilibili.com",
    "rumble.com",
    "odysee.com",
    "bandcamp.com",
    "ted.com",
    "nebula.tv",
    "x.com",
    "twitter.com",
];

/// Lowercased host of an http(s) URL, without userinfo or port. `None` for
/// anything that isn't a plain http(s) URL with a non-empty host.
fn url_host(url: &str) -> Option<String> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?; // drop any user:pass@
    let host = host.split(':').next()?; // drop :port
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

/// True when the URL's host is one of `RIPPABLE_HOSTS` or a subdomain of one.
pub fn rippable_host(url: &str) -> bool {
    let Some(host) = url_host(url) else {
        return false;
    };
    RIPPABLE_HOSTS
        .iter()
        .any(|&h| host == h || host.ends_with(&format!(".{h}")))
}

/// Accept only plain http(s) URLs; everything else could be an option
/// injection or a local path.
pub fn valid_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
        && url.len() > "https://".len()
}

/// Split the template shell-style and append the app-owned parts:
/// `--newline` (line-buffered progress), `-P <dir>`, `-- <url>`.
pub fn build_args(template: &str, output_dir: &Path, url: &str) -> Result<Vec<String>, String> {
    if !valid_url(url) {
        return Err(format!("not an http(s) URL: {url}"));
    }
    let mut args = shell_words::split(template)
        .map_err(|e| format!("args template has unbalanced quotes: {e}"))?;
    args.push("--newline".into());
    args.push("-P".into());
    args.push(output_dir.display().to_string());
    args.push("--".into());
    args.push(url.into());
    Ok(args)
}

/// One parsed line of yt-dlp stdout.
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressEvent {
    /// `[download]  42.3% of ~ 12MiB at 1.2MiB/s ...`
    Percent { percent: f32, speed: Option<String> },
    /// A postprocessor started: `[ExtractAudio]`, `[EmbedThumbnail]`, …
    Stage(String),
    /// `[download] Destination: /path/file.ext` — bare filename, no path.
    Destination(String),
}

/// Best-effort parse; None for lines that carry no progress signal
/// (extractor chatter, warnings, blank lines).
pub fn parse_line(line: &str) -> Option<ProgressEvent> {
    let line = line.trim();
    let rest = line.strip_prefix('[')?;
    let close = rest.find(']')?;
    let (tag, after) = (&rest[..close], rest[close + 1..].trim_start());

    if tag == "download" {
        if let Some(dest) = after.strip_prefix("Destination:") {
            let name = Path::new(dest.trim()).file_name()?.to_str()?.to_string();
            return Some(ProgressEvent::Destination(name));
        }
        let mut words = after.split_whitespace();
        let percent = words.next()?.strip_suffix('%')?.parse::<f32>().ok()?;
        let mut speed = None;
        let mut prev = "";
        for w in after.split_whitespace() {
            if prev == "at" {
                speed = Some(w.to_string()).filter(|s| s != "Unknown");
                break;
            }
            prev = w;
        }
        return Some(ProgressEvent::Percent { percent, speed });
    }

    // Postprocessor tags are CamelCase ([ExtractAudio], [Metadata]); extractor
    // tags ([youtube], [info]) are lowercase and carry no progress meaning.
    if tag.chars().next()?.is_ascii_uppercase() {
        if let Some(dest) = after.strip_prefix("Destination:") {
            let name = Path::new(dest.trim()).file_name()?.to_str()?.to_string();
            return Some(ProgressEvent::Destination(name));
        }
        return Some(ProgressEvent::Stage(tag.to_string()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_args_parse_and_keep_quoted_postprocessor_arg() {
        let dir = PathBuf::from("/tmp/out dir");
        let args = build_args(DEFAULT_ARGS, &dir, "https://youtu.be/x").unwrap();
        assert!(args.contains(&"ExtractAudio:-af volume=1.5".to_string()));
        assert!(args.contains(&"%(title)s.%(ext)s".to_string()));
        assert!(args.contains(&"--newline".to_string()));
        // app-owned tail: -P <dir> -- <url>
        let p = args.iter().position(|a| a == "-P").unwrap();
        assert_eq!(args[p + 1], "/tmp/out dir");
        assert_eq!(args[args.len() - 2], "--");
        assert_eq!(args[args.len() - 1], "https://youtu.be/x");
    }

    #[test]
    fn rippable_host_matches_known_sites_and_subdomains() {
        assert!(rippable_host("https://www.youtube.com/watch?v=x"));
        assert!(rippable_host("https://youtu.be/x"));
        assert!(rippable_host("https://m.youtube.com/watch?v=x"));
        assert!(rippable_host("http://vimeo.com/12345"));
        assert!(rippable_host("https://VIMEO.com/12345#frag"));
        assert!(rippable_host("https://user@twitch.tv/somechannel"));
    }

    #[test]
    fn rippable_host_rejects_articles_and_lookalikes() {
        assert!(!rippable_host("https://example.com/post"));
        assert!(!rippable_host("https://notyoutube.com/x"));
        assert!(!rippable_host("https://youtube.com.evil.com/x"));
        assert!(!rippable_host("ftp://youtube.com/x"));
        assert!(!rippable_host("https://"));
        assert!(!rippable_host(""));
    }

    #[test]
    fn build_args_rejects_bad_urls() {
        let dir = PathBuf::from("/tmp");
        for url in ["", "ftp://x", "-U", "file:///etc/passwd", "https://a b", "https://"] {
            assert!(build_args(DEFAULT_ARGS, &dir, url).is_err(), "{url}");
        }
    }

    #[test]
    fn build_args_rejects_unbalanced_quotes() {
        assert!(build_args("--foo \"bar", &PathBuf::from("/tmp"), "https://y.be/x").is_err());
    }

    #[test]
    fn parses_percent_and_speed() {
        let e = parse_line("[download]  42.3% of ~  12.34MiB at    1.23MiB/s ETA 00:12").unwrap();
        assert_eq!(
            e,
            ProgressEvent::Percent { percent: 42.3, speed: Some("1.23MiB/s".into()) }
        );
        let e = parse_line("[download] 100% of 12.34MiB in 00:00:10 at 1.2MiB/s").unwrap();
        assert_eq!(e, ProgressEvent::Percent { percent: 100.0, speed: Some("1.2MiB/s".into()) });
    }

    #[test]
    fn unknown_speed_reads_as_none() {
        let e = parse_line("[download]   0.0% of ~ 10MiB at Unknown B/s ETA Unknown").unwrap();
        assert_eq!(e, ProgressEvent::Percent { percent: 0.0, speed: None });
    }

    #[test]
    fn parses_destinations_from_download_and_postprocessor() {
        let e = parse_line("[download] Destination: /music/Some_Video.webm").unwrap();
        assert_eq!(e, ProgressEvent::Destination("Some_Video.webm".into()));
        let e = parse_line("[ExtractAudio] Destination: /music/Some_Video.mp3").unwrap();
        assert_eq!(e, ProgressEvent::Destination("Some_Video.mp3".into()));
    }

    #[test]
    fn parses_postprocessor_stages() {
        assert_eq!(parse_line("[EmbedThumbnail] ffmpeg: Adding thumbnail to \"x.mp3\"").unwrap(), ProgressEvent::Stage("EmbedThumbnail".into()));
        assert_eq!(parse_line("[Metadata] Adding metadata to \"x.mp3\"").unwrap(), ProgressEvent::Stage("Metadata".into()));
        assert_eq!(parse_line("[SponsorBlock] Found 2 segments in the SponsorBlock database").unwrap(), ProgressEvent::Stage("SponsorBlock".into()));
    }

    #[test]
    fn ignores_extractor_chatter_and_garbage() {
        assert!(parse_line("[youtube] dQw4: Downloading webpage").is_none());
        assert!(parse_line("[info] Available formats").is_none());
        assert!(parse_line("WARNING: something").is_none());
        assert!(parse_line("").is_none());
        assert!(parse_line("[download] Resuming download at byte 123").is_none());
    }
}
