use crate::mode::{Mode, Narrate};
use crate::state::AppState;
use clip2pod_core::queue::{Job, JobStatus};
use clip2pod_core::tts::TtsError;
use serde::Serialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::UnboundedReceiver;

/// Per-chunk render progress, pushed to the footer while a job encodes.
#[derive(Serialize, Clone)]
struct Progress {
    done: usize,
    total: usize,
}

/// Why a render stopped short.
enum RenderErr {
    /// User hit Cancel — the job lands as Cancelled, not Failed.
    Cancelled,
    Failed(String),
}

async fn render_job(app: &AppHandle, job: &Job) -> Result<(), RenderErr> {
    let state = app.state::<AppState>();
    let dir = state.output_dir();
    std::fs::create_dir_all(&dir).map_err(|e| RenderErr::Failed(e.to_string()))?;
    let out_path = dir.join(&job.filename);

    let voice_name = job.voice.name.clone();
    let text = job.text.clone();
    let path = out_path.clone();
    let cancel = state.cancel_flag.clone();
    let progress_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        clip2pod_core::tts::synthesize_to_file_cb(&voice_name, &text, &path, &cancel, |done, total| {
            let _ = progress_app.emit("render-progress", Progress { done, total });
        })
    })
    .await
    .map_err(|e| RenderErr::Failed(e.to_string()))?
    .map_err(|e| match e {
        TtsError::Cancelled => RenderErr::Cancelled,
        other => RenderErr::Failed(other.to_string()),
    })?;

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
                // Fresh cancel flag for this job; a stray Cancel from before
                // must not abort the one that just started.
                app.state::<AppState>().cancel_flag.store(false, Ordering::Relaxed);
                Narrate::refresh(&app);

                let result = render_job(&app, &job).await;

                let (status, detail) = match result {
                    Ok(()) => (JobStatus::Done, String::new()),
                    Err(RenderErr::Cancelled) => (JobStatus::Cancelled, "Cancelled".to_string()),
                    Err(RenderErr::Failed(e)) => (JobStatus::Failed, e),
                };
                let settled = {
                    let state = app.state::<AppState>();
                    let mut q = state.queue.lock().unwrap();
                    q.finish(&job.id, status, detail)
                };
                if let Some(entry) = settled.as_ref().and_then(Job::log_entry) {
                    Narrate::log_and_emit(&app, entry);
                }
                Narrate::refresh(&app);
            }
        }
    });
}
