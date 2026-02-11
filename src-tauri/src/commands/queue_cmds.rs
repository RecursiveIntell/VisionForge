use crate::queue::manager;
use crate::state::AppState;
use crate::types::queue::{QueueJob, QueuePriority};

#[tauri::command]
pub async fn add_to_queue(
    state: tauri::State<'_, AppState>,
    job: QueueJob,
) -> Result<String, String> {
    manager::add_job(&state, job)
        .map_err(|e| format!("Failed to add job to queue: {:#}", e))
}

#[tauri::command]
pub async fn get_queue(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<QueueJob>, String> {
    manager::get_all_jobs(&state)
        .map_err(|e| format!("Failed to get queue: {:#}", e))
}

#[tauri::command]
pub async fn reorder_queue(
    state: tauri::State<'_, AppState>,
    job_id: String,
    new_priority: QueuePriority,
) -> Result<(), String> {
    manager::reorder_job(&state, &job_id, new_priority)
        .map_err(|e| format!("Failed to reorder queue: {:#}", e))
}

#[tauri::command]
pub async fn cancel_queue_job(
    state: tauri::State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    manager::cancel_job(&state, &job_id)
        .map_err(|e| format!("Failed to cancel job: {:#}", e))
}

#[tauri::command]
pub async fn pause_queue(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    manager::pause_queue(&state);
    Ok(())
}

#[tauri::command]
pub async fn resume_queue(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    manager::resume_queue(&state);
    Ok(())
}

#[tauri::command]
pub async fn is_queue_paused(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    Ok(manager::is_paused(&state))
}
