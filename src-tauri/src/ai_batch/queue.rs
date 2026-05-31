use std::collections::HashMap;
use std::sync::Mutex;

use super::types::*;

/// The in-memory batch queue state.
pub struct AiBatchQueue {
    jobs: Mutex<Vec<BatchJob>>,
    eta_data: Mutex<HashMap<EtaKey, EtaStats>>,
}

#[derive(Debug, Clone)]
struct EtaStats {
    total_ms: u64,
    count: u64,
}

impl EtaStats {
    fn avg_ms(&self) -> u64 {
        self.total_ms.checked_div(self.count).unwrap_or(0)
    }
}

impl AiBatchQueue {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(Vec::new()),
            eta_data: Mutex::new(HashMap::new()),
        }
    }

    /// Add a new batch job and perform model-aware reordering. Returns job ID.
    pub fn enqueue(&self, mut job: BatchJob) -> anyhow::Result<String> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

        if job.id.is_empty() {
            job.id = uuid::Uuid::new_v4().to_string();
        }
        job.status = BatchJobStatus::Queued;
        job.created_at = chrono::Utc::now().to_rfc3339();

        let job_id = job.id.clone();
        jobs.push(job);

        Self::reorder_queued_jobs(&mut jobs);
        Ok(job_id)
    }

    /// Reorder only queued jobs to group by model (minimizes GPU model swaps).
    fn reorder_queued_jobs(jobs: &mut [BatchJob]) {
        let queued_indices: Vec<usize> = jobs
            .iter()
            .enumerate()
            .filter(|(_, j)| j.status == BatchJobStatus::Queued)
            .map(|(i, _)| i)
            .collect();

        if queued_indices.len() < 2 {
            return;
        }

        let mut queued_jobs: Vec<BatchJob> =
            queued_indices.iter().map(|&i| jobs[i].clone()).collect();

        let original_order: Vec<String> = queued_jobs.iter().map(|j| j.id.clone()).collect();
        queued_jobs.sort_by(|a, b| a.model.cmp(&b.model));
        let new_order: Vec<String> = queued_jobs.iter().map(|j| j.id.clone()).collect();

        if original_order != new_order {
            for job in &mut queued_jobs {
                job.reordered = true;
                job.reorder_note =
                    Some("Reordered: grouping by model to minimize GPU swaps".to_string());
            }
            for (slot_idx, job) in queued_indices.iter().zip(queued_jobs) {
                jobs[*slot_idx] = job;
            }
        }
    }

    /// Get the next queued job.
    pub fn next_queued(&self) -> Option<BatchJob> {
        let jobs = self.jobs.lock().ok()?;
        jobs.iter()
            .find(|j| j.status == BatchJobStatus::Queued)
            .cloned()
    }

    /// Mark a job as running.
    pub fn mark_running(&self, job_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            job.status = BatchJobStatus::Running;
            job.started_at = Some(chrono::Utc::now().to_rfc3339());
        }
        Ok(())
    }

    /// Update a single item's status within a job.
    pub fn update_item(
        &self,
        job_id: &str,
        image_id: &str,
        status: BatchItemStatus,
        error: Option<String>,
        duration_ms: Option<u64>,
    ) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            if let Some(item) = job.items.iter_mut().find(|i| i.image_id == image_id) {
                // Capture data for ETA before mutating
                let should_record = status == BatchItemStatus::Completed && duration_ms.is_some();
                let eta_model = job.model.clone();
                let eta_op = job.op;
                let eta_bucket = SizeBucket::from_dimensions(item.width, item.height);

                item.status = status;
                item.error = error;
                item.duration_ms = duration_ms;

                if should_record {
                    let ms = duration_ms.unwrap();
                    drop(jobs); // Release jobs lock before eta lock
                    let key = EtaKey {
                        model: eta_model,
                        op: eta_op,
                        size_bucket: eta_bucket,
                    };
                    match self.eta_data.lock() {
                        Ok(mut eta) => {
                            let entry = eta.entry(key).or_insert(EtaStats {
                                total_ms: 0,
                                count: 0,
                            });
                            entry.total_ms += ms;
                            entry.count += 1;
                        }
                        Err(e) => {
                            eprintln!(
                                "[ai_batch] WARNING: Failed to update ETA stats (mutex poisoned): {}",
                                e
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Mark a job as completed (or completed-with-errors).
    pub fn mark_completed(&self, job_id: &str) -> anyhow::Result<Option<BatchCompletionSummary>> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            let failed = job
                .items
                .iter()
                .filter(|i| i.status == BatchItemStatus::Failed)
                .count();
            let succeeded = job
                .items
                .iter()
                .filter(|i| i.status == BatchItemStatus::Completed)
                .count();
            let skipped = job
                .items
                .iter()
                .filter(|i| i.status == BatchItemStatus::Cancelled)
                .count();

            job.status = if failed > 0 {
                BatchJobStatus::CompletedWithErrors
            } else {
                BatchJobStatus::Completed
            };
            job.completed_at = Some(chrono::Utc::now().to_rfc3339());

            let total_ms: u64 = job.items.iter().filter_map(|i| i.duration_ms).sum();
            let processed = succeeded + failed;
            let avg_ms = if processed > 0 {
                total_ms / processed as u64
            } else {
                0
            };

            return Ok(Some(BatchCompletionSummary {
                job_id: job.id.clone(),
                op: job.op,
                model: job.model.clone(),
                total: job.items.len(),
                succeeded,
                failed,
                skipped,
                total_duration_ms: total_ms,
                avg_duration_ms: avg_ms,
            }));
        }
        Ok(None)
    }

    /// Cancel a single item within a running job.
    pub fn cancel_item(&self, job_id: &str, image_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            if let Some(item) = job.items.iter_mut().find(|i| i.image_id == image_id) {
                if item.status == BatchItemStatus::Pending {
                    item.status = BatchItemStatus::Cancelled;
                }
            }
        }
        Ok(())
    }

    /// Cancel an entire batch job. Running items finish, pending items cancelled.
    pub fn cancel_job(&self, job_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            for item in &mut job.items {
                if item.status == BatchItemStatus::Pending {
                    item.status = BatchItemStatus::Cancelled;
                }
            }
            let any_running = job
                .items
                .iter()
                .any(|i| i.status == BatchItemStatus::Running);
            if !any_running {
                job.status = BatchJobStatus::Cancelled;
                job.completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }
        Ok(())
    }

    /// Retry all failed items in a job by resetting them to Pending.
    pub fn retry_failed(&self, job_id: &str) -> anyhow::Result<()> {
        let mut jobs = self.jobs.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
            let has_failed = job
                .items
                .iter()
                .any(|i| i.status == BatchItemStatus::Failed);
            if !has_failed {
                anyhow::bail!("No failed items to retry in job {}", job_id);
            }
            for item in &mut job.items {
                if item.status == BatchItemStatus::Failed {
                    item.status = BatchItemStatus::Pending;
                    item.error = None;
                    item.duration_ms = None;
                }
            }
            job.status = BatchJobStatus::Queued;
            job.completed_at = None;
            Self::reorder_queued_jobs(&mut jobs);
        }
        Ok(())
    }

    /// Get all jobs.
    pub fn list_jobs(&self) -> Vec<BatchJob> {
        self.jobs.lock().map(|j| j.clone()).unwrap_or_default()
    }

    /// Get a specific job by ID.
    pub fn get_job(&self, job_id: &str) -> Option<BatchJob> {
        self.jobs
            .lock()
            .ok()?
            .iter()
            .find(|j| j.id == job_id)
            .cloned()
    }

    /// Estimate remaining time for a job based on historical data.
    pub fn estimate_remaining_ms(&self, job_id: &str) -> Option<u64> {
        let jobs = self.jobs.lock().ok()?;
        let job = jobs.iter().find(|j| j.id == job_id)?;
        let eta_data = self.eta_data.lock().ok()?;

        let remaining: Vec<&BatchItem> = job
            .items
            .iter()
            .filter(|i| {
                i.status == BatchItemStatus::Pending || i.status == BatchItemStatus::Running
            })
            .collect();

        if remaining.is_empty() {
            return Some(0);
        }

        let mut total_estimate: u64 = 0;
        let mut has_data = false;

        for item in &remaining {
            let bucket = SizeBucket::from_dimensions(item.width, item.height);
            let key = EtaKey {
                model: job.model.clone(),
                op: job.op,
                size_bucket: bucket,
            };

            if let Some(stats) = eta_data.get(&key) {
                total_estimate += stats.avg_ms();
                has_data = true;
            } else {
                let fallback = EtaKey {
                    model: job.model.clone(),
                    op: job.op,
                    size_bucket: SizeBucket::Unknown,
                };
                if let Some(stats) = eta_data.get(&fallback) {
                    total_estimate += stats.avg_ms();
                    has_data = true;
                }
            }
        }

        if has_data {
            Some(total_estimate)
        } else {
            None
        }
    }

    /// Check if any batch job is currently running.
    pub fn has_running_job(&self) -> bool {
        self.jobs
            .lock()
            .map(|j| j.iter().any(|job| job.status == BatchJobStatus::Running))
            .unwrap_or(false)
    }

    /// Get the number of queued jobs.
    pub fn queued_count(&self) -> usize {
        self.jobs
            .lock()
            .map(|j| {
                j.iter()
                    .filter(|job| job.status == BatchJobStatus::Queued)
                    .count()
            })
            .unwrap_or(0)
    }
}

impl Default for AiBatchQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_job(model: &str, op: BatchOpKind, num_items: usize) -> BatchJob {
        let items: Vec<BatchItem> = (0..num_items)
            .map(|i| BatchItem {
                image_id: format!("img-{}", i),
                filename: format!("img-{}.png", i),
                status: BatchItemStatus::Pending,
                error: None,
                duration_ms: None,
                width: Some(512),
                height: Some(512),
            })
            .collect();
        BatchJob {
            id: String::new(),
            op,
            model: model.to_string(),
            overwrite_policy: OverwritePolicy::Skip,
            items,
            status: BatchJobStatus::Queued,
            created_at: String::new(),
            started_at: None,
            completed_at: None,
            reordered: false,
            reorder_note: None,
        }
    }

    #[test]
    fn test_enqueue_assigns_id() {
        let queue = AiBatchQueue::new();
        let job = make_test_job("llava:7b", BatchOpKind::Tag, 3);
        let id = queue.enqueue(job).unwrap();
        assert!(!id.is_empty());
    }

    #[test]
    fn test_next_queued() {
        let queue = AiBatchQueue::new();
        assert!(queue.next_queued().is_none());

        let job = make_test_job("llava:7b", BatchOpKind::Tag, 2);
        let id = queue.enqueue(job).unwrap();

        let next = queue.next_queued().unwrap();
        assert_eq!(next.id, id);
    }

    #[test]
    fn test_mark_running() {
        let queue = AiBatchQueue::new();
        let job = make_test_job("llava:7b", BatchOpKind::Tag, 1);
        let id = queue.enqueue(job).unwrap();

        queue.mark_running(&id).unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::Running);
        assert!(job.started_at.is_some());
    }

    #[test]
    fn test_update_item_and_complete() {
        let queue = AiBatchQueue::new();
        let job = make_test_job("llava:7b", BatchOpKind::Tag, 2);
        let id = queue.enqueue(job).unwrap();
        queue.mark_running(&id).unwrap();

        queue
            .update_item(&id, "img-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();
        queue
            .update_item(&id, "img-1", BatchItemStatus::Completed, None, Some(2000))
            .unwrap();

        let summary = queue.mark_completed(&id).unwrap().unwrap();
        assert_eq!(summary.succeeded, 2);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.total_duration_ms, 3000);
        assert_eq!(summary.avg_duration_ms, 1500);
    }

    #[test]
    fn test_completed_with_errors() {
        let queue = AiBatchQueue::new();
        let job = make_test_job("llava:7b", BatchOpKind::Tag, 2);
        let id = queue.enqueue(job).unwrap();
        queue.mark_running(&id).unwrap();

        queue
            .update_item(&id, "img-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();
        queue
            .update_item(
                &id,
                "img-1",
                BatchItemStatus::Failed,
                Some("timeout".to_string()),
                Some(5000),
            )
            .unwrap();

        let summary = queue.mark_completed(&id).unwrap().unwrap();
        assert_eq!(summary.succeeded, 1);
        assert_eq!(summary.failed, 1);

        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::CompletedWithErrors);
    }

    #[test]
    fn test_cancel_job() {
        let queue = AiBatchQueue::new();
        let job = make_test_job("llava:7b", BatchOpKind::Tag, 3);
        let id = queue.enqueue(job).unwrap();

        queue.cancel_job(&id).unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::Cancelled);
        assert!(job
            .items
            .iter()
            .all(|i| i.status == BatchItemStatus::Cancelled));
    }

    #[test]
    fn test_cancel_single_item() {
        let queue = AiBatchQueue::new();
        let job = make_test_job("llava:7b", BatchOpKind::Tag, 3);
        let id = queue.enqueue(job).unwrap();

        queue.cancel_item(&id, "img-1").unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.items[0].status, BatchItemStatus::Pending);
        assert_eq!(job.items[1].status, BatchItemStatus::Cancelled);
        assert_eq!(job.items[2].status, BatchItemStatus::Pending);
    }

    #[test]
    fn test_retry_failed() {
        let queue = AiBatchQueue::new();
        let job = make_test_job("llava:7b", BatchOpKind::Tag, 2);
        let id = queue.enqueue(job).unwrap();
        queue.mark_running(&id).unwrap();

        queue
            .update_item(&id, "img-0", BatchItemStatus::Completed, None, Some(1000))
            .unwrap();
        queue
            .update_item(
                &id,
                "img-1",
                BatchItemStatus::Failed,
                Some("err".to_string()),
                None,
            )
            .unwrap();
        queue.mark_completed(&id).unwrap();

        queue.retry_failed(&id).unwrap();
        let job = queue.get_job(&id).unwrap();
        assert_eq!(job.status, BatchJobStatus::Queued);
        assert_eq!(job.items[1].status, BatchItemStatus::Pending);
        assert!(job.items[1].error.is_none());
    }

    #[test]
    fn test_model_aware_reordering() {
        let queue = AiBatchQueue::new();
        // Enqueue jobs with alternating models
        let job_a = make_test_job("model-b", BatchOpKind::Tag, 1);
        let job_b = make_test_job("model-a", BatchOpKind::Caption, 1);
        let job_c = make_test_job("model-b", BatchOpKind::Caption, 1);

        let _id_a = queue.enqueue(job_a).unwrap();
        let _id_b = queue.enqueue(job_b).unwrap();
        let _id_c = queue.enqueue(job_c).unwrap();

        let jobs = queue.list_jobs();
        // After reordering by model: model-a first, then model-b jobs
        assert_eq!(jobs[0].model, "model-a");
        assert_eq!(jobs[1].model, "model-b");
        assert_eq!(jobs[2].model, "model-b");
    }

    #[test]
    fn test_list_and_count() {
        let queue = AiBatchQueue::new();
        assert_eq!(queue.queued_count(), 0);
        assert!(!queue.has_running_job());

        let job = make_test_job("llava:7b", BatchOpKind::Tag, 1);
        let id = queue.enqueue(job).unwrap();
        assert_eq!(queue.queued_count(), 1);

        queue.mark_running(&id).unwrap();
        assert!(queue.has_running_job());
        assert_eq!(queue.queued_count(), 0);
    }
}
