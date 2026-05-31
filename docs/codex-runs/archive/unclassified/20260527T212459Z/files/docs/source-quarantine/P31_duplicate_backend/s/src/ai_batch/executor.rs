use anyhow::{Context, Result};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::ai::{captioner, tagger};
use crate::db;
use crate::gallery::storage;
use crate::state::AppState;

use super::queue::AiBatchQueue;
use super::types::*;

const POLL_INTERVAL: Duration = Duration::from_secs(2);

// -- Tauri event payloads --

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchJobStartedEvent {
    job_id: String,
    op: BatchOpKind,
    model: String,
    total_items: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchItemProgressEvent {
    job_id: String,
    image_id: String,
    status: BatchItemStatus,
    completed: usize,
    total: usize,
    error: Option<String>,
    duration_ms: Option<u64>,
    eta_remaining_ms: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchJobCompletedEvent {
    summary: BatchCompletionSummary,
}

/// Spawn the background batch executor. Call once during app setup.
pub fn spawn(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        run_loop(app_handle).await;
    });
}

async fn run_loop(app_handle: AppHandle) {
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

        let queue = match app_handle.try_state::<AiBatchQueue>() {
            Some(q) => q,
            None => continue,
        };

        if queue.has_running_job() {
            continue;
        }

        let job = match queue.next_queued() {
            Some(j) => j,
            None => continue,
        };

        let state = match app_handle.try_state::<AppState>() {
            Some(s) => s,
            None => continue,
        };

        process_batch_job(&app_handle, &state, &queue, &job).await;
    }
}

async fn process_batch_job(
    app_handle: &AppHandle,
    state: &AppState,
    queue: &AiBatchQueue,
    job: &BatchJob,
) {
    let job_id = job.id.clone();

    if let Err(e) = queue.mark_running(&job_id) {
        eprintln!("[ai_batch] Failed to mark job {} as running: {}", job_id, e);
        return;
    }

    let _ = app_handle.emit(
        "ai_batch:job_started",
        BatchJobStartedEvent {
            job_id: job_id.clone(),
            op: job.op,
            model: job.model.clone(),
            total_items: job.items.len(),
        },
    );

    let endpoint = {
        match state.config.read() {
            Ok(c) => c.ollama.endpoint.clone(),
            Err(e) => {
                eprintln!("[ai_batch] Config read failed: {}", e);
                return;
            }
        }
    };

    let total = job.items.len();
    let mut completed_count: usize = 0;

    for item in &job.items {
        // Check if item was cancelled
        if let Some(current_job) = queue.get_job(&job_id) {
            if let Some(ci) = current_job.items.iter().find(|i| i.image_id == item.image_id) {
                if ci.status == BatchItemStatus::Cancelled {
                    completed_count += 1;
                    continue;
                }
            }
            if current_job.status == BatchJobStatus::Cancelled {
                break;
            }
        }

        // Check overwrite policy
        if should_skip_item(state, &item.image_id, job.op, job.overwrite_policy) {
            let _ = queue.update_item(
                &job_id,
                &item.image_id,
                BatchItemStatus::Cancelled,
                Some("Skipped: already has data".to_string()),
                None,
            );
            completed_count += 1;

            let eta = queue.estimate_remaining_ms(&job_id);
            let _ = app_handle.emit(
                "ai_batch:item_progress",
                BatchItemProgressEvent {
                    job_id: job_id.clone(),
                    image_id: item.image_id.clone(),
                    status: BatchItemStatus::Cancelled,
                    completed: completed_count,
                    total,
                    error: Some("Skipped".to_string()),
                    duration_ms: None,
                    eta_remaining_ms: eta,
                },
            );
            continue;
        }

        // Mark item as running
        let _ = queue.update_item(
            &job_id,
            &item.image_id,
            BatchItemStatus::Running,
            None,
            None,
        );

        // Resolve image path
        let image_path = match resolve_image_path(state, &item.image_id) {
            Ok(p) => p,
            Err(e) => {
                let err = format!("Failed to resolve path: {}", e);
                let _ = queue.update_item(
                    &job_id,
                    &item.image_id,
                    BatchItemStatus::Failed,
                    Some(err.clone()),
                    None,
                );
                completed_count += 1;
                let eta = queue.estimate_remaining_ms(&job_id);
                let _ = app_handle.emit(
                    "ai_batch:item_progress",
                    BatchItemProgressEvent {
                        job_id: job_id.clone(),
                        image_id: item.image_id.clone(),
                        status: BatchItemStatus::Failed,
                        completed: completed_count,
                        total,
                        error: Some(err),
                        duration_ms: None,
                        eta_remaining_ms: eta,
                    },
                );
                continue;
            }
        };

        // Process the item
        let start = Instant::now();
        let result = match job.op {
            BatchOpKind::Tag => {
                process_tag(state, &endpoint, &job.model, &image_path, &item.image_id).await
            }
            BatchOpKind::Caption => {
                process_caption(state, &endpoint, &job.model, &image_path, &item.image_id).await
            }
        };
        let duration_ms = start.elapsed().as_millis() as u64;

        let (status, error) = match result {
            Ok(_) => (BatchItemStatus::Completed, None),
            Err(e) => (BatchItemStatus::Failed, Some(format!("{:#}", e))),
        };

        let _ = queue.update_item(
            &job_id,
            &item.image_id,
            status.clone(),
            error.clone(),
            Some(duration_ms),
        );

        completed_count += 1;
        let eta = queue.estimate_remaining_ms(&job_id);
        let _ = app_handle.emit(
            "ai_batch:item_progress",
            BatchItemProgressEvent {
                job_id: job_id.clone(),
                image_id: item.image_id.clone(),
                status,
                completed: completed_count,
                total,
                error,
                duration_ms: Some(duration_ms),
                eta_remaining_ms: eta,
            },
        );
    }

    match queue.mark_completed(&job_id) {
        Ok(Some(summary)) => {
            let _ = app_handle.emit(
                "ai_batch:job_completed",
                BatchJobCompletedEvent { summary },
            );
        }
        Ok(None) => {}
        Err(e) => eprintln!("[ai_batch] Failed to mark job {} completed: {}", job_id, e),
    }
}

fn should_skip_item(
    state: &AppState,
    image_id: &str,
    op: BatchOpKind,
    policy: OverwritePolicy,
) -> bool {
    if policy == OverwritePolicy::Overwrite {
        return false;
    }
    let conn = match state.db.lock() {
        Ok(c) => c,
        Err(_) => return false,
    };
    match op {
        BatchOpKind::Tag => {
            let tags = db::tags::get_image_tags(&conn, image_id).unwrap_or_default();
            tags.iter().any(|t| t.source.as_deref() == Some("ai"))
        }
        BatchOpKind::Caption => {
            let image = db::images::get_image(&conn, image_id).ok().flatten();
            image
                .map(|img| img.caption.is_some() && !img.caption.as_ref().unwrap().is_empty())
                .unwrap_or(false)
        }
    }
}

fn resolve_image_path(
    state: &AppState,
    image_id: &str,
) -> Result<std::path::PathBuf> {
    let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let image = db::images::get_image(&conn, image_id)
        .context("DB error")?
        .ok_or_else(|| anyhow::anyhow!("Image {} not found", image_id))?;
    let config = state.config.read().map_err(|e| anyhow::anyhow!("{}", e))?;

    let path = storage::get_image_path_for(&config, &image.filename);
    if path.exists() {
        return Ok(path);
    }
    let fallback = storage::get_image_path(&image.filename);
    if fallback.exists() {
        return Ok(fallback);
    }
    anyhow::bail!("Image file not found: {}", image.filename)
}

async fn process_tag(
    state: &AppState,
    endpoint: &str,
    model: &str,
    image_path: &std::path::Path,
    image_id: &str,
) -> Result<()> {
    let tags = tagger::tag_image(&state.http_client, endpoint, model, image_path)
        .await
        .context("Tagging failed")?;

    let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    for tag_name in &tags {
        let _ = db::tags::add_image_tag(&conn, image_id, tag_name, "ai", None);
    }
    Ok(())
}

async fn process_caption(
    state: &AppState,
    endpoint: &str,
    model: &str,
    image_path: &std::path::Path,
    image_id: &str,
) -> Result<()> {
    let caption = captioner::caption_image(&state.http_client, endpoint, model, image_path)
        .await
        .context("Captioning failed")?;

    let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    db::images::update_image_caption(&conn, image_id, &caption, false)
        .context("Failed to save caption")?;
    Ok(())
}
