use crate::db;
use crate::state::AppState;
use crate::types::checkpoints::{CheckpointObservation, CheckpointProfile, PromptTerm};

#[tauri::command]
pub async fn upsert_checkpoint(
    state: tauri::State<'_, AppState>,
    profile: CheckpointProfile,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::upsert_checkpoint(&conn, &profile)
        .map_err(|e| format!("Failed to upsert checkpoint: {:#}", e))
}

#[tauri::command]
pub async fn get_checkpoint(
    state: tauri::State<'_, AppState>,
    filename: String,
) -> Result<Option<CheckpointProfile>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::get_checkpoint(&conn, &filename)
        .map_err(|e| format!("Failed to get checkpoint: {:#}", e))
}

#[tauri::command]
pub async fn list_checkpoint_profiles(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<CheckpointProfile>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::list_checkpoints(&conn)
        .map_err(|e| format!("Failed to list checkpoints: {:#}", e))
}

#[tauri::command]
pub async fn add_prompt_term(
    state: tauri::State<'_, AppState>,
    term: PromptTerm,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::add_prompt_term(&conn, &term)
        .map_err(|e| format!("Failed to add prompt term: {:#}", e))
}

#[tauri::command]
pub async fn get_prompt_terms(
    state: tauri::State<'_, AppState>,
    checkpoint_id: i64,
) -> Result<Vec<PromptTerm>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::get_prompt_terms(&conn, checkpoint_id)
        .map_err(|e| format!("Failed to get prompt terms: {:#}", e))
}

#[tauri::command]
pub async fn add_checkpoint_observation(
    state: tauri::State<'_, AppState>,
    observation: CheckpointObservation,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::add_observation(&conn, &observation)
        .map_err(|e| format!("Failed to add observation: {:#}", e))
}

#[tauri::command]
pub async fn get_checkpoint_observations(
    state: tauri::State<'_, AppState>,
    checkpoint_id: i64,
) -> Result<Vec<CheckpointObservation>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::get_observations(&conn, checkpoint_id)
        .map_err(|e| format!("Failed to get observations: {:#}", e))
}

#[tauri::command]
pub async fn get_checkpoint_context(
    state: tauri::State<'_, AppState>,
    filename: String,
) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::checkpoints::get_checkpoint_context(&conn, &filename)
        .map_err(|e| format!("Failed to get checkpoint context: {:#}", e))
}
