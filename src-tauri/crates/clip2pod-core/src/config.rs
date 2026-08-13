use crate::voices::{AuthorGender, CyclingState, VoiceInfo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

/// All persisted settings. `enabled_voices: None` and `custom_junk: None`
/// mean "using defaults" (computed from the voice list / junk defaults).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub output_dir: Option<PathBuf>,
    pub prefix_c2p: bool,
    pub author_gender: AuthorGender,
    pub enabled_voices: Option<HashSet<String>>,
    pub custom_junk: Option<Vec<String>>,
    pub theme: Theme,
    pub cycling: CyclingState,
    pub cached_voices: Vec<VoiceInfo>,
    pub start_minimized: bool,
    /// None = user hasn't answered the first-run prompt yet.
    pub launch_at_startup: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            output_dir: None,
            prefix_c2p: false,
            author_gender: AuthorGender::Unknown,
            enabled_voices: None,
            custom_junk: None,
            theme: Theme::Dark,
            cycling: CyclingState::default(),
            cached_voices: Vec::new(),
            start_minimized: false,
            launch_at_startup: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogStatus {
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub status: LogStatus,
    pub title: String,
    pub voice: String,
    pub filename: String,
    pub detail: String,
}

/// Platform config dir, e.g. %APPDATA%\Clip2Pod2 or ~/.config/Clip2Pod2.
pub fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Clip2Pod2")
}

/// Default episode folder when none is configured: ~/Music, falling back to
/// Downloads, home, then the current dir.
pub fn default_output_dir() -> PathBuf {
    dirs::audio_dir()
        .or_else(dirs::download_dir)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
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
            voice: "en-US-TestNeural".to_string(),
            filename: format!("{title}.mp3"),
            detail: String::new(),
        }
    }

    #[test]
    fn missing_config_yields_defaults() {
        let dir = tempdir().unwrap();
        let c = load_config(dir.path());
        assert_eq!(c.theme, Theme::Dark);
        assert!(c.enabled_voices.is_none());
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
        c.prefix_c2p = true;
        c.theme = Theme::Light;
        c.custom_junk = Some(vec!["sponsored".to_string()]);
        c.cycling.male_idx = 3;
        save_config(dir.path(), &c).unwrap();
        let loaded = load_config(dir.path());
        assert!(loaded.prefix_c2p);
        assert_eq!(loaded.theme, Theme::Light);
        assert_eq!(loaded.custom_junk.as_deref(), Some(&["sponsored".to_string()][..]));
        assert_eq!(loaded.cycling.male_idx, 3);
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
}
