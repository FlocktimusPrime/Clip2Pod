use crate::config::{LogEntry, LogStatus};
use crate::cookies::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Processing,
    Done,
    Failed,
    Cancelled,
}

/// Live progress for a Processing job, updated from yt-dlp stdout.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobProgress {
    /// Download percent 0–100; None before the first progress line and
    /// during postprocessing.
    pub percent: Option<f32>,
    /// Human stage: "Downloading", "ExtractAudio", "EmbedThumbnail", …
    pub stage: Option<String>,
    pub speed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub url: String,
    /// Output filename once yt-dlp announces it; doubles as display title.
    pub filename: Option<String>,
    pub status: JobStatus,
    pub progress: JobProgress,
    pub created: DateTime<Utc>,
    pub started: Option<DateTime<Utc>>,
    pub finished: Option<DateTime<Utc>>,
    /// Failure detail (stderr tail) or completion note ("already downloaded").
    pub detail: String,
    /// Extension-supplied cookies for this run. Never serialized; handed to
    /// the runner once by `start_next`.
    #[serde(skip)]
    pub cookies: Option<CookieJar>,
}

impl Job {
    /// The log row for a settled job: title is the output filename once known,
    /// else the URL. None while the job is still Queued or Processing.
    pub fn log_entry(&self) -> Option<LogEntry> {
        let status = match self.status {
            JobStatus::Done => LogStatus::Done,
            JobStatus::Failed => LogStatus::Failed,
            JobStatus::Cancelled => LogStatus::Cancelled,
            JobStatus::Queued | JobStatus::Processing => return None,
        };
        Some(LogEntry {
            timestamp: self.finished.unwrap_or_else(Utc::now),
            status,
            title: self.filename.clone().unwrap_or_else(|| self.url.clone()),
            detail: self.detail.clone(),
        })
    }
}

/// In-memory job list. The Tauri layer wraps this in a mutex and drives a
/// single worker; all transition rules live here where they are testable.
#[derive(Debug, Default)]
pub struct Queue {
    jobs: Vec<Job>,
}

impl Queue {
    /// Enqueue a URL; rejects a duplicate that is still pending or running.
    pub fn enqueue(&mut self, url: &str, cookies: Option<CookieJar>) -> Result<String, String> {
        let pending = self
            .jobs
            .iter()
            .any(|j| j.url == url && matches!(j.status, JobStatus::Queued | JobStatus::Processing));
        if pending {
            return Err("already queued".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.jobs.push(Job {
            id: id.clone(),
            url: url.to_string(),
            filename: None,
            status: JobStatus::Queued,
            progress: JobProgress::default(),
            created: Utc::now(),
            started: None,
            finished: None,
            detail: String::new(),
            cookies,
        });
        Ok(id)
    }

    /// Oldest queued job, flipped to Processing; None when idle.
    pub fn start_next(&mut self) -> Option<Job> {
        let job = self
            .jobs
            .iter_mut()
            .find(|j| j.status == JobStatus::Queued)?;
        job.status = JobStatus::Processing;
        job.started = Some(Utc::now());
        let cookies = job.cookies.take();
        let mut handed = job.clone();
        handed.cookies = cookies;
        Some(handed)
    }

    pub fn set_progress(&mut self, id: &str, progress: JobProgress) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.progress = progress;
        }
    }

    pub fn set_filename(&mut self, id: &str, filename: &str) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.filename = Some(filename.to_string());
        }
    }

    /// Settle a run's outcome and return the job as it now stands, which is
    /// the only thing the caller should log. A failure after
    /// `cancel_processing` stays Cancelled: the killed process's non-zero exit
    /// is the stop, not a fault. A success still lands as Done, because the
    /// stop arrived too late and the file is on disk.
    pub fn finish(&mut self, id: &str, result: Result<String, String>) -> Option<Job> {
        let job = self.jobs.iter_mut().find(|j| j.id == id)?;
        match result {
            Err(_) if job.status == JobStatus::Cancelled => {}
            Ok(note) => {
                job.status = JobStatus::Done;
                job.detail = note;
                job.finished = Some(Utc::now());
                job.progress = JobProgress::default();
            }
            Err(detail) => {
                job.status = JobStatus::Failed;
                job.detail = detail;
                job.finished = Some(Utc::now());
                job.progress = JobProgress::default();
            }
        }
        Some(job.clone())
    }

    /// Cancel every Queued job; the Processing one (if any) is untouched.
    /// Returns the cancelled jobs.
    pub fn clear_pending(&mut self) -> Vec<Job> {
        let mut cancelled = Vec::new();
        for job in &mut self.jobs {
            if job.status == JobStatus::Queued {
                job.status = JobStatus::Cancelled;
                job.finished = Some(Utc::now());
                job.cookies = None;
                cancelled.push(job.clone());
            }
        }
        cancelled
    }

    /// Cancel the currently Processing job if its id matches. Call this before
    /// killing the subprocess so `finish` always sees Cancelled and absorbs
    /// the kill's non-zero exit. Logging is left to whoever calls `finish`.
    pub fn cancel_processing(&mut self, id: &str) -> Option<Job> {
        let job = self
            .jobs
            .iter_mut()
            .find(|j| j.id == id && j.status == JobStatus::Processing)?;
        job.status = JobStatus::Cancelled;
        job.finished = Some(Utc::now());
        job.detail = "Stopped by user".into();
        job.progress = JobProgress::default();
        Some(job.clone())
    }

    /// Filenames claimed by jobs still writing output (delete must skip them).
    pub fn reserved_filenames(&self) -> Vec<String> {
        self.jobs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Queued | JobStatus::Processing))
            .filter_map(|j| j.filename.clone())
            .collect()
    }

    /// (is_processing, queued_count) for the status lamp.
    pub fn lamp(&self) -> (bool, usize) {
        let processing = self.jobs.iter().any(|j| j.status == JobStatus::Processing);
        let queued = self
            .jobs
            .iter()
            .filter(|j| j.status == JobStatus::Queued)
            .count();
        (processing, queued)
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cookies::Cookie;

    fn setup_three(q: &mut Queue) -> Vec<String> {
        (0..3)
            .map(|i| q.enqueue(&format!("https://youtu.be/v{i}"), None).unwrap())
            .collect()
    }

    fn jar() -> CookieJar {
        CookieJar(vec![Cookie {
            domain: ".example.com".into(),
            host_only: false,
            path: "/".into(),
            secure: true,
            expiration_date: None,
            name: "sid".into(),
            value: "SECRET".into(),
        }])
    }

    #[test]
    fn cookies_are_not_serialized_or_debug_printed() {
        let mut q = Queue::default();
        q.enqueue("https://youtu.be/a", Some(jar())).unwrap();
        let json = serde_json::to_string(&q.jobs()[0]).unwrap();
        let dbg = format!("{:?}", q.jobs()[0]);
        assert!(!json.contains("SECRET"));
        assert!(!dbg.contains("SECRET"));
        assert!(dbg.contains("1 cookies"));
    }

    #[test]
    fn start_next_hands_cookies_to_the_runner_once() {
        let mut q = Queue::default();
        q.enqueue("https://youtu.be/a", Some(jar())).unwrap();
        let job = q.start_next().unwrap();
        assert_eq!(job.cookies.unwrap().0.len(), 1);
        assert!(q.jobs()[0].cookies.is_none());
    }

    #[test]
    fn fifo_processing_order() {
        let mut q = Queue::default();
        let ids = setup_three(&mut q);
        let j1 = q.start_next().unwrap();
        assert_eq!(j1.id, ids[0]);
        assert_eq!(j1.status, JobStatus::Processing);
        q.finish(&j1.id, Ok(String::new()));
        let j2 = q.start_next().unwrap();
        assert_eq!(j2.id, ids[1]);
    }

    #[test]
    fn duplicate_pending_url_rejected_but_finished_url_reenqueues() {
        let mut q = Queue::default();
        let id = q.enqueue("https://youtu.be/a", None).unwrap();
        assert!(q.enqueue("https://youtu.be/a", None).is_err());
        let j = q.start_next().unwrap();
        assert!(q.enqueue("https://youtu.be/a", None).is_err()); // processing
        q.finish(&j.id, Ok(String::new()));
        assert!(q.enqueue("https://youtu.be/a", None).is_ok());
        assert_ne!(q.jobs().last().unwrap().id, id);
    }

    #[test]
    fn finish_records_status_detail_and_clears_progress() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let j = q.start_next().unwrap();
        q.set_progress(
            &j.id,
            JobProgress {
                percent: Some(50.0),
                stage: None,
                speed: None,
            },
        );
        q.finish(&j.id, Err("network down".into()));
        let job = q.jobs().iter().find(|x| x.id == j.id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.detail, "network down");
        assert!(job.finished.is_some());
        assert!(job.progress.percent.is_none());
    }

    #[test]
    fn clear_pending_drops_cookies() {
        let mut q = Queue::default();
        q.enqueue("https://youtu.be/a", Some(jar())).unwrap();
        let cancelled = q.clear_pending();
        assert_eq!(cancelled.len(), 1);
        assert!(cancelled[0].cookies.is_none());
        assert!(q.jobs()[0].cookies.is_none());
    }

    #[test]
    fn clear_pending_spares_the_processing_job() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let processing = q.start_next().unwrap();
        let cancelled = q.clear_pending();
        assert_eq!(cancelled.len(), 2);
        assert!(cancelled.iter().all(|j| j.status == JobStatus::Cancelled));
        let still = q.jobs().iter().find(|j| j.id == processing.id).unwrap();
        assert_eq!(still.status, JobStatus::Processing);
    }

    #[test]
    fn cancel_processing_marks_job_cancelled() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let processing = q.start_next().unwrap();
        let cancelled = q.cancel_processing(&processing.id).unwrap();
        assert_eq!(cancelled.status, JobStatus::Cancelled);
        assert_eq!(cancelled.detail, "Stopped by user");
        let still = q.jobs().iter().find(|j| j.id == processing.id).unwrap();
        assert_eq!(still.status, JobStatus::Cancelled);
        // Queued jobs behind it are untouched.
        assert!(q.jobs().iter().any(|j| j.status == JobStatus::Queued));
    }

    #[test]
    fn cancel_processing_ignores_non_matching_or_non_processing_id() {
        let mut q = Queue::default();
        let ids = setup_three(&mut q);
        q.start_next().unwrap();
        assert!(q.cancel_processing(&ids[1]).is_none()); // still Queued, not Processing
        assert!(q.cancel_processing("no-such-id").is_none());
    }

    #[test]
    fn finish_does_not_overwrite_cancelled() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let processing = q.start_next().unwrap();
        q.cancel_processing(&processing.id);
        let settled = q
            .finish(&processing.id, Err("yt-dlp exited with signal".into()))
            .unwrap();
        assert_eq!(settled.status, JobStatus::Cancelled);
        assert_eq!(settled.detail, "Stopped by user");
        let job = q.jobs().iter().find(|j| j.id == processing.id).unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
    }

    #[test]
    fn finish_ok_after_cancel_settles_done() {
        // Stop clicked as yt-dlp finished: the file exists, so Done wins.
        let mut q = Queue::default();
        setup_three(&mut q);
        let processing = q.start_next().unwrap();
        q.cancel_processing(&processing.id);
        let settled = q
            .finish(&processing.id, Ok("already downloaded".into()))
            .unwrap();
        assert_eq!(settled.status, JobStatus::Done);
        assert_eq!(settled.detail, "already downloaded");
    }

    #[test]
    fn finish_unknown_id_returns_none() {
        let mut q = Queue::default();
        assert!(q.finish("no-such-id", Ok(String::new())).is_none());
    }

    #[test]
    fn stopped_job_logs_one_cancelled_row() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let processing = q.start_next().unwrap();
        q.cancel_processing(&processing.id);
        let entry = q
            .finish(&processing.id, Err("yt-dlp exited with signal".into()))
            .and_then(|j| j.log_entry())
            .unwrap();
        assert!(matches!(entry.status, LogStatus::Cancelled));
        assert_eq!(entry.detail, "Stopped by user");
        assert_eq!(entry.title, processing.url); // no filename announced yet
    }

    #[test]
    fn log_entry_maps_status_and_prefers_filename() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let j = q.start_next().unwrap();
        assert!(
            q.jobs()[0].log_entry().is_none(),
            "Processing is not loggable"
        );
        assert!(q.jobs()[1].log_entry().is_none(), "Queued is not loggable");
        q.set_filename(&j.id, "Talk.mp3");

        let done = q
            .finish(&j.id, Ok(String::new()))
            .unwrap()
            .log_entry()
            .unwrap();
        assert!(matches!(done.status, LogStatus::Done));
        assert_eq!(done.title, "Talk.mp3");

        let k = q.start_next().unwrap();
        let failed = q
            .finish(&k.id, Err("network down".into()))
            .unwrap()
            .log_entry()
            .unwrap();
        assert!(matches!(failed.status, LogStatus::Failed));
        assert_eq!(failed.detail, "network down");
        assert_eq!(failed.title, k.url);
    }

    #[test]
    fn reserved_filenames_cover_active_jobs_with_known_names() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let j = q.start_next().unwrap();
        q.set_filename(&j.id, "a.mp3");
        assert_eq!(q.reserved_filenames(), vec!["a.mp3".to_string()]);
        q.finish(&j.id, Ok(String::new()));
        assert!(q.reserved_filenames().is_empty()); // done job releases its name
    }

    #[test]
    fn lamp_reflects_queue_state() {
        let mut q = Queue::default();
        assert_eq!(q.lamp(), (false, 0));
        setup_three(&mut q);
        assert_eq!(q.lamp(), (false, 3));
        q.start_next().unwrap();
        assert_eq!(q.lamp(), (true, 2));
    }
}
