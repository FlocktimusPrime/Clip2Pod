use clip2pod_core::config::Config;
use clip2pod_core::queue::Queue;
use clip2pod_core::voices::VoiceInfo;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

pub struct AppState {
    pub config_dir: PathBuf,
    pub config: Mutex<Config>,
    pub queue: Mutex<Queue>,
    /// Filtered narration catalog (English, non-cartoon).
    pub voices: Mutex<Vec<VoiceInfo>>,
    /// Wakes the TTS worker when jobs are enqueued.
    pub wake_worker: UnboundedSender<()>,
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

    pub fn save_config(&self) {
        let cfg = self.config.lock().unwrap();
        if let Err(e) = clip2pod_core::config::save_config(&self.config_dir, &cfg) {
            eprintln!("failed to save config: {e}");
        }
    }
}
