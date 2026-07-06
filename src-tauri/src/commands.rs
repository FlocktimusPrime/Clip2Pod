use crate::state::AppState;
use crate::worker::{emit_lamp, emit_queue, log_and_emit};
use clip2pod_core::config::{self, Config, LogEntry, LogStatus, Theme};
use clip2pod_core::junk::{self, JunkMatch};
use clip2pod_core::naming;
use clip2pod_core::queue::Job;
use clip2pod_core::textclean::{self, Autofill};
use clip2pod_core::voices::{self, AuthorGender, VoiceInfo};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};

type CmdResult<T> = Result<T, String>;

fn fallback_output_dir() -> PathBuf {
    dirs::audio_dir()
        .or_else(dirs::download_dir)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Serialize)]
pub struct CleanResult {
    pub cleaned: String,
    pub autofill: Autofill,
}

#[tauri::command]
pub fn clean_text(text: String) -> CleanResult {
    let cleaned = textclean::clean_for_tts(&text);
    let autofill = textclean::autofill_from(&cleaned);
    CleanResult { cleaned, autofill }
}

fn effective_junk(cfg: &Config) -> Vec<String> {
    cfg.custom_junk.clone().unwrap_or_else(junk::default_phrases)
}

#[tauri::command]
pub fn find_junk(state: State<AppState>, text: String, from_line: usize) -> Option<JunkMatch> {
    let phrases = effective_junk(&state.config.lock().unwrap());
    junk::find_next(&text, from_line, &phrases)
}

#[tauri::command]
pub fn get_junk_phrases(state: State<AppState>) -> Vec<String> {
    effective_junk(&state.config.lock().unwrap())
}

#[tauri::command]
pub fn set_junk_phrases(state: State<AppState>, phrases: Vec<String>) {
    let cleaned: Vec<String> = phrases
        .iter()
        .map(|p| p.trim().to_lowercase())
        .filter(|p| !p.is_empty())
        .collect();
    state.config.lock().unwrap().custom_junk =
        if junk::is_default(&cleaned) { None } else { Some(cleaned) };
    state.save_config();
}

#[derive(Serialize, Clone)]
pub struct VoicesView {
    pub voices: Vec<VoiceInfo>,
    pub enabled: Vec<String>,
    pub male_total: usize,
    pub female_total: usize,
    pub male_enabled: usize,
    pub female_enabled: usize,
}

fn voices_view(state: &AppState) -> VoicesView {
    let voices = state.voices.lock().unwrap().clone();
    let enabled = state.enabled_set();
    let is_male = |v: &&VoiceInfo| v.gender == voices::Gender::Male;
    VoicesView {
        male_total: voices.iter().filter(is_male).count(),
        female_total: voices.iter().filter(|v| !is_male(v)).count(),
        male_enabled: voices
            .iter()
            .filter(|v| is_male(v) && enabled.contains(&v.short_name))
            .count(),
        female_enabled: voices
            .iter()
            .filter(|v| !is_male(v) && enabled.contains(&v.short_name))
            .count(),
        enabled: enabled.into_iter().collect(),
        voices,
    }
}

fn emit_voices(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _ = app.emit("voices-changed", voices_view(&state));
}

/// Fetch (or refresh) the catalog. Falls back to the config cache when the
/// network is down; errors only when both are empty.
#[tauri::command]
pub async fn list_voices(app: AppHandle, refresh: bool) -> CmdResult<VoicesView> {
    let need_fetch = {
        let state = app.state::<AppState>();
        let empty = state.voices.lock().unwrap().is_empty();
        refresh || empty
    };
    if need_fetch {
        let fetched = tauri::async_runtime::spawn_blocking(clip2pod_core::tts::fetch_voices)
            .await
            .map_err(|e| e.to_string())?;
        let state = app.state::<AppState>();
        match fetched {
            Ok(all) => {
                let filtered = voices::filter_narration_voices(all);
                *state.voices.lock().unwrap() = filtered.clone();
                state.config.lock().unwrap().cached_voices = filtered;
                state.save_config();
            }
            Err(e) => {
                let cached = state.config.lock().unwrap().cached_voices.clone();
                if cached.is_empty() {
                    return Err(format!("could not fetch voices: {e}"));
                }
                *state.voices.lock().unwrap() = cached;
            }
        }
    }
    let state = app.state::<AppState>();
    Ok(voices_view(&state))
}

fn set_enabled(app: &AppHandle, mutate: impl FnOnce(&mut HashSet<String>, &[VoiceInfo])) {
    let state = app.state::<AppState>();
    let voices = state.voices.lock().unwrap().clone();
    let mut enabled = state.enabled_set();
    mutate(&mut enabled, &voices);
    state.config.lock().unwrap().enabled_voices = Some(enabled);
    state.save_config();
    emit_voices(app);
}

#[tauri::command]
pub fn set_voice_enabled(app: AppHandle, short_name: String, enabled: bool) {
    set_enabled(&app, |set, _| {
        if enabled {
            set.insert(short_name);
        } else {
            set.remove(&short_name);
        }
    });
}

#[tauri::command]
pub fn enable_all_voices(app: AppHandle) {
    set_enabled(&app, |set, voices| {
        set.extend(voices.iter().map(|v| v.short_name.clone()));
    });
}

#[tauri::command]
pub fn disable_all_voices(app: AppHandle) {
    set_enabled(&app, |set, _| set.clear());
}

/// Short audition sample; raw MP3 bytes for an <audio> blob in the frontend.
#[tauri::command]
pub async fn preview_voice(app: AppHandle, short_name: String) -> CmdResult<tauri::ipc::Response> {
    let voice = {
        let state = app.state::<AppState>();
        let voices = state.voices.lock().unwrap();
        voices
            .iter()
            .find(|v| v.short_name == short_name)
            .cloned()
            .ok_or_else(|| format!("unknown voice {short_name}"))?
    };
    let text = format!(
        "Hi, this is {}. This is how your articles will sound on Clip2Pod.",
        voice.short_name
    );
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        clip2pod_core::tts::synthesize_bytes(&voice.name, &text)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// Clean, pick a voice (gender preference + rotation), reserve a
/// collision-safe filename, queue, and wake the worker.
#[tauri::command]
pub fn enqueue_generate(
    app: AppHandle,
    text: String,
    title: String,
    author: String,
    filename_title: String,
    source_url: Option<String>,
) -> CmdResult<String> {
    do_enqueue(&app, text, title, author, filename_title, source_url)
}

/// Shared by the IPC command, the tray menu item, and the global hotkey.
pub fn do_enqueue(
    app: &AppHandle,
    text: String,
    title: String,
    author: String,
    filename_title: String,
    source_url: Option<String>,
) -> CmdResult<String> {
    let state = app.state::<AppState>();
    let cleaned = textclean::clean_for_tts(&text);
    if cleaned.trim().is_empty() {
        return Err("nothing to narrate: the script is empty after cleaning".into());
    }
    let narration = textclean::narration_text(&cleaned, &title, &author);

    let voices = state.voices.lock().unwrap().clone();
    if voices.is_empty() {
        return Err("voice list not loaded yet".into());
    }
    let enabled = state.enabled_set();

    let (voice, prefix, out_dir) = {
        let mut cfg = state.config.lock().unwrap();
        let voice = voices::pick_voice(&voices, &enabled, cfg.author_gender, &mut cfg.cycling)
            .ok_or("no enabled voice matches the author gender — enable more voices")?;
        (voice, cfg.prefix_c2p, cfg.output_dir.clone())
    };
    state.save_config(); // persist advanced cycling state

    let dir = out_dir.unwrap_or_else(fallback_output_dir);

    let (id, filename) = {
        let mut q = state.queue.lock().unwrap();
        let reserved = q.reserved_filenames();
        let filename = naming::output_filename(&dir, &filename_title, prefix, &reserved);
        let id = q.enqueue(&title, &author, &narration, voice.clone(), &filename, source_url);
        (id, filename)
    };

    log_and_emit(
        app,
        LogEntry {
            timestamp: chrono::Utc::now(),
            status: LogStatus::Queued,
            title,
            voice: voice.short_name,
            filename,
            detail: String::new(),
        },
    );
    emit_lamp(app);
    emit_queue(app);
    let _ = state.wake_worker.send(());
    Ok(id)
}

/// Fetch a web article and return its readable text for the editor.
#[tauri::command]
pub async fn extract_url(url: String) -> CmdResult<clip2pod_core::extract::Extracted> {
    tauri::async_runtime::spawn_blocking(move || clip2pod_core::extract::extract_url(&url))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_queue(state: State<AppState>) -> Vec<Job> {
    state.queue.lock().unwrap().jobs().to_vec()
}

#[tauri::command]
pub fn clear_pending(app: AppHandle) {
    let cancelled = {
        let state = app.state::<AppState>();
        let mut q = state.queue.lock().unwrap();
        q.clear_pending()
    };
    for job in cancelled {
        log_and_emit(
            &app,
            LogEntry {
                timestamp: chrono::Utc::now(),
                status: LogStatus::Cancelled,
                title: job.title,
                voice: job.voice.short_name,
                filename: job.filename,
                detail: "Cancelled".into(),
            },
        );
    }
    emit_lamp(&app);
    emit_queue(&app);
}

#[tauri::command]
pub fn get_log(state: State<AppState>, query: String) -> Vec<LogEntry> {
    let log = config::load_log(&state.config_dir);
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
pub fn clear_log(state: State<AppState>) -> CmdResult<()> {
    config::clear_log(&state.config_dir).map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct ConfigView {
    pub output_dir: String,
    pub prefix_c2p: bool,
    pub author_gender: AuthorGender,
    pub theme: Theme,
    pub global_hotkey: String,
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> ConfigView {
    let cfg = state.config.lock().unwrap();
    let dir = cfg.output_dir.clone().unwrap_or_else(fallback_output_dir);
    ConfigView {
        output_dir: dir.display().to_string(),
        prefix_c2p: cfg.prefix_c2p,
        author_gender: cfg.author_gender,
        theme: cfg.theme,
        global_hotkey: cfg.global_hotkey.clone(),
    }
}

#[tauri::command]
pub fn set_output_dir(state: State<AppState>, dir: String) {
    state.config.lock().unwrap().output_dir = Some(PathBuf::from(dir));
    state.save_config();
}

#[tauri::command]
pub fn set_prefix(state: State<AppState>, prefix: bool) {
    state.config.lock().unwrap().prefix_c2p = prefix;
    state.save_config();
}

#[tauri::command]
pub fn set_author_gender(state: State<AppState>, gender: AuthorGender) {
    state.config.lock().unwrap().author_gender = gender;
    state.save_config();
}

#[tauri::command]
pub fn set_theme(state: State<AppState>, theme: Theme) {
    state.config.lock().unwrap().theme = theme;
    state.save_config();
}

/// Subscribe URL for the LAN podcast feed, shown in the Feed dialog.
#[tauri::command]
pub fn feed_url() -> String {
    format!(
        "http://{}:{}/feed.xml",
        crate::feed::lan_ip(),
        crate::feed::FEED_PORT
    )
}
