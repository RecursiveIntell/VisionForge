use crate::ai_batch::queue::AiBatchQueue;
use crate::ai_batch::types::*;
use crate::db;
use crate::state::AppState;

#[tauri::command]
pub async fn submit_batch_job(
    state: tauri::State<'_, AppState>,
    queue: tauri::State<'_, AiBatchQueue>,
    request: BatchRequest,
) -> Result<String, String> {
    let model = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        match request.op {
            BatchOpKind::Tag => config.models.tagger.clone(),
            BatchOpKind::Caption => config.models.captioner.clone(),
        }
    };

    let items: Vec<BatchItem> = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        request
            .image_ids
            .iter()
            .filter_map(|id| {
                let image = db::images::get_image(&conn, id).ok()??;
                Some(BatchItem {
                    image_id: id.clone(),
                    filename: image.filename,
                    status: BatchItemStatus::Pending,
                    error: None,
                    duration_ms: None,
                    width: image.width,
                    height: image.height,
                })
            })
            .collect()
    };

    if items.is_empty() {
        return Err("No valid images found for batch job".to_string());
    }

    let job = BatchJob {
        id: String::new(),
        op: request.op,
        model,
        overwrite_policy: request.overwrite_policy,
        items,
        status: BatchJobStatus::Queued,
        created_at: String::new(),
        started_at: None,
        completed_at: None,
        reordered: false,
        reorder_note: None,
    };

    queue
        .enqueue(job)
        .map_err(|e| format!("Failed to enqueue batch job: {:#}", e))
}

#[tauri::command]
pub async fn get_batch_jobs(
    queue: tauri::State<'_, AiBatchQueue>,
) -> Result<Vec<BatchJob>, String> {
    Ok(queue.list_jobs())
}

#[tauri::command]
pub async fn get_batch_job(
    queue: tauri::State<'_, AiBatchQueue>,
    job_id: String,
) -> Result<Option<BatchJob>, String> {
    Ok(queue.get_job(&job_id))
}

#[tauri::command]
pub async fn cancel_batch_item(
    queue: tauri::State<'_, AiBatchQueue>,
    job_id: String,
    image_id: String,
) -> Result<(), String> {
    queue
        .cancel_item(&job_id, &image_id)
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn cancel_batch_job(
    queue: tauri::State<'_, AiBatchQueue>,
    job_id: String,
) -> Result<(), String> {
    queue.cancel_job(&job_id).map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn retry_batch_failed(
    queue: tauri::State<'_, AiBatchQueue>,
    job_id: String,
) -> Result<(), String> {
    queue.retry_failed(&job_id).map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn get_batch_eta(
    queue: tauri::State<'_, AiBatchQueue>,
    job_id: String,
) -> Result<Option<u64>, String> {
    Ok(queue.estimate_remaining_ms(&job_id))
}

#[tauri::command]
pub async fn preview_batch_job(
    state: tauri::State<'_, AppState>,
    request: BatchRequest,
) -> Result<BatchPreview, String> {
    let model = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        match request.op {
            BatchOpKind::Tag => config.models.tagger.clone(),
            BatchOpKind::Caption => config.models.captioner.clone(),
        }
    };

    let mut would_process = 0usize;
    let mut would_skip = 0usize;

    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        for id in &request.image_ids {
            let has_data = match request.op {
                BatchOpKind::Tag => {
                    let tags = db::tags::get_image_tags(&conn, id).unwrap_or_default();
                    tags.iter().any(|t| t.source.as_deref() == Some("ai"))
                }
                BatchOpKind::Caption => {
                    let image = db::images::get_image(&conn, id).ok().flatten();
                    image
                        .map(|img| {
                            img.caption.is_some() && !img.caption.as_ref().unwrap().is_empty()
                        })
                        .unwrap_or(false)
                }
            };

            if has_data && request.overwrite_policy == OverwritePolicy::Skip {
                would_skip += 1;
            } else {
                would_process += 1;
            }
        }
    }

    Ok(BatchPreview {
        model,
        total: request.image_ids.len(),
        would_process,
        would_skip,
        op: request.op,
    })
}
