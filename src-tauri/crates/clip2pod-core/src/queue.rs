use crate::voices::VoiceInfo;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub title: String,
    pub author: String,
    pub text: String,
    pub voice: VoiceInfo,
    pub filename: String,
    #[serde(default)]
    pub source_url: Option<String>,
    pub status: JobStatus,
    pub created: DateTime<Utc>,
    pub started: Option<DateTime<Utc>>,
    pub finished: Option<DateTime<Utc>>,
    pub detail: String,
}

/// In-memory job list. The Tauri layer wraps this in a mutex and drives a
/// single worker; all transition rules live here where they are testable.
#[derive(Debug, Default)]
pub struct Queue {
    jobs: Vec<Job>,
}

impl Queue {
    pub fn enqueue(
        &mut self,
        title: &str,
        author: &str,
        text: &str,
        voice: VoiceInfo,
        filename: &str,
        source_url: Option<String>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.jobs.push(Job {
            id: id.clone(),
            title: title.to_string(),
            author: author.to_string(),
            text: text.to_string(),
            voice,
            filename: filename.to_string(),
            source_url,
            status: JobStatus::Queued,
            created: Utc::now(),
            started: None,
            finished: None,
            detail: String::new(),
        });
        id
    }

    /// Oldest queued job, flipped to Processing; None when idle.
    pub fn start_next(&mut self) -> Option<Job> {
        let job = self.jobs.iter_mut().find(|j| j.status == JobStatus::Queued)?;
        job.status = JobStatus::Processing;
        job.started = Some(Utc::now());
        Some(job.clone())
    }

    /// Record a terminal state for the job. `status` is one of Done / Failed /
    /// Cancelled; `detail` carries the error or cancellation note (empty for
    /// Done).
    pub fn finish(&mut self, id: &str, status: JobStatus, detail: String) {
        if let Some(job) = self.jobs.iter_mut().find(|j| j.id == id) {
            job.finished = Some(Utc::now());
            job.status = status;
            job.detail = detail;
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

    /// Filenames claimed by jobs that will still write output.
    pub fn reserved_filenames(&self) -> Vec<String> {
        self.jobs
            .iter()
            .filter(|j| matches!(j.status, JobStatus::Queued | JobStatus::Processing))
            .map(|j| j.filename.clone())
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
    use crate::voices::Gender;

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

    fn setup_three(q: &mut Queue) -> Vec<String> {
        (0..3)
            .map(|i| q.enqueue(&format!("t{i}"), "a", "text", voice(), &format!("t{i}.mp3"), None))
            .collect()
    }

    #[test]
    fn fifo_processing_order() {
        let mut q = Queue::default();
        let ids = setup_three(&mut q);
        let j1 = q.start_next().unwrap();
        assert_eq!(j1.id, ids[0]);
        assert_eq!(j1.status, JobStatus::Processing);
        q.finish(&j1.id, JobStatus::Done, String::new());
        let j2 = q.start_next().unwrap();
        assert_eq!(j2.id, ids[1]);
    }

    #[test]
    fn finish_records_status_and_detail() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let j = q.start_next().unwrap();
        q.finish(&j.id, JobStatus::Failed, "network down".into());
        let job = q.jobs().iter().find(|x| x.id == j.id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
        assert_eq!(job.detail, "network down");
        assert!(job.finished.is_some());
    }

    #[test]
    fn finish_can_mark_cancelled() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let j = q.start_next().unwrap();
        q.finish(&j.id, JobStatus::Cancelled, "Cancelled".into());
        let job = q.jobs().iter().find(|x| x.id == j.id).unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
        assert!(job.finished.is_some());
    }

    #[test]
    fn clear_pending_spares_the_processing_job() {
        let mut q = Queue::default();
        let ids = setup_three(&mut q);
        let processing = q.start_next().unwrap();
        let cancelled = q.clear_pending();
        assert_eq!(cancelled.len(), 2);
        assert!(cancelled.iter().all(|j| j.status == JobStatus::Cancelled));
        let still = q.jobs().iter().find(|j| j.id == processing.id).unwrap();
        assert_eq!(still.status, JobStatus::Processing);
        assert!(!cancelled.iter().any(|j| j.id == ids[0]));
    }

    #[test]
    fn reserved_filenames_cover_queued_and_processing_only() {
        let mut q = Queue::default();
        setup_three(&mut q);
        let j = q.start_next().unwrap();
        q.finish(&j.id, JobStatus::Done, String::new());
        q.start_next().unwrap();
        let reserved = q.reserved_filenames();
        assert_eq!(reserved, vec!["t1.mp3".to_string(), "t2.mp3".to_string()]);
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

    #[test]
    fn enqueue_records_source_url() {
        let mut q = Queue::default();
        let id = q.enqueue(
            "t",
            "a",
            "text",
            voice(),
            "t.mp3",
            Some("https://example.com/a".into()),
        );
        let job = q.jobs().iter().find(|j| j.id == id).unwrap();
        assert_eq!(job.source_url.as_deref(), Some("https://example.com/a"));
    }
}
