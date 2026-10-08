use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

pub const LOG_CAP: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

/// All persisted settings. `output_dir: None` means "use the default
/// (~/Documents/Clip2Pod Feeds/Ripped)"; `args_template: None` means "use
/// DEFAULT_ARGS".
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub output_dir: Option<PathBuf>,
    pub args_template: Option<String>,
    /// yt-dlp executable. `None` means "use `yt-dlp` from PATH"; set an absolute
    /// path to pin a specific build (e.g. a nightly release).
    pub ytdlp_path: Option<String>,
    pub theme: Theme,
    /// Launch with the main window hidden — tray icon only.
    pub start_minimized: bool,
    /// None = user hasn't answered the first-run prompt yet.
    pub launch_at_startup: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogStatus {
    Queued,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub status: LogStatus,
    /// Video title once known, otherwise the URL.
    pub title: String,
    pub detail: String,
}

/// Platform config dir, e.g. %APPDATA%\yt-dlFeed or ~/.config/yt-dlFeed.
pub fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("yt-dlFeed")
}

/// Default episode folder: ~/Documents/Clip2Pod Feeds/Ripped (home, then the
/// current dir, if there is no Documents folder). Narrate's default is the
/// `Narrated` sibling.
pub fn default_output_dir() -> PathBuf {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Clip2Pod Feeds")
        .join("Ripped")
}

fn write_atomic(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

/// Load config from `dir/config.json`; missing or corrupt file yields defaults.
pub fn load_config(dir: &Path) -> Config {
    std::fs::read_to_string(dir.join("config.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_config(dir: &Path, config: &Config) -> io::Result<()> {
    let json = serde_json::to_string_pretty(config).map_err(io::Error::other)?;
    write_atomic(&dir.join("config.json"), &json)
}

pub fn load_log(dir: &Path) -> Vec<LogEntry> {
    std::fs::read_to_string(dir.join("log.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Append an entry, keeping only the newest LOG_CAP entries (newest first).
pub fn append_log(dir: &Path, entry: LogEntry) -> io::Result<()> {
    let mut log = load_log(dir);
    log.insert(0, entry);
    log.truncate(LOG_CAP);
    let json = serde_json::to_string(&log).map_err(io::Error::other)?;
    write_atomic(&dir.join("log.json"), &json)
}

pub fn clear_log(dir: &Path) -> io::Result<()> {
    write_atomic(&dir.join("log.json"), "[]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn entry(title: &str) -> LogEntry {
        LogEntry {
            timestamp: Utc::now(),
            status: LogStatus::Done,
            title: title.to_string(),
            detail: String::new(),
        }
    }

    #[test]
    fn missing_config_yields_defaults() {
        let dir = tempdir().unwrap();
        let c = load_config(dir.path());
        assert_eq!(c.theme, Theme::Dark);
        assert!(c.output_dir.is_none());
        assert!(c.args_template.is_none());
        assert!(c.ytdlp_path.is_none());
        assert!(!c.start_minimized);
    }

    #[test]
    fn corrupt_config_yields_defaults() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("config.json"), "{not json").unwrap();
        let c = load_config(dir.path());
        assert_eq!(c.theme, Theme::Dark);
    }

    #[test]
    fn config_roundtrip() {
        let dir = tempdir().unwrap();
        let mut c = Config::default();
        c.theme = Theme::Light;
        c.args_template = Some("--extract-audio".into());
        c.output_dir = Some(PathBuf::from("/tmp/pods"));
        c.ytdlp_path = Some("/opt/yt-dlp-nightly/yt-dlp".into());
        save_config(dir.path(), &c).unwrap();
        let loaded = load_config(dir.path());
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.args_template.as_deref(), Some("--extract-audio"));
        assert_eq!(loaded.output_dir.as_deref(), Some(Path::new("/tmp/pods")));
        assert_eq!(loaded.ytdlp_path.as_deref(), Some("/opt/yt-dlp-nightly/yt-dlp"));
    }

    #[test]
    fn log_appends_newest_first_and_caps() {
        let dir = tempdir().unwrap();
        for i in 0..(LOG_CAP + 5) {
            append_log(dir.path(), entry(&format!("t{i}"))).unwrap();
        }
        let log = load_log(dir.path());
        assert_eq!(log.len(), LOG_CAP);
        assert_eq!(log[0].title, format!("t{}", LOG_CAP + 4));
    }

    #[test]
    fn clear_log_empties() {
        let dir = tempdir().unwrap();
        append_log(dir.path(), entry("x")).unwrap();
        clear_log(dir.path()).unwrap();
        assert!(load_log(dir.path()).is_empty());
    }

    #[test]
    fn default_output_dir_ends_with_app_folder() {
        assert!(default_output_dir().ends_with("Clip2Pod Feeds/Ripped"));
    }
}
