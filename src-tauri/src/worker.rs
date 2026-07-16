use crate::state::AppState;
use clip2pod_core::config::{append_log, LogEntry, LogStatus};
use clip2pod_core::queue::Job;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Serialize, Clone)]
#[serde(tag = "state", content = "queued")]
pub enum Lamp {
    Idle,
    Queued(usize),
    OnAir,
}

pub fn emit_lamp(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (processing, queued) = state.queue.lock().unwrap().lamp();
    let lamp = if processing {
        Lamp::OnAir
    } else if queued > 0 {
        Lamp::Queued(queued)
    } else {
        Lamp::Idle
    };
    let _ = app.emit("lamp", lamp);
}

pub fn emit_queue(app: &AppHandle) {
    let state = app.state::<AppState>();
    let jobs = state.queue.lock().unwrap().jobs().to_vec();
    let _ = app.emit("queue-changed", jobs);
}

pub fn log_and_emit(app: &AppHandle, entry: LogEntry) {
    let state = app.state::<AppState>();
    if let Err(e) = append_log(&state.config_dir, entry.clone()) {
        eprintln!("failed to append log: {e}");
    }
    let _ = app.emit("log-appended", entry);
}

async fn render_job(app: &AppHandle, job: &Job) -> Result<(), String> {
    let state = app.state::<AppState>();
    let dir = state.output_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let out_path = dir.join(&job.filename);

    let voice_name = job.voice.name.clone();
    let text = job.text.clone();
    let path = out_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        clip2pod_core::tts::synthesize_to_file(&voice_name, &text, &path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    // Tagging is best-effort per spec: keep the MP3 on failure.
    let summary = clip2pod_core::textclean::summary_snippet(&job.text);
    let duration_ms = clip2pod_core::tagging::mp3_duration_ms(&out_path);
    if let Err(e) = clip2pod_core::tagging::tag_mp3(
        &out_path,
        &job.title,
        &job.author,
        &job.voice.short_name,
        &summary,
        job.source_url.as_deref(),
        duration_ms,
    ) {
        eprintln!("tagging failed for {}: {e}", job.filename);
    }
    Ok(())
}

/// Single background worker: drains the queue one job at a time, sleeping
/// on the channel until an enqueue wakes it.
pub fn spawn(app: AppHandle, mut wake: UnboundedReceiver<()>) {
    tauri::async_runtime::spawn(async move {
        while wake.recv().await.is_some() {
            loop {
                let job = {
                    let state = app.state::<AppState>();
                    let mut q = state.queue.lock().unwrap();
                    q.start_next()
                };
                let Some(job) = job else { break };
                emit_lamp(&app);
                emit_queue(&app);

                let result = render_job(&app, &job).await;

                {
                    let state = app.state::<AppState>();
                    let mut q = state.queue.lock().unwrap();
                    q.finish(&job.id, result.clone());
                }
                let (status, detail) = match &result {
                    Ok(()) => (LogStatus::Done, String::new()),
                    Err(e) => (LogStatus::Failed, e.clone()),
                };
                log_and_emit(
                    &app,
                    LogEntry {
                        timestamp: chrono::Utc::now(),
                        status,
                        title: job.title.clone(),
                        voice: job.voice.short_name.clone(),
                        filename: job.filename.clone(),
                        detail,
                    },
                );
                emit_lamp(&app);
                emit_queue(&app);
            }
        }
    });
}
