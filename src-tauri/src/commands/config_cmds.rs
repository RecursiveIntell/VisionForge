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
pub fn save_config(state: tauri::State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    config::manager::save_config_to_disk(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    // Create image directories immediately so no restart is needed
    let image_base = config::manager::image_dir(&config);
    std::fs::create_dir_all(image_base.join("originals"))
        .map_err(|e| format!("Failed to create image directory: {}", e))?;
    std::fs::create_dir_all(image_base.join("thumbnails"))
        .map_err(|e| format!("Failed to create thumbnails directory: {}", e))?;

    let mut current = state
        .config
        .lock()
        .map_err(|e| format!("Failed to lock config: {}", e))?;
    *current = config;

    Ok(())
}
