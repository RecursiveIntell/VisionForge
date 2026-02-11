use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::types::queue::{QueueJob, QueueJobStatus, QueuePriority};

pub fn insert_job(conn: &Connection, job: &QueueJob) -> Result<()> {
    conn.execute(
        "INSERT INTO queue_jobs (
            id, priority, status, positive_prompt, negative_prompt,
            settings_json, pipeline_log, original_idea, linked_comparison_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            job.id,
            job.priority.as_i32(),
            job.status.as_str(),
            job.positive_prompt,
            job.negative_prompt,
            job.settings_json,
            job.pipeline_log,
            job.original_idea,
            job.linked_comparison_id,
        ],
    )
    .context("Failed to insert queue job")?;
    Ok(())
}

pub fn get_job(conn: &Connection, id: &str) -> Result<Option<QueueJob>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, priority, status, positive_prompt, negative_prompt,
                    settings_json, pipeline_log, original_idea, linked_comparison_id,
                    created_at, started_at, completed_at, result_image_id
             FROM queue_jobs WHERE id = ?1",
        )
        .context("Failed to prepare get_job query")?;

    let mut rows = stmt
        .query_map(params![id], row_to_job)
        .context("Failed to execute get_job query")?;

    match rows.next() {
        Some(row) => Ok(Some(row.context("Failed to read job row")?)),
        None => Ok(None),
    }
}

pub fn list_jobs(conn: &Connection) -> Result<Vec<QueueJob>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, priority, status, positive_prompt, negative_prompt,
                    settings_json, pipeline_log, original_idea, linked_comparison_id,
                    created_at, started_at, completed_at, result_image_id
             FROM queue_jobs
             ORDER BY
                CASE status
                    WHEN 'generating' THEN 0
                    WHEN 'pending' THEN 1
                    WHEN 'completed' THEN 2
                    WHEN 'failed' THEN 3
                    WHEN 'cancelled' THEN 4
                END,
                priority ASC,
                created_at ASC",
        )
        .context("Failed to prepare list_jobs query")?;

    let rows = stmt
        .query_map([], row_to_job)
        .context("Failed to execute list_jobs query")?;

    let mut jobs = Vec::new();
    for row in rows {
        jobs.push(row.context("Failed to read job row")?);
    }
    Ok(jobs)
}

pub fn get_pending_jobs(conn: &Connection) -> Result<Vec<QueueJob>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, priority, status, positive_prompt, negative_prompt,
                    settings_json, pipeline_log, original_idea, linked_comparison_id,
                    created_at, started_at, completed_at, result_image_id
             FROM queue_jobs
             WHERE status = 'pending'
             ORDER BY priority ASC, created_at ASC",
        )
        .context("Failed to prepare get_pending_jobs query")?;

    let rows = stmt
        .query_map([], row_to_job)
        .context("Failed to execute get_pending_jobs query")?;

    let mut jobs = Vec::new();
    for row in rows {
        jobs.push(row.context("Failed to read job row")?);
    }
    Ok(jobs)
}

pub fn update_job_status(
    conn: &Connection,
    id: &str,
    status: &QueueJobStatus,
) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();

    match status {
        QueueJobStatus::Generating => {
            conn.execute(
                "UPDATE queue_jobs SET status = ?1, started_at = ?2 WHERE id = ?3",
                params![status.as_str(), now, id],
            )
        }
        QueueJobStatus::Completed | QueueJobStatus::Failed => {
            conn.execute(
                "UPDATE queue_jobs SET status = ?1, completed_at = ?2 WHERE id = ?3",
                params![status.as_str(), now, id],
            )
        }
        _ => {
            conn.execute(
                "UPDATE queue_jobs SET status = ?1 WHERE id = ?2",
                params![status.as_str(), id],
            )
        }
    }
    .context("Failed to update job status")?;
    Ok(())
}

pub fn set_job_result_image(conn: &Connection, job_id: &str, image_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE queue_jobs SET result_image_id = ?1 WHERE id = ?2",
        params![image_id, job_id],
    )
    .context("Failed to set job result image")?;
    Ok(())
}

pub fn update_job_priority(conn: &Connection, id: &str, priority: &QueuePriority) -> Result<()> {
    conn.execute(
        "UPDATE queue_jobs SET priority = ?1 WHERE id = ?2",
        params![priority.as_i32(), id],
    )
    .context("Failed to update job priority")?;
    Ok(())
}

pub fn cancel_job(conn: &Connection, id: &str) -> Result<()> {
    let rows = conn
        .execute(
            "UPDATE queue_jobs SET status = 'cancelled' WHERE id = ?1 AND status = 'pending'",
            params![id],
        )
        .context("Failed to cancel job")?;
    if rows == 0 {
        anyhow::bail!("Job '{}' not found or is not in pending status", id);
    }
    Ok(())
}

pub fn requeue_interrupted_jobs(conn: &Connection) -> Result<u32> {
    let count = conn
        .execute(
            "UPDATE queue_jobs SET status = 'pending'
             WHERE status = 'generating'",
            [],
        )
        .context("Failed to requeue interrupted jobs")?;
    Ok(count as u32)
}

fn row_to_job(row: &rusqlite::Row) -> rusqlite::Result<QueueJob> {
    let priority_val: i32 = row.get(1)?;
    let status_str: String = row.get(2)?;

    Ok(QueueJob {
        id: row.get(0)?,
        priority: QueuePriority::from_i32(priority_val),
        status: QueueJobStatus::from_str(&status_str).unwrap_or(QueueJobStatus::Pending),
        positive_prompt: row.get(3)?,
        negative_prompt: row.get(4)?,
        settings_json: row.get(5)?,
        pipeline_log: row.get(6)?,
        original_idea: row.get(7)?,
        linked_comparison_id: row.get(8)?,
        created_at: row.get(9)?,
        started_at: row.get(10)?,
        completed_at: row.get(11)?,
        result_image_id: row.get(12)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Connection { db::open_memory_database().unwrap() }

    fn make_job(id: &str, priority: QueuePriority) -> QueueJob {
        QueueJob {
            id: id.to_string(),
            priority,
            status: QueueJobStatus::Pending,
            positive_prompt: "a cat".to_string(),
            negative_prompt: "lowres".to_string(),
            settings_json: r#"{"steps":20}"#.to_string(),
            pipeline_log: None,
            original_idea: Some("cat".to_string()),
            linked_comparison_id: None,
            created_at: None,
            started_at: None,
            completed_at: None,
            result_image_id: None,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let conn = setup();
        let job = make_job("job-1", QueuePriority::Normal);
        insert_job(&conn, &job).unwrap();

        let retrieved = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(retrieved.positive_prompt, "a cat");
        assert_eq!(retrieved.priority, QueuePriority::Normal);
        assert_eq!(retrieved.status, QueueJobStatus::Pending);
    }

    #[test]
    fn test_pending_jobs_sorted_by_priority() {
        let conn = setup();
        insert_job(&conn, &make_job("low-1", QueuePriority::Low)).unwrap();
        insert_job(&conn, &make_job("high-1", QueuePriority::High)).unwrap();
        insert_job(&conn, &make_job("normal-1", QueuePriority::Normal)).unwrap();

        let pending = get_pending_jobs(&conn).unwrap();
        assert_eq!(pending.len(), 3);
        assert_eq!(pending[0].id, "high-1");
        assert_eq!(pending[1].id, "normal-1");
        assert_eq!(pending[2].id, "low-1");
    }

    #[test]
    fn test_update_status() {
        let conn = setup();
        insert_job(&conn, &make_job("job-1", QueuePriority::Normal)).unwrap();

        update_job_status(&conn, "job-1", &QueueJobStatus::Generating).unwrap();
        let job = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(job.status, QueueJobStatus::Generating);
        assert!(job.started_at.is_some());

        update_job_status(&conn, "job-1", &QueueJobStatus::Completed).unwrap();
        let job = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(job.status, QueueJobStatus::Completed);
        assert!(job.completed_at.is_some());
    }

    #[test]
    fn test_cancel_job() {
        let conn = setup();
        insert_job(&conn, &make_job("job-1", QueuePriority::Normal)).unwrap();
        cancel_job(&conn, "job-1").unwrap();

        let job = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(job.status, QueueJobStatus::Cancelled);
    }

    #[test]
    fn test_cancel_only_pending() {
        let conn = setup();
        insert_job(&conn, &make_job("job-1", QueuePriority::Normal)).unwrap();
        update_job_status(&conn, "job-1", &QueueJobStatus::Generating).unwrap();

        // Cancelling a non-pending job should return an error
        let result = cancel_job(&conn, "job-1");
        assert!(result.is_err());

        // Status should remain unchanged
        let job = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(job.status, QueueJobStatus::Generating);
    }

    #[test]
    fn test_requeue_interrupted() {
        let conn = setup();
        insert_job(&conn, &make_job("job-1", QueuePriority::Normal)).unwrap();
        update_job_status(&conn, "job-1", &QueueJobStatus::Generating).unwrap();

        let count = requeue_interrupted_jobs(&conn).unwrap();
        assert_eq!(count, 1);

        let job = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(job.status, QueueJobStatus::Pending);
        // Requeued jobs retain their original priority
        assert_eq!(job.priority, QueuePriority::Normal);
    }

    #[test]
    fn test_update_priority() {
        let conn = setup();
        insert_job(&conn, &make_job("job-1", QueuePriority::Low)).unwrap();
        update_job_priority(&conn, "job-1", &QueuePriority::High).unwrap();

        let job = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(job.priority, QueuePriority::High);
    }

    #[test]
    fn test_set_result_image() {
        let conn = setup();
        // Insert a test image to satisfy foreign key
        conn.execute(
            "INSERT INTO images (id, filename) VALUES ('img-001', 'test.png')",
            [],
        ).unwrap();

        insert_job(&conn, &make_job("job-1", QueuePriority::Normal)).unwrap();
        set_job_result_image(&conn, "job-1", "img-001").unwrap();

        let job = get_job(&conn, "job-1").unwrap().unwrap();
        assert_eq!(job.result_image_id.unwrap(), "img-001");
    }
}
