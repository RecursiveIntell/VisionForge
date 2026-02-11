use crate::db;
use crate::gallery::storage;
use crate::state::AppState;
use crate::types::gallery::{GalleryFilter, ImageEntry};

#[tauri::command]
pub async fn get_gallery_images(
    state: tauri::State<'_, AppState>,
    filter: GalleryFilter,
) -> Result<Vec<ImageEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut images = db::images::list_images(&conn, &filter)
        .map_err(|e| format!("Failed to load gallery: {:#}", e))?;

    // Load tags for each image
    for img in &mut images {
        let tags = db::tags::get_image_tags(&conn, &img.id)
            .unwrap_or_default();
        if !tags.is_empty() {
            img.tags = Some(tags);
        }
    }

    Ok(images)
}

#[tauri::command]
pub async fn get_image(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Option<ImageEntry>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut image = db::images::get_image(&conn, &id)
        .map_err(|e| format!("Failed to get image: {:#}", e))?;

    if let Some(ref mut img) = image {
        let tags = db::tags::get_image_tags(&conn, &img.id).unwrap_or_default();
        if !tags.is_empty() {
            img.tags = Some(tags);
        }
    }

    Ok(image)
}

#[tauri::command]
pub async fn delete_image(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::images::soft_delete_image(&conn, &id)
        .map_err(|e| format!("Failed to delete image: {:#}", e))
}

#[tauri::command]
pub async fn restore_image(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::images::restore_image(&conn, &id)
        .map_err(|e| format!("Failed to restore image: {:#}", e))
}

#[tauri::command]
pub async fn permanently_delete_image(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get filename before deleting from DB
    let image = db::images::get_image(&conn, &id)
        .map_err(|e| format!("Failed to get image: {:#}", e))?;

    db::images::permanently_delete_image(&conn, &id)
        .map_err(|e| format!("Failed to permanently delete image: {:#}", e))?;

    // Delete files from disk
    if let Some(img) = image {
        let _ = storage::delete_image_files(&img.filename);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_image_rating(
    state: tauri::State<'_, AppState>,
    id: String,
    rating: Option<u32>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::images::update_image_rating(&conn, &id, rating)
        .map_err(|e| format!("Failed to update rating: {:#}", e))
}

#[tauri::command]
pub async fn update_image_favorite(
    state: tauri::State<'_, AppState>,
    id: String,
    favorite: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::images::update_image_favorite(&conn, &id, favorite)
        .map_err(|e| format!("Failed to update favorite: {:#}", e))
}

#[tauri::command]
pub async fn update_caption(
    state: tauri::State<'_, AppState>,
    id: String,
    caption: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::images::update_image_caption(&conn, &id, &caption, true)
        .map_err(|e| format!("Failed to update caption: {:#}", e))
}

#[tauri::command]
pub async fn update_image_note(
    state: tauri::State<'_, AppState>,
    id: String,
    note: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::images::update_image_note(&conn, &id, &note)
        .map_err(|e| format!("Failed to update note: {:#}", e))
}

#[tauri::command]
pub async fn add_tag(
    state: tauri::State<'_, AppState>,
    image_id: String,
    tag: String,
    source: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tags::add_image_tag(&conn, &image_id, &tag, &source, None)
        .map_err(|e| format!("Failed to add tag: {:#}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag(
    state: tauri::State<'_, AppState>,
    image_id: String,
    tag_id: i64,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tags::remove_image_tag(&conn, &image_id, tag_id)
        .map_err(|e| format!("Failed to remove tag: {:#}", e))
}

#[tauri::command]
pub async fn get_image_lineage(
    state: tauri::State<'_, AppState>,
    image_id: String,
) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let image = db::images::get_image(&conn, &image_id)
        .map_err(|e| format!("Failed to get image: {:#}", e))?;

    Ok(image.and_then(|img| img.pipeline_log))
}

#[tauri::command]
pub async fn get_image_file_path(
    _state: tauri::State<'_, AppState>,
    filename: String,
) -> Result<String, String> {
    let path = storage::get_image_path(&filename);
    if path.exists() {
        Ok(path.to_string_lossy().to_string())
    } else {
        Err(format!("Image file not found: {}", filename))
    }
}

#[tauri::command]
pub async fn get_thumbnail_file_path(
    _state: tauri::State<'_, AppState>,
    filename: String,
) -> Result<String, String> {
    let path = storage::get_thumbnail_path(&filename);
    if path.exists() {
        Ok(path.to_string_lossy().to_string())
    } else {
        Err(format!("Thumbnail not found for: {}", filename))
    }
}
