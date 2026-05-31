use crate::db;
use crate::gallery::export;
use crate::state::AppState;
use crate::types::gallery::GalleryFilter;

#[tauri::command]
pub async fn export_images(
    state: tauri::State<'_, AppState>,
    image_ids: Vec<String>,
    output_path: String,
) -> Result<(), String> {
    // Validate export path BEFORE doing any work
    let validated_path = export::validate_export_path(&output_path)
        .map_err(|e| format!("Invalid export path: {:#}", e))?;

    let (images, config) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut images = Vec::new();
        for id in &image_ids {
            if let Some(img) = db::images::get_image(&conn, id).map_err(|e| format!("{:#}", e))? {
                images.push(img);
            }
        }
        // Clone config and release lock before disk I/O
        let cfg = state.config.read().map_err(|e| e.to_string())?.clone();
        (images, cfg)
    };

    if images.is_empty() {
        return Err("No images found to export".to_string());
    }

    export::create_export_bundle_with_config(&images, &validated_path, Some(&config))
        .map_err(|e| format!("Failed to create export: {:#}", e))
}

#[tauri::command]
pub async fn export_gallery(
    state: tauri::State<'_, AppState>,
    filter: GalleryFilter,
    output_path: String,
) -> Result<u32, String> {
    // Validate export path BEFORE doing any work
    let validated_path = export::validate_export_path(&output_path)
        .map_err(|e| format!("Invalid export path: {:#}", e))?;

    let (images, config) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let imgs = db::images::list_images(&conn, &filter)
            .map_err(|e| format!("Failed to query images: {:#}", e))?;
        let cfg = state.config.read().map_err(|e| e.to_string())?.clone();
        (imgs, cfg)
    };

    if images.is_empty() {
        return Err("No images match the filter".to_string());
    }

    let count = images.len() as u32;
    export::create_export_bundle_with_config(&images, &validated_path, Some(&config))
        .map_err(|e| format!("Failed to create export: {:#}", e))?;

    Ok(count)
}
