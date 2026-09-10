// IPC commands for RIP mode. Ported from the standalone yt-dlFeed app; every
// command is prefixed `rip_` so it doesn't collide with the narrate commands,
// and every state access points at the rip slice of AppState. Theme / startup /
// firewall commands are dropped — the narrate side owns those app-wide.

use crate::rip_worker::{emit_lamp, emit_queue, log_and_emit};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use ytdlfeed_core::config::{self, LogEntry, LogStatus};
use ytdlfeed_core::queue::Job;
use ytdlfeed_core::ytdlp;

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

/// Validate the URL, queue it, wake the worker. Shared by the IPC command
/// and the capture listener.
pub fn do_enqueue(app: &AppHandle, url: String) -> CmdResult<String> {
    let url = url.trim().to_string();
    if !ytdlp::valid_url(&url) {
        return Err("not an http(s) URL".into());
    }
    let state = app.state::<AppState>();
    let id = state.rip_queue.lock().unwrap().enqueue(&url)?;
    log_and_emit(
        app,
        LogEntry {
            timestamp: chrono::Utc::now(),
            status: LogStatus::Queued,
            title: url,
            detail: String::new(),
        },
    );
    emit_lamp(app);
    emit_queue(app);
    let _ = state.rip_wake.send(());
    Ok(id)
}

#[tauri::command]
pub fn rip_enqueue(app: AppHandle, url: String) -> CmdResult<String> {
    do_enqueue(&app, url)
}

#[tauri::command]
pub fn rip_get_queue(state: State<AppState>) -> Vec<Job> {
    state.rip_queue.lock().unwrap().jobs().to_vec()
}

#[tauri::command]
pub fn rip_clear_pending(app: AppHandle) {
    let cancelled = {
        let state = app.state::<AppState>();
        let mut q = state.rip_queue.lock().unwrap();
        q.clear_pending()
    };
    for job in cancelled {
        log_and_emit(
            &app,
            LogEntry {
                timestamp: chrono::Utc::now(),
                status: LogStatus::Cancelled,
                title: job.url,
                detail: "Cancelled".into(),
            },
        );
    }
    emit_lamp(&app);
    emit_queue(&app);
}

/// Kill the yt-dlp subprocess for the given job if it's the one currently
/// Processing, and mark it Cancelled. No-op if the job already finished on its
/// own (lost the race with the click).
#[tauri::command]
pub fn rip_stop_job(app: AppHandle, job_id: String) {
    let child = {
        let state = app.state::<AppState>();
        let running = state.rip_running.lock().unwrap();
        running
            .as_ref()
            .filter(|(id, _)| id == &job_id)
            .map(|(_, c)| c.clone())
    };
    let Some(child) = child else { return };
    let _ = child.lock().unwrap().kill();

    let cancelled = {
        let state = app.state::<AppState>();
        let mut q = state.rip_queue.lock().unwrap();
        q.cancel_processing(&job_id)
    };
    if let Some(job) = cancelled {
        log_and_emit(
            &app,
            LogEntry {
                timestamp: chrono::Utc::now(),
                status: LogStatus::Cancelled,
                title: job.url,
                detail: "Stopped by user".into(),
            },
        );
    }
    emit_lamp(&app);
    emit_queue(&app);
}

#[tauri::command]
pub fn rip_get_log(state: State<AppState>, query: String) -> Vec<LogEntry> {
    let log = config::load_log(&state.rip_config_dir);
    if query.trim().is_empty() {
        log
    } else {
        let q = query.to_lowercase();
        log.into_iter()
            .filter(|e| e.title.to_lowercase().contains(&q))
            .collect()
    }
}

#[tauri::command]
pub fn rip_clear_log(state: State<AppState>) -> CmdResult<()> {
    config::clear_log(&state.rip_config_dir).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct RipConfigView {
    pub output_dir: String,
    pub args_template: String,
    pub default_args: &'static str,
    /// Configured yt-dlp binary, or "" meaning "yt-dlp from PATH".
    pub ytdlp_path: String,
}

#[tauri::command]
pub fn rip_get_config(state: State<AppState>) -> RipConfigView {
    let ytdlp_path = state
        .rip_config
        .lock()
        .unwrap()
        .ytdlp_path
        .clone()
        .unwrap_or_default();
    // output_dir()/args_template() take the config lock themselves — the guard
    // above is already released by here.
    RipConfigView {
        output_dir: state.rip_output_dir().display().to_string(),
        args_template: state.rip_args_template(),
        default_args: ytdlp::DEFAULT_ARGS,
        ytdlp_path,
    }
}

#[tauri::command]
pub fn rip_set_output_dir(state: State<AppState>, dir: String) {
    state.rip_config.lock().unwrap().output_dir = Some(PathBuf::from(dir));
    state.save_rip_config();
}

/// Save the yt-dlp args template; rejects templates that can't be split
/// (unbalanced quotes) so a broken template never reaches the worker.
#[tauri::command]
pub fn rip_set_args_template(state: State<AppState>, template: String) -> CmdResult<()> {
    let trimmed = template.trim().to_string();
    shell_words_check(&trimmed)?;
    state.rip_config.lock().unwrap().args_template = if trimmed == ytdlp::DEFAULT_ARGS {
        None
    } else {
        Some(trimmed)
    };
    state.save_rip_config();
    Ok(())
}

fn shell_words_check(template: &str) -> CmdResult<()> {
    // build_args validates the same way; a dummy URL exercises the split.
    ytdlp::build_args(template, std::path::Path::new("/"), "https://example.com/x").map(|_| ())
}

/// Set the yt-dlp executable path (e.g. a nightly build). Empty string resets to
/// "yt-dlp from PATH".
#[tauri::command]
pub fn rip_set_ytdlp_path(state: State<AppState>, path: String) {
    let trimmed = path.trim().to_string();
    state.rip_config.lock().unwrap().ytdlp_path =
        if trimmed.is_empty() { None } else { Some(trimmed) };
    state.save_rip_config();
}

/// Episode list for the UI (delete buttons live here).
#[derive(Serialize)]
pub struct EpisodeView {
    pub title: String,
    pub filename: String,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub artist: Option<String>,
    pub duration_ms: Option<u32>,
}

#[tauri::command]
pub fn rip_list_episodes(state: State<AppState>) -> Vec<EpisodeView> {
    ytdlfeed_core::feed::scan_episodes(&state.rip_output_dir())
        .into_iter()
        .map(|e| EpisodeView {
            title: e.title,
            filename: e.filename,
            size: e.size,
            modified: e.modified.into(),
            artist: e.artist,
            duration_ms: e.duration_ms,
        })
        .collect()
}

fn safe_episode_name(name: &str) -> CmdResult<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || !name.to_ascii_lowercase().ends_with(".mp3")
    {
        return Err("bad filename".into());
    }
    Ok(())
}

/// Delete one episode file; refuses names outside the flat mp3 namespace and
/// files reserved by an in-flight job.
#[tauri::command]
pub fn rip_delete_episode(state: State<AppState>, filename: String) -> CmdResult<()> {
    safe_episode_name(&filename)?;
    let reserved = state.rip_queue.lock().unwrap().reserved_filenames();
    if reserved.contains(&filename) {
        return Err("that file is still being written".into());
    }
    std::fs::remove_file(state.rip_output_dir().join(&filename)).map_err(|e| e.to_string())
}

/// Delete every mp3 the feed lists, except files reserved by queued or in-flight
/// jobs. `cover.jpg` is never a candidate (not an mp3).
#[tauri::command]
pub fn rip_delete_all_episodes(state: State<AppState>) -> CmdResult<usize> {
    let dir = state.rip_output_dir();
    let reserved: HashSet<String> = state
        .rip_queue
        .lock()
        .unwrap()
        .reserved_filenames()
        .into_iter()
        .collect();
    let mut deleted = 0;
    let mut failed = Vec::new();
    for episode in ytdlfeed_core::feed::scan_episodes(&dir) {
        if reserved.contains(&episode.filename) {
            continue;
        }
        match std::fs::remove_file(dir.join(&episode.filename)) {
            Ok(()) => deleted += 1,
            Err(_) => failed.push(episode.filename),
        }
    }
    if failed.is_empty() {
        Ok(deleted)
    } else {
        Err(format!(
            "deleted {deleted}, but could not delete: {}",
            failed.join(", ")
        ))
    }
}

/// Subscribe URL for the ripped-audio feed, shown in the Feed dialog.
#[tauri::command]
pub fn rip_feed_url() -> String {
    format!(
        "http://{}:{}/video/feed.xml",
        crate::feed::lan_ip(),
        crate::feed::FEED_PORT
    )
}

#[derive(Serialize)]
pub struct DoctorReport {
    pub ytdlp_found: bool,
    pub ytdlp_version: Option<String>,
    pub ffmpeg_found: bool,
}

/// Probe for the tools RIP mode needs so the UI can warn before a job fails.
#[tauri::command]
pub fn rip_doctor(state: State<AppState>) -> DoctorReport {
    let bin = state.rip_ytdlp_bin();
    let ytdlp_version = probe_version(&bin, "--version");
    let ffmpeg_found = probe_version("ffmpeg", "-version").is_some();
    DoctorReport {
        ytdlp_found: ytdlp_version.is_some(),
        ytdlp_version,
        ffmpeg_found,
    }
}

fn probe_version(bin: &str, flag: &str) -> Option<String> {
    let out = crate::rip_worker::ytdlp_command(bin).arg(flag).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}
