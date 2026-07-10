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
}

/// In-memory job list. The Tauri layer wraps this in a mutex and drives a
/// single worker; all transition rules live here where they are testable.
#[derive(Debug, Default)]
pub struct Queue {
    jobs: Vec<Job>,
}

impl Queue {
    /// Enqueue a URL; rejects a duplicate that is still pending or running.
    pub fn enqueue(&mut self, url: &str) -> Result<String, String> {
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
        });
        Ok(id)
    }

    /// Oldest queued job, flipped to Processing; None when idle.
    pub fn start_next(&mut self) -> Option<Job> {
        let job = self.jobs.iter_mut().find(|j| j.status == JobStatus::Queued)?;
        job.status = JobStatus::Processing;
        job.started = Some(Utc::now());
        Some(job.clone())
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

    pub fn finish(&mut self, id: &str, result: Result<String, String>) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.finished = Some(Utc::now());
            job.progress = JobProgress::default();
            match result {
                Ok(note) => {
                    job.status = JobStatus::Done;
                    job.detail = note;
                }
                Err(detail) => {
                    job.status = JobStatus::Failed;
                    job.detail = detail;
                }
            }
        }
    }

    /// Cancel every Queued job; the Processing one (if any) is untouched.
    /// Returns the cancelled jobs.
    pub fn clear_pending(&mut self) -> Vec<Job> {
        let mut cancelled = Vec::new();
        for job in &mut self.jobs {
            if job.status == JobStatus::Queued {
                job.status = JobStatus::Cancelled;
                job.finished = Some(Utc::now());
                cancelled.push(job.clone());
            }
        }
        cancelled
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
        let queued = self.jobs.iter().filter(|j| j.status == JobStatus::Queued).count();
        (processing, queued)
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_three(q: &mut Queue) -> Vec<String> {
        (0..3)
            .map(|i| q.enqueue(&format!("https://youtu.be/v{i}")).unwrap())
            .collect()
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
        let id = q.enqueue("https://youtu.be/a").unwrap();
        assert!(q.enqueue("https://youtu.be/a").is_err());
        let j = q.start_next().unwrap();
        assert!(q.enqueue("https://youtu.be/a").is_err()); // processing
        q.finish(&j.id, Ok(String::new()));
        assert!(q.enqueue("https://youtu.be/a").is_ok());
        assert_ne!(q.jobs().last().unwrap().id, id);
    }

    #[test]
    fn finish_records_status_detail_and_clears_progress() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let j = q.start_next().unwrap();
        q.set_progress(&j.id, JobProgress { percent: Some(50.0), stage: None, speed: None });
        q.finish(&j.id, Err("network down".into()));
        let job = q.jobs().iter().find(|x| x.id == j.id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.detail, "network down");
        assert!(job.finished.is_some());
        assert!(job.progress.percent.is_none());
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
