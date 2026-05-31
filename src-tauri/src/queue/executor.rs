use anyhow::{Context, Result};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

use crate::comfyui::{client, workflow};
use crate::db;
use crate::gallery::storage;
use crate::queue::manager;
use crate::state::AppState;
use crate::types::gallery::ImageEntry;
use crate::types::generation::GenerationRequest;

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const COMFYUI_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes

/// Event payloads emitted to the frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStartedEvent {
    pub job_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobCompletedEvent {
    pub job_id: String,
    pub image_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobFailedEvent {
    pub job_id: String,
    pub error: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobProgressEvent {
    pub job_id: String,
    pub current_step: u32,
    pub total_steps: u32,
    pub progress: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobCancelledEvent {
    pub job_id: String,
}

/// Spawn the background queue executor. Call this once during app setup.
pub fn spawn(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        run_loop(app_handle).await;
    });
}

async fn run_loop(app_handle: AppHandle) {
    let mut consecutive_count: u32 = 0;

    // Wait for AppState to become available and get shutdown receiver
    let state = loop {
        if let Some(s) = app_handle.try_state::<AppState>() {
            break s;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    };
    let mut shutdown_rx = state.shutdown_tx.subscribe();

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                eprintln!("[queue] Shutdown signal received, stopping executor");
                return;
            }
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
        }

        let state = match app_handle.try_state::<AppState>() {
            Some(s) => s,
            None => {
                eprintln!("[queue] AppState not available yet, waiting...");
                continue;
            }
        };

        // Check if paused
        if state.queue_paused.load(Ordering::Relaxed) {
            continue;
        }

        // Read hardware config
        let (cooldown_secs, max_consecutive) = {
            match state.config_snapshot() {
                Ok(c) => (
                    c.hardware.cooldown_seconds,
                    c.hardware.max_consecutive_generations,
                ),
                Err(e) => {
                    eprintln!("[queue] Config mutex poisoned: {}", e);
                    continue;
                }
            }
        };

        // Check consecutive limit
        if max_consecutive > 0 && consecutive_count >= max_consecutive {
            eprintln!(
                "[queue] Consecutive generation limit ({}) reached, cooling down",
                max_consecutive
            );
            tokio::time::sleep(Duration::from_secs(cooldown_secs as u64)).await;
            consecutive_count = 0;
            continue;
        }

        // Get next pending job
        let job = {
            let conn = match state.db.lock() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[queue] DB mutex poisoned: {}", e);
                    continue;
                }
            };
            match manager::next_pending_job(&conn) {
                Ok(Some(j)) => j,
                Ok(None) => {
                    consecutive_count = 0;
                    continue;
                }
                Err(e) => {
                    eprintln!("[queue] Failed to query next pending job: {:#}", e);
                    continue;
                }
            }
        };

        // Process the job
        let result = process_job(&app_handle, &state, &job).await;

        match result {
            Ok(_) => {
                consecutive_count += 1;

                // Cooldown between generations
                if cooldown_secs > 0 {
                    tokio::time::sleep(Duration::from_secs(cooldown_secs as u64)).await;
                }
            }
            Err(e) => {
                let err_msg = format!("{:#}", e);
                // Check if this was a cancellation — don't re-mark as failed
                let was_cancelled = {
                    match state.db.lock() {
                        Ok(conn) => db::queue::is_job_cancelled(&conn, &job.id).unwrap_or(false),
                        Err(e) => {
                            eprintln!(
                                "[queue] WARNING: DB mutex poisoned while checking cancellation for job {}: {}",
                                job.id, e
                            );
                            false
                        }
                    }
                };

                if was_cancelled {
                    eprintln!("[queue] Job {} was cancelled", job.id);
                    let _ = app_handle.emit(
                        "queue:job_cancelled",
                        JobCancelledEvent {
                            job_id: job.id.clone(),
                        },
                    );
                } else {
                    eprintln!("[queue] Job {} failed: {}", job.id, err_msg);
                    if let Ok(conn) = state.db.lock() {
                        let _ = manager::mark_failed(&conn, &job.id);
                    }
                    let _ = app_handle.emit(
                        "queue:job_failed",
                        JobFailedEvent {
                            job_id: job.id.clone(),
                            error: err_msg,
                        },
                    );
                }
            }
        }
    }
}

async fn process_job(
    app_handle: &AppHandle,
    state: &AppState,
    job: &crate::types::queue::QueueJob,
) -> Result<()> {
    let endpoint = state.config_snapshot()?.comfyui.endpoint;

    // Mark as generating
    {
        let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        manager::mark_generating(&conn, &job.id)?;
    }

    let _ = app_handle.emit(
        "queue:job_started",
        JobStartedEvent {
            job_id: job.id.clone(),
        },
    );

    // Build generation request from job data
    let gen_request = build_generation_request(job)?;
    let (workflow_json, actual_seed) = workflow::build_txt2img(&gen_request);
    let client_id = uuid::Uuid::new_v4().to_string();

    // Queue prompt to ComfyUI
    let prompt_id = client::queue_prompt(&state.http_client, &endpoint, &workflow_json, &client_id)
        .await
        .context("Failed to queue prompt to ComfyUI")?;

    // Wait for completion with real-time progress via WebSocket,
    // racing against a cancellation poll loop that checks the DB every 2s.
    let job_id_for_progress = job.id.clone();
    let ah_progress = app_handle.clone();
    let ws_future = client::wait_for_completion_ws(
        &state.http_client,
        &endpoint,
        &prompt_id,
        &client_id,
        COMFYUI_TIMEOUT,
        move |update| {
            let progress = if update.total_steps > 0 {
                update.current_step as f64 / update.total_steps as f64
            } else {
                0.0
            };
            let _ = ah_progress.emit(
                "queue:job_progress",
                JobProgressEvent {
                    job_id: job_id_for_progress.clone(),
                    current_step: update.current_step,
                    total_steps: update.total_steps,
                    progress,
                },
            );
        },
    );

    let job_id_cancel = job.id.clone();
    let cancel_poll = async {
        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let is_cancelled = {
                if let Ok(conn) = state.db.lock() {
                    db::queue::is_job_cancelled(&conn, &job_id_cancel).unwrap_or(false)
                } else {
                    false
                }
            };
            if is_cancelled {
                return;
            }
        }
    };

    let gen_status = tokio::select! {
        result = ws_future => result.context("Error waiting for ComfyUI completion")?,
        _ = cancel_poll => {
            // Job was cancelled — interrupt ComfyUI best-effort
            let _ = client::interrupt(&state.http_client, &endpoint).await;
            anyhow::bail!("Job cancelled by user");
        }
    };

    if let Some(ref error) = gen_status.error {
        anyhow::bail!("Generation failed: {}", error);
    }

    // Fetch full history to get ImageRef data (subfolder, type)
    let history = client::get_history(&state.http_client, &endpoint, &prompt_id)
        .await
        .context("Failed to fetch ComfyUI history after completion")?
        .with_context(|| "Completed prompt has no history entry")?;

    if history.image_filenames.is_empty() {
        anyhow::bail!("ComfyUI returned no image filenames");
    }

    // Prefer the last image (most likely to be the final output, not a preview)
    let img_ref = history
        .image_filenames
        .last()
        .context("ComfyUI returned no image filenames")?;
    let image_bytes = client::get_image(
        &state.http_client,
        &endpoint,
        &img_ref.filename,
        &img_ref.subfolder,
        &img_ref.img_type,
    )
    .await
    .context("Failed to download image from ComfyUI")?;

    let local_filename = storage::generate_filename();
    let config_clone = state.config_snapshot()?;
    {
        let filename_clone = local_filename.clone();
        let bytes_clone = image_bytes.clone();
        let config_for_save = config_clone.clone();
        tokio::task::spawn_blocking(move || {
            storage::save_image_from_bytes_with_config(
                &config_for_save,
                &bytes_clone,
                &filename_clone,
            )
        })
        .await
        .context("Image save task panicked")?
        .context("Failed to save image to gallery")?;
    }

    // === POST-GENERATION CANCELLATION CHECK ===
    // If the job was cancelled while we were downloading, don't persist to gallery.
    {
        let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        let was_cancelled = db::queue::is_job_cancelled(&conn, &job.id).unwrap_or(false);
        drop(conn);
        if was_cancelled {
            // Clean up the file we just saved
            if let Err(cleanup_err) =
                storage::delete_image_files_for(&config_clone, &local_filename)
            {
                eprintln!(
                    "[queue] ERROR: Failed to clean up cancelled job image {}: {}",
                    local_filename, cleanup_err
                );
            }
            anyhow::bail!("Job cancelled by user");
        }
    }

    // Insert into gallery DB
    let image_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let image_entry = ImageEntry {
        id: image_id.clone(),
        filename: local_filename,
        created_at: now,
        positive_prompt: Some(job.positive_prompt.clone()),
        negative_prompt: Some(job.negative_prompt.clone()),
        original_idea: job.original_idea.clone(),
        checkpoint: Some(gen_request.checkpoint.clone()),
        width: Some(gen_request.width),
        height: Some(gen_request.height),
        steps: Some(gen_request.steps),
        cfg_scale: Some(gen_request.cfg_scale),
        sampler: Some(gen_request.sampler.clone()),
        scheduler: Some(gen_request.scheduler.clone()),
        seed: Some(actual_seed),
        pipeline_log: job.pipeline_log.clone(),
        selected_concept: job.selected_concept,
        auto_approved: job.auto_approved,
        caption: None,
        caption_edited: false,
        rating: None,
        favorite: false,
        deleted: false,
        user_note: None,
        tags: None,
    };

    {
        let conn = state.db.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        db::images::insert_image(&conn, &image_entry)?;
        manager::mark_completed(&conn, &job.id, &image_id)?;
    }

    let _ = app_handle.emit(
        "queue:job_completed",
        JobCompletedEvent {
            job_id: job.id.clone(),
            image_id,
        },
    );

    Ok(())
}

/// Parse the settings_json stored in a QueueJob into a GenerationRequest.
fn build_generation_request(job: &crate::types::queue::QueueJob) -> Result<GenerationRequest> {
    use crate::types::generation::GenerationSettings;

    let settings: GenerationSettings =
        serde_json::from_str(&job.settings_json).context("Failed to parse job settings_json")?;

    settings.validate().context("Invalid generation settings")?;

    Ok(GenerationRequest {
        positive_prompt: job.positive_prompt.clone(),
        negative_prompt: job.negative_prompt.clone(),
        checkpoint: settings.checkpoint,
        width: settings.width,
        height: settings.height,
        steps: settings.steps,
        cfg_scale: settings.cfg_scale,
        sampler: settings.sampler,
        scheduler: settings.scheduler,
        seed: settings.seed,
        batch_size: settings.batch_size,
    })
}

#[cfg(test)]
#[path = "executor_test.rs"]
mod tests;
