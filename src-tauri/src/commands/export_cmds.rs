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
    let images = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let mut images = Vec::new();
        for id in &image_ids {
            if let Some(img) = db::images::get_image(&conn, id).map_err(|e| format!("{:#}", e))? {
                images.push(img);
            }
        }
        images
    };

    if images.is_empty() {
        return Err("No images found to export".to_string());
    }

    let config = state.config.lock().map_err(|e| e.to_string())?;
    let path = std::path::Path::new(&output_path);
    export::create_export_bundle_with_config(&images, path, Some(&config))
        .map_err(|e| format!("Failed to create export: {:#}", e))
}

#[tauri::command]
pub async fn export_gallery(
    state: tauri::State<'_, AppState>,
    filter: GalleryFilter,
    output_path: String,
) -> Result<u32, String> {
    let (images, config) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let imgs = db::images::list_images(&conn, &filter)
            .map_err(|e| format!("Failed to query images: {:#}", e))?;
        let cfg = state.config.lock().map_err(|e| e.to_string())?.clone();
        (imgs, cfg)
    };

    if images.is_empty() {
        return Err("No images match the filter".to_string());
    }

    let count = images.len() as u32;
    let path = std::path::Path::new(&output_path);
    export::create_export_bundle_with_config(&images, path, Some(&config))
        .map_err(|e| format!("Failed to create export: {:#}", e))?;

    Ok(count)
}
