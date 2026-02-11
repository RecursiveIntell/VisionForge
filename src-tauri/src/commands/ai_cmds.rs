use crate::ai::{captioner, tagger};
use crate::db;
use crate::gallery::storage;
use crate::state::AppState;

#[tauri::command]
pub async fn tag_image(
    state: tauri::State<'_, AppState>,
    image_id: String,
) -> Result<Vec<String>, String> {
    let (endpoint, model, filename) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        let endpoint = config.ollama.endpoint.clone();
        let model = config.models.tagger.clone();

        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let image = db::images::get_image(&conn, &image_id)
            .map_err(|e| format!("{:#}", e))?
            .ok_or_else(|| format!("Image {} not found", image_id))?;

        (endpoint, model, image.filename)
    };

    let image_path = storage::get_image_path(&filename);
    if !image_path.exists() {
        return Err(format!("Image file not found: {}", image_path.display()));
    }

    let tags = tagger::tag_image(&state.http_client, &endpoint, &model, &image_path)
        .await
        .map_err(|e| format!("Tagging failed: {:#}", e))?;

    // Save tags to database
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        for tag_name in &tags {
            let _ = db::tags::add_image_tag(&conn, &image_id, tag_name, "ai", None);
        }
    }

    Ok(tags)
}

#[tauri::command]
pub async fn caption_image(
    state: tauri::State<'_, AppState>,
    image_id: String,
) -> Result<String, String> {
    let (endpoint, model, filename) = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        let endpoint = config.ollama.endpoint.clone();
        let model = config.models.captioner.clone();

        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let image = db::images::get_image(&conn, &image_id)
            .map_err(|e| format!("{:#}", e))?
            .ok_or_else(|| format!("Image {} not found", image_id))?;

        (endpoint, model, image.filename)
    };

    let image_path = storage::get_image_path(&filename);
    if !image_path.exists() {
        return Err(format!("Image file not found: {}", image_path.display()));
    }

    let caption = captioner::caption_image(&state.http_client, &endpoint, &model, &image_path)
        .await
        .map_err(|e| format!("Captioning failed: {:#}", e))?;

    // Save caption to database (AI-generated, not user-edited)
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        db::images::update_image_caption(&conn, &image_id, &caption, false)
            .map_err(|e| format!("{:#}", e))?;
    }

    Ok(caption)
}
