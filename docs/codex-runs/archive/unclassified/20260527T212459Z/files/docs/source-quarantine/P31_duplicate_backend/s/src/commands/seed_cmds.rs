use crate::db;
use crate::state::AppState;
use crate::types::seeds::{SeedCheckpointNote, SeedEntry, SeedFilter};

#[tauri::command]
pub async fn create_seed(
    state: tauri::State<'_, AppState>,
    seed: SeedEntry,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::insert_seed(&conn, &seed).map_err(|e| format!("Failed to create seed: {:#}", e))
}

#[tauri::command]
pub async fn get_seed(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> Result<Option<SeedEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::get_seed(&conn, id).map_err(|e| format!("Failed to get seed: {:#}", e))
}

#[tauri::command]
pub async fn list_seeds(
    state: tauri::State<'_, AppState>,
    filter: SeedFilter,
) -> Result<Vec<SeedEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::list_seeds(&conn, &filter).map_err(|e| format!("Failed to list seeds: {:#}", e))
}

#[tauri::command]
pub async fn delete_seed(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::delete_seed(&conn, id).map_err(|e| format!("Failed to delete seed: {:#}", e))
}

#[tauri::command]
pub async fn add_seed_tag(
    state: tauri::State<'_, AppState>,
    seed_id: i64,
    tag_name: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::add_seed_tag(&conn, seed_id, &tag_name)
        .map_err(|e| format!("Failed to add seed tag: {:#}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn remove_seed_tag(
    state: tauri::State<'_, AppState>,
    seed_id: i64,
    tag_id: i64,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::remove_seed_tag(&conn, seed_id, tag_id)
        .map_err(|e| format!("Failed to remove seed tag: {:#}", e))
}

#[tauri::command]
pub async fn add_seed_checkpoint_note(
    state: tauri::State<'_, AppState>,
    note: SeedCheckpointNote,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::add_checkpoint_note(&conn, &note)
        .map_err(|e| format!("Failed to add checkpoint note: {:#}", e))
}

#[tauri::command]
pub async fn get_seed_checkpoint_notes(
    state: tauri::State<'_, AppState>,
    seed_id: i64,
) -> Result<Vec<SeedCheckpointNote>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::seeds::get_checkpoint_notes(&conn, seed_id)
        .map_err(|e| format!("Failed to get checkpoint notes: {:#}", e))
}
