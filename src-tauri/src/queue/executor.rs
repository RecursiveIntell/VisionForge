use anyhow::{Context, Result};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

use crate::comfyui::{client, workflow};
use crate::db;
use crate::gallery::storage;
use crate::queue::manager;
use crate::state::AppState;
use crate::types::generation::GenerationRequest;
use crate::types::gallery::ImageEntry;

const POLL_INTERVAL: Duration = Duration::from_secs(3);
const COMFYUI_POLL_INTERVAL: Duration = Duration::from_secs(2);
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

/// Spawn the background queue executor. Call this once during app setup.
pub fn spawn(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        run_loop(app_handle).await;
    });
}

async fn run_loop(app_handle: AppHandle) {
    let mut consecutive_count: u32 = 0;

    loop {
        tokio::time::sleep(POLL_INTERVAL).await;

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
            match state.config.lock() {
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
                eprintln!("[queue] Job {} failed: {:#}", job.id, e);
                // Mark failed in DB
                if let Ok(conn) = state.db.lock() {
                    let _ = manager::mark_failed(&conn, &job.id);
                }
                let _ = app_handle.emit(
                    "queue:job_failed",
                    JobFailedEvent {
                        job_id: job.id.clone(),
                        error: format!("{:#}", e),
                    },
                );
            }
        }
    }
}

async fn process_job(
    app_handle: &AppHandle,
    state: &AppState,
    job: &crate::types::queue::QueueJob,
) -> Result<()> {
    let (endpoint, _) = {
        let config = state.config.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        (config.comfyui.endpoint.clone(), ())
    };

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
    let workflow_json = workflow::build_txt2img(&gen_request);
    let client_id = uuid::Uuid::new_v4().to_string();

    // Queue prompt to ComfyUI
    let prompt_id = client::queue_prompt(
        &state.http_client,
        &endpoint,
        &workflow_json,
        &client_id,
    )
    .await
    .context("Failed to queue prompt to ComfyUI")?;

    // Wait for completion
    let gen_status = client::wait_for_completion(
        &state.http_client,
        &endpoint,
        &prompt_id,
        COMFYUI_POLL_INTERVAL,
        COMFYUI_TIMEOUT,
    )
    .await
    .context("Error waiting for ComfyUI completion")?;

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

    // Download and save the first image (batch_size=1 typical)
    let img_ref = &history.image_filenames[0];
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
    storage::save_image_from_bytes(&image_bytes, &local_filename)
        .context("Failed to save image to gallery")?;

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
        seed: Some(gen_request.seed),
        pipeline_log: job.pipeline_log.clone(),
        selected_concept: None,
        auto_approved: false,
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
fn build_generation_request(
    job: &crate::types::queue::QueueJob,
) -> Result<GenerationRequest> {
    let settings: serde_json::Value = serde_json::from_str(&job.settings_json)
        .context("Failed to parse job settings_json")?;

    Ok(GenerationRequest {
        positive_prompt: job.positive_prompt.clone(),
        negative_prompt: job.negative_prompt.clone(),
        checkpoint: settings
            .get("checkpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("dreamshaper_8.safetensors")
            .to_string(),
        width: settings
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(512) as u32,
        height: settings
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(768) as u32,
        steps: settings
            .get("steps")
            .and_then(|v| v.as_u64())
            .unwrap_or(25) as u32,
        cfg_scale: settings
            .get("cfgScale")
            .or_else(|| settings.get("cfg_scale"))
            .and_then(|v| v.as_f64())
            .unwrap_or(7.5),
        sampler: settings
            .get("sampler")
            .and_then(|v| v.as_str())
            .unwrap_or("dpmpp_2m")
            .to_string(),
        scheduler: settings
            .get("scheduler")
            .and_then(|v| v.as_str())
            .unwrap_or("karras")
            .to_string(),
        seed: settings
            .get("seed")
            .and_then(|v| v.as_i64())
            .unwrap_or(-1),
        batch_size: settings
            .get("batchSize")
            .or_else(|| settings.get("batch_size"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32,
    })
}

#[cfg(test)]
#[path = "executor_test.rs"]
mod tests;
