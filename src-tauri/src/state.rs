use clip2pod_core::config::Config;
use clip2pod_core::queue::Queue;
use clip2pod_core::voices::VoiceInfo;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedSender;

pub struct AppState {
    pub config_dir: PathBuf,
    pub config: Mutex<Config>,
    pub queue: Mutex<Queue>,
    /// Filtered narration catalog (English, non-cartoon).
    pub voices: Mutex<Vec<VoiceInfo>>,
    /// Wakes the TTS worker when jobs are enqueued.
    pub wake_worker: UnboundedSender<()>,
    /// Set by the Cancel command; the worker clears it before each job and the
    /// synth loop checks it between chunks.
    pub cancel_flag: Arc<AtomicBool>,

    // --- RIP mode (yt-dlp) ---
    /// `config_dir/rip/` — keeps the rip config.json and log.json out of the
    /// narrate ones. Both cores' load/save take a dir, so no code overlap.
    pub rip_config_dir: PathBuf,
    pub rip_config: Mutex<ytdlfeed_core::config::Config>,
    pub rip_queue: Mutex<ytdlfeed_core::queue::Queue>,
    /// Wakes the yt-dlp worker when jobs are enqueued.
    pub rip_wake: UnboundedSender<()>,
    /// (job id, child handle) for the in-flight yt-dlp process, if any. Shared
    /// between the worker (which waits on it) and the stop command (which kills
    /// it).
    pub rip_running: Mutex<Option<(String, Arc<Mutex<Child>>)>>,
}

impl AppState {
    /// Effective enabled set: explicit config selection, or the spec default
    /// (standard en-US voices) when the user never customized it.
    pub fn enabled_set(&self) -> HashSet<String> {
        let cfg = self.config.lock().unwrap();
        match &cfg.enabled_voices {
            Some(set) => set.clone(),
            None => {
                let voices = self.voices.lock().unwrap();
                clip2pod_core::voices::default_enabled(&voices)
            }
        }
    }

    /// Effective episode folder: configured dir, or the platform default.
    /// Takes the config lock — callers must not hold it (std Mutex is not
    /// reentrant).
    pub fn output_dir(&self) -> PathBuf {
        self.config
            .lock()
            .unwrap()
            .output_dir
            .clone()
            .unwrap_or_else(clip2pod_core::config::default_output_dir)
    }

    pub fn save_config(&self) {
        let cfg = self.config.lock().unwrap();
        if let Err(e) = clip2pod_core::config::save_config(&self.config_dir, &cfg) {
            eprintln!("failed to save config: {e}");
        }
    }

    /// Effective rip episode folder: configured dir, or ~/Music/yt-dlFeed.
    pub fn rip_output_dir(&self) -> PathBuf {
        self.rip_config
            .lock()
            .unwrap()
            .output_dir
            .clone()
            .unwrap_or_else(ytdlfeed_core::config::default_output_dir)
    }

    /// Effective yt-dlp args template: configured, or the bundled default.
    pub fn rip_args_template(&self) -> String {
        self.rip_config
            .lock()
            .unwrap()
            .args_template
            .clone()
            .unwrap_or_else(|| ytdlfeed_core::ytdlp::DEFAULT_ARGS.to_string())
    }

    /// yt-dlp executable to spawn: the configured path, or `yt-dlp` from PATH.
    pub fn rip_ytdlp_bin(&self) -> String {
        self.rip_config
            .lock()
            .unwrap()
            .ytdlp_path
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "yt-dlp".to_string())
    }

    pub fn save_rip_config(&self) {
        let cfg = self.rip_config.lock().unwrap();
        if let Err(e) = ytdlfeed_core::config::save_config(&self.rip_config_dir, &cfg) {
            eprintln!("failed to save rip config: {e}");
        }
    }
}
