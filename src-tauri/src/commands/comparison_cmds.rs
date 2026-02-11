use crate::db;
use crate::state::AppState;
use crate::types::comparison::Comparison;

#[tauri::command]
pub async fn create_comparison(
    state: tauri::State<'_, AppState>,
    comparison: Comparison,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::comparisons::insert_comparison(&conn, &comparison)
        .map_err(|e| format!("Failed to create comparison: {:#}", e))
}

#[tauri::command]
pub async fn get_comparison(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Option<Comparison>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::comparisons::get_comparison(&conn, &id)
        .map_err(|e| format!("Failed to get comparison: {:#}", e))
}

#[tauri::command]
pub async fn list_comparisons(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Comparison>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::comparisons::list_comparisons(&conn)
        .map_err(|e| format!("Failed to list comparisons: {:#}", e))
}

#[tauri::command]
pub async fn list_comparisons_for_checkpoint(
    state: tauri::State<'_, AppState>,
    checkpoint: String,
) -> Result<Vec<Comparison>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::comparisons::list_comparisons_for_checkpoint(&conn, &checkpoint)
        .map_err(|e| format!("Failed to list comparisons: {:#}", e))
}

#[tauri::command]
pub async fn update_comparison_note(
    state: tauri::State<'_, AppState>,
    id: String,
    note: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::comparisons::update_comparison_note(&conn, &id, &note)
        .map_err(|e| format!("Failed to update comparison note: {:#}", e))
}

#[tauri::command]
pub async fn delete_comparison(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::comparisons::delete_comparison(&conn, &id)
        .map_err(|e| format!("Failed to delete comparison: {:#}", e))
}
