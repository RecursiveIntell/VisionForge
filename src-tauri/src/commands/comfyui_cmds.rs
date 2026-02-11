use crate::comfyui::{client, models, workflow};
use crate::state::AppState;
use crate::types::generation::{GenerationRequest, GenerationStatus, GenerationStatusKind};

#[tauri::command]
pub async fn check_comfyui_health(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    client::check_health(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn get_comfyui_checkpoints(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    models::list_checkpoints(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn get_comfyui_samplers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    models::list_samplers(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn get_comfyui_schedulers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    models::list_schedulers(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn queue_generation(
    state: tauri::State<'_, AppState>,
    request: GenerationRequest,
) -> Result<GenerationStatus, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    let workflow_json = workflow::build_txt2img(&request);
    let client_id = uuid::Uuid::new_v4().to_string();

    let prompt_id =
        client::queue_prompt(&state.http_client, &endpoint, &workflow_json, &client_id)
            .await
            .map_err(|e| format!("{:#}", e))?;

    Ok(GenerationStatus {
        prompt_id,
        status: GenerationStatusKind::Queued,
        progress: None,
        current_step: None,
        total_steps: None,
        image_filenames: None,
        error: None,
    })
}

#[tauri::command]
pub async fn get_generation_status(
    state: tauri::State<'_, AppState>,
    prompt_id: String,
) -> Result<GenerationStatus, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    let history = client::get_history(&state.http_client, &endpoint, &prompt_id)
        .await
        .map_err(|e| format!("{:#}", e))?;

    match history {
        Some(h) => {
            if h.completed {
                let filenames: Vec<String> =
                    h.image_filenames.iter().map(|r| r.filename.clone()).collect();
                Ok(GenerationStatus {
                    prompt_id,
                    status: GenerationStatusKind::Completed,
                    progress: Some(1.0),
                    current_step: None,
                    total_steps: None,
                    image_filenames: if filenames.is_empty() {
                        None
                    } else {
                        Some(filenames)
                    },
                    error: None,
                })
            } else if h.status == "error" {
                Ok(GenerationStatus {
                    prompt_id,
                    status: GenerationStatusKind::Failed,
                    progress: None,
                    current_step: None,
                    total_steps: None,
                    image_filenames: None,
                    error: Some("ComfyUI generation failed".to_string()),
                })
            } else {
                Ok(GenerationStatus {
                    prompt_id,
                    status: GenerationStatusKind::Generating,
                    progress: None,
                    current_step: None,
                    total_steps: None,
                    image_filenames: None,
                    error: None,
                })
            }
        }
        None => Ok(GenerationStatus {
            prompt_id,
            status: GenerationStatusKind::Queued,
            progress: None,
            current_step: None,
            total_steps: None,
            image_filenames: None,
            error: None,
        }),
    }
}

#[tauri::command]
pub async fn get_comfyui_queue_status(
    state: tauri::State<'_, AppState>,
) -> Result<client::QueueStatus, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    client::get_queue_status(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn free_comfyui_memory(
    state: tauri::State<'_, AppState>,
    unload_models: bool,
) -> Result<(), String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    client::free_memory(&state.http_client, &endpoint, unload_models)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn interrupt_comfyui(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.comfyui.endpoint.clone()
    };

    client::interrupt(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}
