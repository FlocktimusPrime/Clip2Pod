//! The two modes, NARRATE and RIP, behind one interface. Everything here used to
//! exist twice (worker.rs / rip_worker.rs, commands.rs / rip_commands.rs) with
//! only the event prefix and config dir changed. A `Mode` supplies those
//! per-mode facts; the lamp, queue and log plumbing is written once below.
//!
//! Naming rule (see CLAUDE.md): narrate events keep bare names, rip events are
//! prefixed `rip:`. For events, `EVENT_PREFIX` is the only place that rule
//! lives.

use crate::state::AppState;
use serde::Serialize;
use std::io;
use std::path::Path;
use tauri::{AppHandle, Emitter, Manager, Runtime};

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "state", content = "queued")]
pub enum Lamp {
    Idle,
    Queued(usize),
    OnAir,
}

impl Lamp {
    /// From a queue's `lamp()` counts: (is_processing, queued_count).
    pub fn from_counts((processing, queued): (bool, usize)) -> Self {
        if processing {
            Lamp::OnAir
        } else if queued > 0 {
            Lamp::Queued(queued)
        } else {
            Lamp::Idle
        }
    }
}

pub trait Mode {
    /// Prepended to every event this mode emits: "" (narrate) or "rip:".
    const EVENT_PREFIX: &'static str;
    type Job: Serialize + Clone;
    type LogEntry: Serialize + Clone;

    /// (is_processing, queued_count) from this mode's queue.
    fn lamp_counts(state: &AppState) -> (bool, usize);
    fn jobs(state: &AppState) -> Vec<Self::Job>;
    /// Cancel every Queued job and return the log rows for them.
    fn cancel_queued(state: &AppState) -> Vec<Self::LogEntry>;
    /// Directory holding this mode's log.json.
    fn log_dir(state: &AppState) -> &Path;
    fn append_log(dir: &Path, entry: Self::LogEntry) -> io::Result<()>;
    fn load_log(dir: &Path) -> Vec<Self::LogEntry>;
    fn clear_log_file(dir: &Path) -> io::Result<()>;
    fn log_title(entry: &Self::LogEntry) -> &str;
    /// Nudge this mode's worker to drain its queue.
    fn wake(state: &AppState);

    /// Full event name for this mode, e.g. `event("lamp")` → "rip:lamp".
    fn event(name: &str) -> String {
        format!("{}{name}", Self::EVENT_PREFIX)
    }

    fn emit_lamp<R: Runtime>(app: &AppHandle<R>) {
        let lamp = Lamp::from_counts(Self::lamp_counts(&app.state::<AppState>()));
        let _ = app.emit(&Self::event("lamp"), lamp);
    }

    fn emit_queue<R: Runtime>(app: &AppHandle<R>) {
        let jobs = Self::jobs(&app.state::<AppState>());
        let _ = app.emit(&Self::event("queue-changed"), jobs);
    }

    /// The queue changed: push the lamp and the job list to the UI.
    fn refresh<R: Runtime>(app: &AppHandle<R>) {
        Self::emit_lamp(app);
        Self::emit_queue(app);
    }

    /// Jobs were added (or re-added): refresh the UI and wake the worker.
    fn enqueued<R: Runtime>(app: &AppHandle<R>) {
        Self::refresh(app);
        Self::wake(&app.state::<AppState>());
    }

    /// Persist the row, then tell the UI. A write failure is reported but
    /// doesn't stop the event: the UI still shows the outcome.
    fn log_and_emit<R: Runtime>(app: &AppHandle<R>, entry: Self::LogEntry) {
        let state = app.state::<AppState>();
        let dir = Self::log_dir(&state);
        if let Err(e) = Self::append_log(dir, entry.clone()) {
            eprintln!("failed to append log in {}: {e}", dir.display());
        }
        let _ = app.emit(&Self::event("log-appended"), entry);
    }

    /// The log, newest first, filtered to titles containing `query`
    /// (case-insensitive); everything when `query` is blank.
    fn query_log(state: &AppState, query: &str) -> Vec<Self::LogEntry> {
        let log = Self::load_log(Self::log_dir(state));
        if query.trim().is_empty() {
            return log;
        }
        let q = query.to_lowercase();
        log.into_iter()
            .filter(|e| Self::log_title(e).to_lowercase().contains(&q))
            .collect()
    }

    fn clear_log(state: &AppState) -> Result<(), String> {
        Self::clear_log_file(Self::log_dir(state)).map_err(|e| e.to_string())
    }

    /// Cancel every Queued job (the Processing one is untouched), log one
    /// Cancelled row per job, and refresh the lamp and queue.
    fn clear_pending<R: Runtime>(app: &AppHandle<R>) {
        let rows = Self::cancel_queued(&app.state::<AppState>());
        for row in rows {
            Self::log_and_emit(app, row);
        }
        Self::refresh(app);
    }
}

pub struct Narrate;

impl Mode for Narrate {
    const EVENT_PREFIX: &'static str = "";
    type Job = clip2pod_core::queue::Job;
    type LogEntry = clip2pod_core::config::LogEntry;

    fn lamp_counts(state: &AppState) -> (bool, usize) {
        state.queue.lock().unwrap().lamp()
    }
    fn jobs(state: &AppState) -> Vec<Self::Job> {
        state.queue.lock().unwrap().jobs().to_vec()
    }
    fn cancel_queued(state: &AppState) -> Vec<Self::LogEntry> {
        let cancelled = state.queue.lock().unwrap().clear_pending();
        cancelled.iter().filter_map(Self::Job::log_entry).collect()
    }
    fn log_dir(state: &AppState) -> &Path {
        &state.config_dir
    }
    fn append_log(dir: &Path, entry: Self::LogEntry) -> io::Result<()> {
        clip2pod_core::config::append_log(dir, entry)
    }
    fn load_log(dir: &Path) -> Vec<Self::LogEntry> {
        clip2pod_core::config::load_log(dir)
    }
    fn clear_log_file(dir: &Path) -> io::Result<()> {
        clip2pod_core::config::clear_log(dir)
    }
    fn log_title(entry: &Self::LogEntry) -> &str {
        &entry.title
    }
    fn wake(state: &AppState) {
        let _ = state.wake_worker.send(());
    }
}

pub struct Rip;

impl Mode for Rip {
    const EVENT_PREFIX: &'static str = "rip:";
    type Job = clip2pod_rip::queue::Job;
    type LogEntry = clip2pod_rip::config::LogEntry;

    fn lamp_counts(state: &AppState) -> (bool, usize) {
        state.rip_queue.lock().unwrap().lamp()
    }
    fn jobs(state: &AppState) -> Vec<Self::Job> {
        state.rip_queue.lock().unwrap().jobs().to_vec()
    }
    fn cancel_queued(state: &AppState) -> Vec<Self::LogEntry> {
        let cancelled = state.rip_queue.lock().unwrap().clear_pending();
        cancelled.iter().filter_map(Self::Job::log_entry).collect()
    }
    fn log_dir(state: &AppState) -> &Path {
        &state.rip_config_dir
    }
    fn append_log(dir: &Path, entry: Self::LogEntry) -> io::Result<()> {
        clip2pod_rip::config::append_log(dir, entry)
    }
    fn load_log(dir: &Path) -> Vec<Self::LogEntry> {
        clip2pod_rip::config::load_log(dir)
    }
    fn clear_log_file(dir: &Path) -> io::Result<()> {
        clip2pod_rip::config::clear_log(dir)
    }
    fn log_title(entry: &Self::LogEntry) -> &str {
        &entry.title
    }
    fn wake(state: &AppState) {
        let _ = state.rip_wake.send(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clip2pod_core::voices::{Gender, VoiceInfo};
    use serde_json::{json, Value};
    use std::sync::{Arc, Mutex};
    use tauri::test::{mock_app, MockRuntime};
    use tauri::{App, Listener};

    const EVENTS: [&str; 6] = [
        "lamp",
        "queue-changed",
        "log-appended",
        "rip:lamp",
        "rip:queue-changed",
        "rip:log-appended",
    ];

    type Seen = Arc<Mutex<Vec<(String, Value)>>>;

    /// A mock app with a real AppState whose config dirs live in `dir`, and
    /// a recorder for every lamp/queue/log event of both modes.
    fn app_in(dir: &Path) -> (App<MockRuntime>, Seen) {
        let rip_dir = dir.join("rip");
        std::fs::create_dir_all(&rip_dir).unwrap();
        let (wake_worker, _) = tokio::sync::mpsc::unbounded_channel();
        let (rip_wake, _) = tokio::sync::mpsc::unbounded_channel();
        let app = mock_app();
        app.manage(AppState {
            config_dir: dir.to_path_buf(),
            config: Mutex::new(Default::default()),
            authors: Mutex::new(Vec::new()),
            queue: Mutex::new(Default::default()),
            voices: Mutex::new(Vec::new()),
            wake_worker,
            cancel_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            feed_token: Mutex::new(String::new()),
            feed_server: Mutex::new(None),
            capture_token: Mutex::new(String::new()),
            rip_config_dir: rip_dir,
            rip_config: Mutex::new(Default::default()),
            rip_queue: Mutex::new(Default::default()),
            rip_wake,
            rip_running: Mutex::new(None),
        });
        let seen: Seen = Arc::default();
        for name in EVENTS {
            let seen = seen.clone();
            app.listen_any(name, move |e| {
                let payload = serde_json::from_str(e.payload()).unwrap();
                seen.lock().unwrap().push((name.to_string(), payload));
            });
        }
        (app, seen)
    }

    fn names(seen: &Seen) -> Vec<String> {
        seen.lock()
            .unwrap()
            .iter()
            .map(|(n, _)| n.clone())
            .collect()
    }

    fn voice() -> VoiceInfo {
        VoiceInfo {
            name: "Microsoft Guy".into(),
            short_name: "en-US-GuyNeural".into(),
            gender: Gender::Male,
            locale: "en-US".into(),
            language: "en".into(),
            country: "US".into(),
            category: "General".into(),
        }
    }

    #[test]
    fn lamp_from_counts() {
        assert_eq!(Lamp::from_counts((false, 0)), Lamp::Idle);
        assert_eq!(Lamp::from_counts((false, 3)), Lamp::Queued(3));
        assert_eq!(Lamp::from_counts((true, 3)), Lamp::OnAir);
    }

    #[test]
    fn narrate_events_are_bare_and_rip_events_are_prefixed() {
        let dir = tempfile::tempdir().unwrap();
        let (app, seen) = app_in(dir.path());
        let handle = app.handle();

        Narrate::emit_lamp(handle);
        Narrate::emit_queue(handle);
        assert_eq!(names(&seen), ["lamp", "queue-changed"]);

        seen.lock().unwrap().clear();
        Rip::emit_lamp(handle);
        Rip::emit_queue(handle);
        assert_eq!(names(&seen), ["rip:lamp", "rip:queue-changed"]);
    }

    #[test]
    fn emitted_lamp_follows_the_queue() {
        let dir = tempfile::tempdir().unwrap();
        let (app, seen) = app_in(dir.path());
        let handle = app.handle();
        let state = handle.state::<AppState>();
        {
            let mut q = state.rip_queue.lock().unwrap();
            q.enqueue("https://youtu.be/a", None).unwrap();
            q.enqueue("https://youtu.be/b", None).unwrap();
        }
        Rip::emit_lamp(handle);
        state.rip_queue.lock().unwrap().start_next();
        Rip::emit_lamp(handle);

        let payloads: Vec<Value> = seen
            .lock()
            .unwrap()
            .iter()
            .map(|(_, p)| p.clone())
            .collect();
        assert_eq!(
            payloads,
            [
                json!({"state": "Queued", "queued": 2}),
                json!({"state": "OnAir"})
            ]
        );
    }

    #[test]
    fn rip_clear_pending_logs_and_persists_one_cancelled_row_per_job() {
        let dir = tempfile::tempdir().unwrap();
        let (app, seen) = app_in(dir.path());
        let handle = app.handle();
        let state = handle.state::<AppState>();
        {
            let mut q = state.rip_queue.lock().unwrap();
            q.enqueue("https://youtu.be/a", None).unwrap();
            q.enqueue("https://youtu.be/b", None).unwrap();
        }

        Rip::clear_pending(handle);

        assert_eq!(
            names(&seen),
            [
                "rip:log-appended",
                "rip:log-appended",
                "rip:lamp",
                "rip:queue-changed"
            ]
        );
        let first = seen.lock().unwrap()[0].1.clone();
        assert_eq!(first["status"], "Cancelled");
        assert_eq!(first["detail"], "Cancelled");
        assert_eq!(first["title"], "https://youtu.be/a");
        assert_eq!(seen.lock().unwrap()[2].1, json!({"state": "Idle"}));

        assert_eq!(Rip::query_log(&state, "").len(), 2);
        assert!(
            Narrate::query_log(&state, "").is_empty(),
            "narrate log untouched"
        );
    }

    #[test]
    fn narrate_clear_pending_logs_title_voice_and_filename() {
        let dir = tempfile::tempdir().unwrap();
        let (app, seen) = app_in(dir.path());
        let handle = app.handle();
        let state = handle.state::<AppState>();
        state
            .queue
            .lock()
            .unwrap()
            .enqueue("Talk", "a", "text", voice(), "Talk.mp3", None);

        Narrate::clear_pending(handle);

        assert_eq!(names(&seen), ["log-appended", "lamp", "queue-changed"]);
        let row = seen.lock().unwrap()[0].1.clone();
        assert_eq!(row["status"], "Cancelled");
        assert_eq!(row["title"], "Talk");
        assert_eq!(row["voice"], "en-US-GuyNeural");
        assert_eq!(row["filename"], "Talk.mp3");
        assert_eq!(Narrate::query_log(&state, "").len(), 1);
        assert!(Rip::query_log(&state, "").is_empty(), "rip log untouched");
    }

    #[test]
    fn query_log_filters_titles_case_insensitively_and_clear_empties_it() {
        let dir = tempfile::tempdir().unwrap();
        let (app, _) = app_in(dir.path());
        let handle = app.handle();
        for title in ["Big Talk", "Other"] {
            Rip::log_and_emit(
                handle,
                clip2pod_rip::config::LogEntry {
                    timestamp: chrono::Utc::now(),
                    status: clip2pod_rip::config::LogStatus::Done,
                    title: title.into(),
                    detail: String::new(),
                },
            );
        }
        let state = handle.state::<AppState>();
        let hits = Rip::query_log(&state, "TALK");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Big Talk");
        assert_eq!(Rip::query_log(&state, "   ").len(), 2);

        Rip::clear_log(&state).unwrap();
        assert!(Rip::query_log(&state, "").is_empty());
    }
}
