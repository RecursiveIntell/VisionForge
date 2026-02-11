use anyhow::{Context, Result};
use rusqlite::Connection;
use std::sync::atomic::Ordering;

use crate::db;
use crate::state::AppState;
use crate::types::queue::{QueueJob, QueueJobStatus, QueuePriority};

/// Add a new job to the queue with a generated ID and pending status.
pub fn add_job(state: &AppState, mut job: QueueJob) -> Result<String> {
    if job.id.is_empty() {
        job.id = uuid::Uuid::new_v4().to_string();
    }
    job.status = QueueJobStatus::Pending;

    let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    db::queue::insert_job(&conn, &job)?;
    Ok(job.id)
}

/// Get all jobs sorted by status then priority then creation time.
pub fn get_all_jobs(state: &AppState) -> Result<Vec<QueueJob>> {
    let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    db::queue::list_jobs(&conn)
}

/// Change the priority of a pending job (used for drag-to-reorder).
pub fn reorder_job(state: &AppState, job_id: &str, new_priority: QueuePriority) -> Result<()> {
    let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;

    let job = db::queue::get_job(&conn, job_id)?
        .with_context(|| format!("Queue job {} not found", job_id))?;

    if job.status != QueueJobStatus::Pending {
        anyhow::bail!("Can only reorder pending jobs (job {} is {:?})", job_id, job.status);
    }

    db::queue::update_job_priority(&conn, job_id, &new_priority)
}

/// Cancel a pending job. No-op if already generating or terminal.
pub fn cancel_job(state: &AppState, job_id: &str) -> Result<()> {
    let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    db::queue::cancel_job(&conn, job_id)
}

/// Pause the queue — executor will finish the current job but won't start new ones.
pub fn pause_queue(state: &AppState) {
    state.queue_paused.store(true, Ordering::Relaxed);
}

/// Resume the queue — executor will start picking up pending jobs again.
pub fn resume_queue(state: &AppState) {
    state.queue_paused.store(false, Ordering::Relaxed);
}

/// Check if the queue is currently paused.
pub fn is_paused(state: &AppState) -> bool {
    state.queue_paused.load(Ordering::Relaxed)
}

/// Get the next pending job for the executor to process.
/// Returns None if queue is paused or no pending jobs.
pub fn next_pending_job(conn: &Connection) -> Result<Option<QueueJob>> {
    let jobs = db::queue::get_pending_jobs(conn)?;
    Ok(jobs.into_iter().next())
}

/// Mark a job as generating (sets started_at).
pub fn mark_generating(conn: &Connection, job_id: &str) -> Result<()> {
    db::queue::update_job_status(conn, job_id, &QueueJobStatus::Generating)
}

/// Mark a job as completed and link the result image.
pub fn mark_completed(conn: &Connection, job_id: &str, image_id: &str) -> Result<()> {
    db::queue::update_job_status(conn, job_id, &QueueJobStatus::Completed)?;
    db::queue::set_job_result_image(conn, job_id, image_id)
}

/// Mark a job as failed.
pub fn mark_failed(conn: &Connection, job_id: &str) -> Result<()> {
    db::queue::update_job_status(conn, job_id, &QueueJobStatus::Failed)
}

/// On app startup, requeue any jobs that were mid-generation when the app closed.
pub fn requeue_interrupted(conn: &Connection) -> Result<u32> {
    db::queue::requeue_interrupted_jobs(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::AppConfig;

    fn make_state() -> AppState {
        let conn = crate::db::open_memory_database().unwrap();
        AppState::new(conn, AppConfig::default())
    }

    fn make_job(positive: &str) -> QueueJob {
        QueueJob {
            id: String::new(),
            priority: QueuePriority::Normal,
            status: QueueJobStatus::Pending,
            positive_prompt: positive.to_string(),
            negative_prompt: "lowres".to_string(),
            settings_json: r#"{"steps":20}"#.to_string(),
            pipeline_log: None,
            original_idea: None,
            linked_comparison_id: None,
            created_at: None,
            started_at: None,
            completed_at: None,
            result_image_id: None,
        }
    }

    #[test]
    fn test_add_job_generates_id() {
        let state = make_state();
        let job = make_job("a cat");
        let id = add_job(&state, job).unwrap();
        assert!(!id.is_empty());

        let jobs = get_all_jobs(&state).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, id);
    }

    #[test]
    fn test_cancel_job() {
        let state = make_state();
        let id = add_job(&state, make_job("a cat")).unwrap();
        cancel_job(&state, &id).unwrap();

        let jobs = get_all_jobs(&state).unwrap();
        assert_eq!(jobs[0].status, QueueJobStatus::Cancelled);
    }

    #[test]
    fn test_reorder_job() {
        let state = make_state();
        let id = add_job(&state, make_job("a cat")).unwrap();
        reorder_job(&state, &id, QueuePriority::High).unwrap();

        let jobs = get_all_jobs(&state).unwrap();
        assert_eq!(jobs[0].priority, QueuePriority::High);
    }

    #[test]
    fn test_reorder_non_pending_fails() {
        let state = make_state();
        let id = add_job(&state, make_job("a cat")).unwrap();

        // Mark generating
        {
            let conn = state.db.lock().unwrap();
            mark_generating(&conn, &id).unwrap();
        }

        let err = reorder_job(&state, &id, QueuePriority::High);
        assert!(err.is_err());
    }

    #[test]
    fn test_pause_resume() {
        let state = make_state();
        assert!(!is_paused(&state));

        pause_queue(&state);
        assert!(is_paused(&state));

        resume_queue(&state);
        assert!(!is_paused(&state));
    }

    #[test]
    fn test_next_pending_job() {
        let state = make_state();
        add_job(&state, make_job("first")).unwrap();
        add_job(&state, make_job("second")).unwrap();

        let conn = state.db.lock().unwrap();
        let next = next_pending_job(&conn).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().positive_prompt, "first");
    }

    #[test]
    fn test_mark_completed_with_image() {
        let state = make_state();
        let job_id = add_job(&state, make_job("a cat")).unwrap();

        let conn = state.db.lock().unwrap();
        // Insert a test image to satisfy FK
        conn.execute(
            "INSERT INTO images (id, filename) VALUES ('img-1', 'test.png')",
            [],
        ).unwrap();

        mark_generating(&conn, &job_id).unwrap();
        mark_completed(&conn, &job_id, "img-1").unwrap();

        let job = db::queue::get_job(&conn, &job_id).unwrap().unwrap();
        assert_eq!(job.status, QueueJobStatus::Completed);
        assert_eq!(job.result_image_id.unwrap(), "img-1");
    }
}
