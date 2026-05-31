use crate::config;
use crate::state::AppState;
use crate::types::config::AppConfig;
use tauri::Manager;

#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state
        .config
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(config.clone())
}

#[tauri::command]
pub fn save_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    config::manager::save_config_to_disk(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    // Create image directories immediately so no restart is needed
    let image_base = config::manager::image_dir(&config);
    std::fs::create_dir_all(image_base.join("originals"))
        .map_err(|e| format!("Failed to create image directory: {}", e))?;
    std::fs::create_dir_all(image_base.join("thumbnails"))
        .map_err(|e| format!("Failed to create thumbnails directory: {}", e))?;

    // Expand asset protocol scope to cover the (possibly new) image directory
    let scope = app.asset_protocol_scope();
    if let Err(e) = scope.allow_directory(&image_base, true) {
        eprintln!(
            "[config] Failed to add image directory to asset scope: {}",
            e
        );
    }

    let mut current = state
        .config
        .write()
        .map_err(|e| format!("Failed to write config: {}", e))?;
    *current = config;

    Ok(())
}
