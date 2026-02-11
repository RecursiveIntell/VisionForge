use crate::config;
use crate::state::AppState;
use crate::types::config::AppConfig;

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    Ok(config.clone())
}

#[tauri::command]
pub fn save_config(
    state: tauri::State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    config::manager::save_config_to_disk(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    let mut current = state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    *current = config;

    Ok(())
}
