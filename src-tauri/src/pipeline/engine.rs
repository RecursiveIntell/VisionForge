use anyhow::{Context, Result};
use reqwest::Client;

use crate::pipeline::prompts::CheckpointContext;
use crate::pipeline::stages;
use crate::types::config::AppConfig;
use crate::types::pipeline::{
    ComposerOutput, ModelsUsed, PipelineConfig, PipelineResult, PipelineStages, PromptPair,
};

pub struct PipelineInput {
    pub idea: String,
    pub num_concepts: u32,
    pub auto_approve: bool,
    pub checkpoint_context: Option<CheckpointContext>,
}

pub async fn run_pipeline(
    client: &Client,
    config: &AppConfig,
    input: PipelineInput,
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
        let ideator_output = stages::run_ideator(
            client,
            endpoint,
            &models.ideator,
            &input.idea,
            input.num_concepts,
        )
        .await
        .context("Pipeline failed at Ideator stage")?;
        let mut concepts = ideator_output.output.clone();
        // Truncate to requested count — LLMs often generate more than asked
        concepts.truncate(input.num_concepts as usize);
        result_stages.ideator = Some(ideator_output);
        concepts
    } else {
        // Bypass: use the raw idea as a single concept
        vec![input.idea.clone()]
    };

    // Stage 2: Composer — enrich each concept
    let (composed, all_composer_outputs) = if stages_enabled[1] {
        let mut composed_descs = Vec::new();
        let mut all_outputs: Vec<ComposerOutput> = Vec::new();

        for (i, concept) in concepts.iter().enumerate() {
            let output =
                stages::run_composer(client, endpoint, &models.composer, concept, i)
                    .await
                    .with_context(|| format!("Pipeline failed at Composer stage for concept {}", i))?;
            composed_descs.push(output.output.clone());
            all_outputs.push(output);
        }

        (composed_descs, all_outputs)
    } else {
        // Bypass: pass concepts through as-is
        (concepts.clone(), Vec::new())
    };

    // Stage 3: Judge — rank composed descriptions (skip if only 1 concept)
    let (top_description, selected_index) = if stages_enabled[2] && composed.len() > 1 {
        let judge_output = stages::run_judge(
            client,
            endpoint,
            &models.judge,
            &input.idea,
            &composed,
        )
        .await
        .context("Pipeline failed at Judge stage")?;

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
        // Bypass: use first composed description
        (composed[0].clone(), 0)
    };

    // Store the composer output for the judge-selected concept (correct metadata)
    if stages_enabled[1] && !all_composer_outputs.is_empty() {
        let idx = selected_index.min(all_composer_outputs.len() - 1);
        result_stages.composer = Some(all_composer_outputs.into_iter().nth(idx).unwrap());
    }

    // Stage 4: Prompt Engineer — convert to SD prompts
    let prompt_pair = if stages_enabled[3] {
        let pe_output = stages::run_prompt_engineer(
            client,
            endpoint,
            &models.prompt_engineer,
            &top_description,
            input.checkpoint_context,
        )
        .await
        .context("Pipeline failed at Prompt Engineer stage")?;
        let pair = pe_output.output.clone();
        result_stages.prompt_engineer = Some(pe_output);
        pair
    } else {
        // Bypass: use description as positive prompt, default negative
        PromptPair {
            positive: top_description.clone(),
            negative: "lowres, bad anatomy, bad hands, text, watermark, blurry".to_string(),
        }
    };

    // Stage 5: Reviewer — sanity check
    if stages_enabled[4] {
        let reviewer_output = stages::run_reviewer(
            client,
            endpoint,
            &models.reviewer,
            &input.idea,
            &prompt_pair.positive,
            &prompt_pair.negative,
        )
        .await
        .context("Pipeline failed at Reviewer stage")?;
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

    Ok(PipelineResult {
        original_idea: input.idea,
        pipeline_config,
        stages: result_stages,
        user_edits: None,
        auto_approved: input.auto_approve,
        generation_settings: None,
    })
}

/// Run a single pipeline stage by name (for the run_pipeline_stage command)
pub async fn run_single_stage(
    client: &Client,
    endpoint: &str,
    stage: &str,
    model: &str,
    input: &str,
    checkpoint_context: Option<CheckpointContext>,
) -> Result<String> {
    match stage {
        "ideator" => {
            let output =
                stages::run_ideator(client, endpoint, model, input, 5).await?;
            serde_json::to_string(&output).context("Failed to serialize ideator output")
        }
        "composer" => {
            let output =
                stages::run_composer(client, endpoint, model, input, 0).await?;
            serde_json::to_string(&output).context("Failed to serialize composer output")
        }
        "judge" => {
            // Input should be JSON array of concepts
            let concepts: Vec<String> =
                serde_json::from_str(input).context("Judge input must be a JSON array of strings")?;
            let output =
                stages::run_judge(client, endpoint, model, "", &concepts).await?;
            serde_json::to_string(&output).context("Failed to serialize judge output")
        }
        "prompt_engineer" => {
            let output = stages::run_prompt_engineer(
                client,
                endpoint,
                model,
                input,
                checkpoint_context,
            )
            .await?;
            serde_json::to_string(&output)
                .context("Failed to serialize prompt engineer output")
        }
        "reviewer" => {
            // Input should be JSON with positive and negative fields
            let pair: PromptPair =
                serde_json::from_str(input).context("Reviewer input must be JSON with positive/negative fields")?;
            let output = stages::run_reviewer(
                client,
                endpoint,
                model,
                "",
                &pair.positive,
                &pair.negative,
            )
            .await?;
            serde_json::to_string(&output).context("Failed to serialize reviewer output")
        }
        _ => anyhow::bail!("Unknown pipeline stage: {}", stage),
    }
}

/// Get the final prompts from a pipeline result
pub fn get_final_prompts(result: &PipelineResult) -> Option<PromptPair> {
    result
        .stages
        .prompt_engineer
        .as_ref()
        .map(|pe| pe.output.clone())
}

/// Get the selected concept index from judge output (0 if no judge)
pub fn get_selected_concept(result: &PipelineResult) -> usize {
    result
        .stages
        .judge
        .as_ref()
        .and_then(|j| j.output.first())
        .map(|r| r.concept_index)
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "engine_test.rs"]
mod tests;
