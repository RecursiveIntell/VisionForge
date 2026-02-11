use anyhow::{Context, Result};
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

use super::engine::PipelineInput;
use super::stages_streaming;
use crate::types::config::AppConfig;
use crate::types::pipeline::{
    ComposerOutput, ModelsUsed, PipelineConfig, PipelineResult, PipelineStageCompleteEvent,
    PipelineStageStartEvent, PipelineStageTokenEvent, PipelineStages, PromptPair,
};

fn check_cancelled(cancelled: &Arc<AtomicBool>) -> Result<()> {
    if cancelled.load(Ordering::Relaxed) {
        anyhow::bail!("Pipeline cancelled by user");
    }
    Ok(())
}

pub async fn run_pipeline_streaming(
    client: &Client,
    config: &AppConfig,
    input: PipelineInput,
    app_handle: AppHandle,
    cancelled: Arc<AtomicBool>,
) -> Result<PipelineResult> {
    let pipeline = &config.pipeline;
    let models = &config.models;
    let endpoint = &config.ollama.endpoint;

    let stages_enabled = [
        pipeline.enable_ideator,
        pipeline.enable_composer,
        pipeline.enable_judge,
        pipeline.enable_prompt_engineer,
        pipeline.enable_reviewer,
    ];

    let models_used = ModelsUsed {
        ideator: if stages_enabled[0] {
            Some(models.ideator.clone())
        } else {
            None
        },
        composer: if stages_enabled[1] {
            Some(models.composer.clone())
        } else {
            None
        },
        judge: if stages_enabled[2] {
            Some(models.judge.clone())
        } else {
            None
        },
        prompt_engineer: if stages_enabled[3] {
            Some(models.prompt_engineer.clone())
        } else {
            None
        },
        reviewer: if stages_enabled[4] {
            Some(models.reviewer.clone())
        } else {
            None
        },
    };

    let pipeline_config = PipelineConfig {
        stages_enabled,
        models_used,
    };

    let mut result_stages = PipelineStages::default();

    // Stage 1: Ideator
    let concepts = if stages_enabled[0] {
        check_cancelled(&cancelled)?;
        let _ = app_handle.emit(
            "pipeline:stage_start",
            PipelineStageStartEvent {
                stage: "ideator".into(),
                model: models.ideator.clone(),
            },
        );
        let ah = app_handle.clone();
        let ideator_output = stages_streaming::run_ideator_streaming(
            client,
            endpoint,
            &models.ideator,
            &input.idea,
            input.num_concepts,
            Some(cancelled.clone()),
            move |token: &str| {
                let _ = ah.emit(
                    "pipeline:stage_token",
                    PipelineStageTokenEvent {
                        stage: "ideator".into(),
                        token: token.to_string(),
                    },
                );
            },
        )
        .await
        .context("Pipeline failed at Ideator stage")?;
        let _ = app_handle.emit(
            "pipeline:stage_complete",
            PipelineStageCompleteEvent {
                stage: "ideator".into(),
                duration_ms: ideator_output.duration_ms,
            },
        );
        let mut concepts = ideator_output.output.clone();
        // Truncate to requested count — LLMs often generate more than asked
        concepts.truncate(input.num_concepts as usize);
        result_stages.ideator = Some(ideator_output);
        concepts
    } else {
        vec![input.idea.clone()]
    };

    // Stage 2: Composer — enrich each concept
    let (composed, all_composer_outputs) = if stages_enabled[1] {
        check_cancelled(&cancelled)?;
        let _ = app_handle.emit(
            "pipeline:stage_start",
            PipelineStageStartEvent {
                stage: "composer".into(),
                model: models.composer.clone(),
            },
        );
        let mut composed_descs = Vec::new();
        let mut all_outputs: Vec<ComposerOutput> = Vec::new();

        for (i, concept) in concepts.iter().enumerate() {
            check_cancelled(&cancelled)?;
            let ah = app_handle.clone();
            let output = stages_streaming::run_composer_streaming(
                client,
                endpoint,
                &models.composer,
                concept,
                i,
                Some(cancelled.clone()),
                move |token: &str| {
                    let _ = ah.emit(
                        "pipeline:stage_token",
                        PipelineStageTokenEvent {
                            stage: "composer".into(),
                            token: token.to_string(),
                        },
                    );
                },
            )
            .await
            .with_context(|| format!("Pipeline failed at Composer stage for concept {}", i))?;
            composed_descs.push(output.output.clone());
            all_outputs.push(output);
        }

        let duration_ms = all_outputs.last().map(|c| c.duration_ms).unwrap_or(0);
        let _ = app_handle.emit(
            "pipeline:stage_complete",
            PipelineStageCompleteEvent {
                stage: "composer".into(),
                duration_ms,
            },
        );
        (composed_descs, all_outputs)
    } else {
        (concepts.clone(), Vec::new())
    };

    // Stage 3: Judge — rank composed descriptions (skip if only 1 concept)
    let (top_description, selected_index) = if stages_enabled[2] && composed.len() > 1 {
        check_cancelled(&cancelled)?;
        let _ = app_handle.emit(
            "pipeline:stage_start",
            PipelineStageStartEvent {
                stage: "judge".into(),
                model: models.judge.clone(),
            },
        );
        let ah = app_handle.clone();
        let judge_output = stages_streaming::run_judge_streaming(
            client,
            endpoint,
            &models.judge,
            &input.idea,
            &composed,
            Some(cancelled.clone()),
            move |token: &str| {
                let _ = ah.emit(
                    "pipeline:stage_token",
                    PipelineStageTokenEvent {
                        stage: "judge".into(),
                        token: token.to_string(),
                    },
                );
            },
        )
        .await
        .context("Pipeline failed at Judge stage")?;

        let _ = app_handle.emit(
            "pipeline:stage_complete",
            PipelineStageCompleteEvent {
                stage: "judge".into(),
                duration_ms: judge_output.duration_ms,
            },
        );

        let top_index = judge_output
            .output
            .first()
            .map(|r| r.concept_index)
            .unwrap_or(0);
        let top_desc = composed
            .get(top_index)
            .cloned()
            .unwrap_or_else(|| composed[0].clone());

        result_stages.judge = Some(judge_output);
        (top_desc, top_index)
    } else {
        (composed[0].clone(), 0)
    };

    // Store the composer output for the judge-selected concept (correct metadata)
    if stages_enabled[1] && !all_composer_outputs.is_empty() {
        let idx = selected_index.min(all_composer_outputs.len() - 1);
        result_stages.composer = Some(all_composer_outputs.into_iter().nth(idx).unwrap());
    }

    // Stage 4: Prompt Engineer — convert to SD prompts
    let prompt_pair = if stages_enabled[3] {
        check_cancelled(&cancelled)?;
        let _ = app_handle.emit(
            "pipeline:stage_start",
            PipelineStageStartEvent {
                stage: "promptEngineer".into(),
                model: models.prompt_engineer.clone(),
            },
        );
        let ah = app_handle.clone();
        let pe_output = stages_streaming::run_prompt_engineer_streaming(
            client,
            endpoint,
            &models.prompt_engineer,
            &top_description,
            input.checkpoint_context,
            Some(cancelled.clone()),
            move |token: &str| {
                let _ = ah.emit(
                    "pipeline:stage_token",
                    PipelineStageTokenEvent {
                        stage: "promptEngineer".into(),
                        token: token.to_string(),
                    },
                );
            },
        )
        .await
        .context("Pipeline failed at Prompt Engineer stage")?;

        let _ = app_handle.emit(
            "pipeline:stage_complete",
            PipelineStageCompleteEvent {
                stage: "promptEngineer".into(),
                duration_ms: pe_output.duration_ms,
            },
        );
        let pair = pe_output.output.clone();
        result_stages.prompt_engineer = Some(pe_output);
        pair
    } else {
        PromptPair {
            positive: top_description.clone(),
            negative: "lowres, bad anatomy, bad hands, text, watermark, blurry".to_string(),
        }
    };

    // Stage 5: Reviewer — sanity check
    if stages_enabled[4] {
        check_cancelled(&cancelled)?;
        let _ = app_handle.emit(
            "pipeline:stage_start",
            PipelineStageStartEvent {
                stage: "reviewer".into(),
                model: models.reviewer.clone(),
            },
        );
        let ah = app_handle.clone();
        let reviewer_output = stages_streaming::run_reviewer_streaming(
            client,
            endpoint,
            &models.reviewer,
            &input.idea,
            &prompt_pair.positive,
            &prompt_pair.negative,
            Some(cancelled.clone()),
            move |token: &str| {
                let _ = ah.emit(
                    "pipeline:stage_token",
                    PipelineStageTokenEvent {
                        stage: "reviewer".into(),
                        token: token.to_string(),
                    },
                );
            },
        )
        .await
        .context("Pipeline failed at Reviewer stage")?;

        let _ = app_handle.emit(
            "pipeline:stage_complete",
            PipelineStageCompleteEvent {
                stage: "reviewer".into(),
                duration_ms: reviewer_output.duration_ms,
            },
        );
        result_stages.reviewer = Some(reviewer_output);
    }

    // If reviewer suggested changes and was not approved, update the prompt engineer output
    if let Some(ref reviewer) = result_stages.reviewer {
        if !reviewer.approved {
            if let Some(ref mut pe) = result_stages.prompt_engineer {
                if let Some(ref suggested_pos) = reviewer.suggested_positive {
                    pe.output.positive = suggested_pos.clone();
                }
                if let Some(ref suggested_neg) = reviewer.suggested_negative {
                    pe.output.negative = suggested_neg.clone();
                }
            }
        }
    }

    // Unload the last used model to free VRAM for Stable Diffusion
    let last_model = if stages_enabled[4] {
        Some(&models.reviewer)
    } else if stages_enabled[3] {
        Some(&models.prompt_engineer)
    } else if stages_enabled[2] && composed.len() > 1 {
        Some(&models.judge)
    } else if stages_enabled[1] {
        Some(&models.composer)
    } else if stages_enabled[0] {
        Some(&models.ideator)
    } else {
        None
    };
    if let Some(model) = last_model {
        let _ = super::ollama::unload_model(client, endpoint, model).await;
    }

    Ok(PipelineResult {
        original_idea: input.idea,
        pipeline_config,
        stages: result_stages,
        user_edits: None,
        auto_approved: input.auto_approve,
        generation_settings: None,
    })
}
