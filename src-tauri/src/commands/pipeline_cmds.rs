use std::sync::atomic::Ordering;

use crate::db;
use crate::pipeline::engine::{self, PipelineInput};
use crate::pipeline::engine_streaming;
use crate::pipeline::ollama;
use crate::pipeline::prompts::CheckpointContext;
use crate::state::AppState;
use crate::types::pipeline::PipelineResult;

#[tauri::command]
pub async fn run_full_pipeline(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    idea: String,
    num_concepts: u32,
    auto_approve: bool,
    checkpoint: Option<String>,
) -> Result<PipelineResult, String> {
    // Reset cancellation flag at start
    state.pipeline_cancelled.store(false, Ordering::Relaxed);

    let config = {
        let cfg = state.config.read().map_err(|e| e.to_string())?;
        cfg.clone()
    };

    // Build checkpoint context if a checkpoint is specified
    let checkpoint_context = if let Some(ref ckpt) = checkpoint {
        let ctx = {
            let conn = state.db.lock().map_err(|e| e.to_string())?;
            db::checkpoints::get_checkpoint_context(&conn, ckpt)
                .map_err(|e| format!("Failed to load checkpoint context: {}", e))?
        };
        if ctx.is_empty() {
            None
        } else {
            Some(parse_checkpoint_context_string(&ctx, ckpt))
        }
    } else {
        None
    };

    let input = PipelineInput {
        idea,
        num_concepts: num_concepts.clamp(1, 10),
        auto_approve,
        checkpoint_context,
    };

    let cancelled = state.pipeline_cancelled.clone();
    engine_streaming::run_pipeline_streaming(
        &state.http_client,
        &config,
        input,
        app_handle,
        cancelled,
    )
    .await
    .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn run_pipeline_stage(
    state: tauri::State<'_, AppState>,
    stage: String,
    input: String,
    model: String,
    checkpoint_context: Option<String>,
) -> Result<String, String> {
    let endpoint = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        config.ollama.endpoint.clone()
    };

    let ctx = checkpoint_context.map(|s| parse_checkpoint_context_string(&s, "unknown"));

    engine::run_single_stage(&state.http_client, &endpoint, &stage, &model, &input, ctx)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn get_available_models(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let endpoint = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        config.ollama.endpoint.clone()
    };

    let models = ollama::list_models(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))?;

    Ok(models.into_iter().map(|m| m.name).collect())
}

/// Returns installed model names that support thinking mode.
/// Combines auto-detection (template probing + known patterns) with
/// user-configured custom thinking models from the config.
#[tauri::command]
pub async fn get_thinking_models(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let (endpoint, custom_thinking) = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        (
            config.ollama.endpoint.clone(),
            config.models.custom_thinking_models.clone(),
        )
    };

    let all_models = ollama::list_models(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))?;

    let model_names: Vec<String> = all_models.into_iter().map(|m| m.name).collect();

    let mut thinking =
        ollama::detect_thinking_models(&state.http_client, &endpoint, &model_names).await;

    // Merge in user-configured custom thinking models (only if installed)
    for custom in &custom_thinking {
        if !thinking.contains(custom) && model_names.iter().any(|m| m == custom) {
            thinking.push(custom.clone());
        }
    }

    thinking.sort();
    thinking.dedup();
    Ok(thinking)
}

#[tauri::command]
pub async fn check_ollama_health(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let endpoint = {
        let config = state.config.read().map_err(|e| e.to_string())?;
        config.ollama.endpoint.clone()
    };

    ollama::check_health(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn cancel_pipeline(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.pipeline_cancelled.store(true, Ordering::Relaxed);
    Ok(())
}

fn parse_checkpoint_context_string(context_str: &str, checkpoint: &str) -> CheckpointContext {
    // Try JSON first (new format)
    if let Ok(ctx) = serde_json::from_str::<CheckpointContext>(context_str) {
        return ctx;
    }

    // Fall back to line-based parsing (legacy format)
    let mut ctx = CheckpointContext {
        checkpoint_name: checkpoint.to_string(),
        ..Default::default()
    };

    for line in context_str.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Checkpoint: ") {
            ctx.checkpoint_name = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("Base model: ") {
            ctx.base_model = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("Strengths: ") {
            ctx.strengths = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("Weaknesses: ") {
            ctx.weaknesses = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("Notes: ") {
            ctx.checkpoint_notes = rest.to_string();
        } else if line.starts_with("Known terms:") {
            ctx.term_list = String::new();
        } else if line.starts_with("- ") {
            if !ctx.term_list.is_empty() {
                ctx.term_list.push('\n');
            }
            ctx.term_list.push_str(line);
        }
    }

    ctx
}
