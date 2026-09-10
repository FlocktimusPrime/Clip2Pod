// RIP mode background worker: drains the rip queue one URL at a time, spawning
// yt-dlp and streaming its `--newline` progress to the frontend. Ported from the
// standalone yt-dlFeed app; events are namespaced `rip:*` so they don't collide
// with the narrate worker's, and the yt-dlp binary is whatever the user
// configured (defaults to `yt-dlp` on PATH).

use crate::state::AppState;
use serde::Serialize;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::UnboundedReceiver;
use ytdlfeed_core::config::{append_log, LogEntry, LogStatus};
use ytdlfeed_core::queue::{Job, JobProgress};
use ytdlfeed_core::ytdlp::{build_args, parse_line, ProgressEvent};

#[derive(Serialize, Clone)]
#[serde(tag = "state", content = "queued")]
pub enum Lamp {
    Idle,
    Queued(usize),
    OnAir,
}

pub fn emit_lamp(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (processing, queued) = state.rip_queue.lock().unwrap().lamp();
    let lamp = if processing {
        Lamp::OnAir
    } else if queued > 0 {
        Lamp::Queued(queued)
    } else {
        Lamp::Idle
    };
    let _ = app.emit("rip:lamp", lamp);
}

pub fn emit_queue(app: &AppHandle) {
    let state = app.state::<AppState>();
    let jobs = state.rip_queue.lock().unwrap().jobs().to_vec();
    let _ = app.emit("rip:queue-changed", jobs);
}

pub fn log_and_emit(app: &AppHandle, entry: LogEntry) {
    let state = app.state::<AppState>();
    if let Err(e) = append_log(&state.rip_config_dir, entry.clone()) {
        eprintln!("failed to append rip log: {e}");
    }
    let _ = app.emit("rip:log-appended", entry);
}

#[derive(Serialize, Clone)]
struct ProgressPayload {
    id: String,
    progress: JobProgress,
}

fn set_and_emit_progress(app: &AppHandle, id: &str, progress: JobProgress) {
    let state = app.state::<AppState>();
    state
        .rip_queue
        .lock()
        .unwrap()
        .set_progress(id, progress.clone());
    let _ = app.emit(
        "rip:job-progress",
        ProgressPayload {
            id: id.to_string(),
            progress,
        },
    );
}

/// The AppImage runtime exports these pointing into the squashfs mount;
/// system yt-dlp (a Python script) and the ffmpeg it spawns must not
/// inherit them or Python aborts with "No module named 'encodings'".
const HOST_ENV_POISON: [&str; 4] = ["PYTHONHOME", "PYTHONPATH", "LD_LIBRARY_PATH", "LD_PRELOAD"];

fn ytdlp_command(bin: &str) -> Command {
    let mut cmd = Command::new(bin);
    for var in HOST_ENV_POISON {
        cmd.env_remove(var);
    }
    // Windows: without this, a console window pops open for every job.
    // ffmpeg spawned by yt-dlp inherits the hidden console too.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Run one yt-dlp job to completion, streaming progress into the queue and
/// to the frontend. Returns a human note on success ("" or "already
/// downloaded"), stderr tail on failure.
fn run_job(app: &AppHandle, job: &Job) -> Result<String, String> {
    let (dir, template, bin) = {
        let state = app.state::<AppState>();
        (
            state.rip_output_dir(),
            state.rip_args_template(),
            state.rip_ytdlp_bin(),
        )
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let args = build_args(&template, &dir, &job.url)?;

    let mut child = ytdlp_command(&bin)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("{bin} not found — install it and make sure it is on PATH")
            } else {
                format!("failed to start {bin}: {e}")
            }
        })?;

    // stderr drains on its own thread so a full pipe can't deadlock the child.
    let stderr = child.stderr.take().expect("piped stderr");
    let stderr_thread = std::thread::spawn(move || {
        let mut tail: Vec<String> = Vec::new();
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            tail.push(line);
            if tail.len() > 20 {
                tail.remove(0);
            }
        }
        tail
    });

    let stdout = child.stdout.take().expect("piped stdout");
    let child = std::sync::Arc::new(std::sync::Mutex::new(child));
    {
        let state = app.state::<AppState>();
        *state.rip_running.lock().unwrap() = Some((job.id.clone(), child.clone()));
    }

    let mut progress = JobProgress::default();
    let mut skipped = false;
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
        if line.contains("has already been downloaded") {
            skipped = true;
        }
        match parse_line(&line) {
            Some(ProgressEvent::Percent { percent, speed }) => {
                // throttle: whole-percent steps are plenty for a progress bar
                let step_changed = progress
                    .percent
                    .is_none_or(|p| (percent - p).abs() >= 1.0 || percent >= 100.0 && p < 100.0);
                progress.percent = Some(percent);
                progress.speed = speed;
                progress.stage = Some("Downloading".into());
                if step_changed {
                    set_and_emit_progress(app, &job.id, progress.clone());
                }
            }
            Some(ProgressEvent::Stage(stage)) => {
                progress.stage = Some(stage);
                progress.speed = None;
                set_and_emit_progress(app, &job.id, progress.clone());
            }
            Some(ProgressEvent::Destination(name)) => {
                let state = app.state::<AppState>();
                state.rip_queue.lock().unwrap().set_filename(&job.id, &name);
                emit_queue(app);
            }
            None => {}
        }
    }

    let status = child.lock().unwrap().wait().map_err(|e| e.to_string())?;
    {
        let state = app.state::<AppState>();
        let mut running = state.rip_running.lock().unwrap();
        if running.as_ref().is_some_and(|(id, _)| id == &job.id) {
            *running = None;
        }
    }
    let stderr_tail = stderr_thread.join().unwrap_or_default();
    if !status.success() {
        let detail = if stderr_tail.is_empty() {
            format!("yt-dlp exited with {status}")
        } else {
            stderr_tail.join("\n")
        };
        return Err(detail);
    }

    // yt-dlp doesn't write TLEN; measure so the feed carries durations.
    ytdlfeed_core::feed::ensure_durations(&dir);
    Ok(if skipped {
        "already downloaded".into()
    } else {
        String::new()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ytdlp_command_strips_appimage_env() {
        let cmd = ytdlp_command("yt-dlp");
        let removed: Vec<_> = cmd
            .get_envs()
            .filter(|(_, v)| v.is_none())
            .map(|(k, _)| k.to_str().unwrap().to_string())
            .collect();
        for var in HOST_ENV_POISON {
            assert!(removed.contains(&var.to_string()), "{var} not removed");
        }
    }
}

/// Single background worker: drains the queue one job at a time, sleeping
/// on the channel until an enqueue wakes it.
pub fn spawn(app: AppHandle, mut wake: UnboundedReceiver<()>) {
    tauri::async_runtime::spawn(async move {
        while wake.recv().await.is_some() {
            loop {
                let job = {
                    let state = app.state::<AppState>();
                    let mut q = state.rip_queue.lock().unwrap();
                    q.start_next()
                };
                let Some(job) = job else { break };
                emit_lamp(&app);
                emit_queue(&app);

                let run_app = app.clone();
                let run_job_ref = job.clone();
                let result = tauri::async_runtime::spawn_blocking(move || {
                    run_job(&run_app, &run_job_ref)
                })
                .await
                .unwrap_or_else(|e| Err(e.to_string()));

                let (title, note) = {
                    let state = app.state::<AppState>();
                    let mut q = state.rip_queue.lock().unwrap();
                    q.finish(&job.id, result.clone());
                    let j = q.jobs().iter().find(|j| j.id == job.id).cloned();
                    (
                        j.and_then(|j| j.filename).unwrap_or_else(|| job.url.clone()),
                        result,
                    )
                };
                let (status, detail) = match note {
                    Ok(n) => (LogStatus::Done, n),
                    Err(e) => (LogStatus::Failed, e),
                };
                log_and_emit(
                    &app,
                    LogEntry {
                        timestamp: chrono::Utc::now(),
                        status,
                        title,
                        detail,
                    },
                );
                emit_lamp(&app);
                emit_queue(&app);
            }
        }
    });
}
