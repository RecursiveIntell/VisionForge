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
    let config = {
        let cfg = state.config.lock().map_err(|e| e.to_string())?;
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
        num_concepts: num_concepts.max(1).min(10),
        auto_approve,
        checkpoint_context,
    };

    engine_streaming::run_pipeline_streaming(&state.http_client, &config, input, app_handle)
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
        let config = state.config.lock().map_err(|e| e.to_string())?;
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
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.ollama.endpoint.clone()
    };

    let models = ollama::list_models(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))?;

    Ok(models.into_iter().map(|m| m.name).collect())
}

#[tauri::command]
pub async fn check_ollama_health(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let endpoint = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.ollama.endpoint.clone()
    };

    ollama::check_health(&state.http_client, &endpoint)
        .await
        .map_err(|e| format!("{:#}", e))
}

fn parse_checkpoint_context_string(context_str: &str, checkpoint: &str) -> CheckpointContext {
    let mut ctx = CheckpointContext::default();
    ctx.checkpoint_name = checkpoint.to_string();

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
            // term_list will be populated from subsequent lines
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
