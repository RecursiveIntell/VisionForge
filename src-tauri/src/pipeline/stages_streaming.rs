use anyhow::{Context, Result};
use reqwest::Client;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

use super::ollama::{self, ChatMessage};
use super::prompts::{self, CheckpointContext};
use super::stages::{
    backfill_rankings, parse_judge_rankings, parse_numbered_list, parse_prompt_pair,
    parse_reviewer_output,
};
use crate::types::pipeline::{
    ComposerOutput, IdeatorOutput, JudgeOutput, PromptEngineerOutput, ReviewerOutput,
};

pub async fn run_ideator_streaming<F: FnMut(&str)>(
    client: &Client,
    endpoint: &str,
    model: &str,
    idea: &str,
    num_concepts: u32,
    cancelled: Option<Arc<AtomicBool>>,
    on_token: F,
) -> Result<IdeatorOutput> {
    let start = Instant::now();
    let (system, user) = prompts::ideator_prompt(idea, num_concepts);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user,
        },
    ];
    let resp = ollama::chat_streaming_with_options(
        client, endpoint, model, &messages, false, &ollama::stage_options(1024), cancelled, on_token,
    )
    .await
    .context("Ideator stage failed")?;
    let concepts = parse_numbered_list(&resp.content);
    if concepts.is_empty() {
        anyhow::bail!(
            "Ideator returned no concepts. Raw response: {}",
            &resp.content[..resp.content.len().min(200)]
        );
    }
    Ok(IdeatorOutput {
        input: idea.to_string(),
        output: concepts,
        duration_ms: start.elapsed().as_millis() as u64,
        model: model.to_string(),
        tokens_in: resp.prompt_eval_count,
        tokens_out: resp.eval_count,
    })
}

pub async fn run_composer_streaming<F: FnMut(&str)>(
    client: &Client,
    endpoint: &str,
    model: &str,
    concept: &str,
    concept_index: usize,
    cancelled: Option<Arc<AtomicBool>>,
    on_token: F,
) -> Result<ComposerOutput> {
    let start = Instant::now();
    let (system, user) = prompts::composer_prompt(concept);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user,
        },
    ];
    let resp = ollama::chat_streaming_with_options(
        client,
        endpoint,
        model,
        &messages,
        false,
        &ollama::stage_options(2048),
        cancelled,
        on_token,
    )
    .await
    .context("Composer stage failed")?;
    let output = resp.content.trim().to_string();
    if output.is_empty() {
        anyhow::bail!("Composer returned empty output for concept: {}", concept);
    }
    Ok(ComposerOutput {
        input_concept_index: concept_index,
        input: concept.to_string(),
        output,
        duration_ms: start.elapsed().as_millis() as u64,
        model: model.to_string(),
        tokens_in: resp.prompt_eval_count,
        tokens_out: resp.eval_count,
    })
}

pub async fn run_judge_streaming<F: FnMut(&str)>(
    client: &Client,
    endpoint: &str,
    model: &str,
    original_idea: &str,
    concepts: &[String],
    cancelled: Option<Arc<AtomicBool>>,
    on_token: F,
) -> Result<JudgeOutput> {
    let start = Instant::now();
    let (system, user) = prompts::judge_prompt(original_idea, concepts);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user,
        },
    ];
    let resp = ollama::chat_streaming_with_options(
        client, endpoint, model, &messages, true, &ollama::stage_options(1024), cancelled, on_token,
    )
    .await
    .context("Judge stage failed")?;
    let rankings =
        parse_judge_rankings(&resp.content).context("Failed to parse Judge output as rankings")?;
    if rankings.is_empty() {
        anyhow::bail!(
            "Judge returned no rankings. Raw response: {}",
            &resp.content[..resp.content.len().min(200)]
        );
    }
    let rankings = backfill_rankings(rankings, concepts.len());
    Ok(JudgeOutput {
        input: concepts.to_vec(),
        output: rankings,
        duration_ms: start.elapsed().as_millis() as u64,
        model: model.to_string(),
    })
}

pub async fn run_prompt_engineer_streaming<F: FnMut(&str)>(
    client: &Client,
    endpoint: &str,
    model: &str,
    description: &str,
    checkpoint_ctx: Option<CheckpointContext>,
    cancelled: Option<Arc<AtomicBool>>,
    on_token: F,
) -> Result<PromptEngineerOutput> {
    let start = Instant::now();
    let ctx = checkpoint_ctx.unwrap_or_default();
    let checkpoint_context_str = format!(
        "Checkpoint: {}, Base: {}, Strengths: {}, Weaknesses: {}",
        ctx.checkpoint_name, ctx.base_model, ctx.strengths, ctx.weaknesses
    );
    let (system, user) = prompts::prompt_engineer_prompt(description, &ctx);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user,
        },
    ];
    let resp = ollama::chat_streaming_with_options(
        client, endpoint, model, &messages, true, &ollama::stage_options(1024), cancelled, on_token,
    )
    .await
    .context("Prompt Engineer stage failed")?;
    let pair = parse_prompt_pair(&resp.content)
        .context("Failed to parse Prompt Engineer output as positive/negative pair")?;
    Ok(PromptEngineerOutput {
        input: description.to_string(),
        checkpoint_context: Some(checkpoint_context_str),
        output: pair,
        duration_ms: start.elapsed().as_millis() as u64,
        model: model.to_string(),
        tokens_in: resp.prompt_eval_count,
        tokens_out: resp.eval_count,
    })
}

pub async fn run_reviewer_streaming<F: FnMut(&str)>(
    client: &Client,
    endpoint: &str,
    model: &str,
    original_idea: &str,
    positive: &str,
    negative: &str,
    cancelled: Option<Arc<AtomicBool>>,
    on_token: F,
) -> Result<ReviewerOutput> {
    let start = Instant::now();
    let (system, user) = prompts::reviewer_prompt(original_idea, positive, negative);
    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: system,
        },
        ChatMessage {
            role: "user".to_string(),
            content: user,
        },
    ];
    let resp = ollama::chat_streaming_with_options(
        client, endpoint, model, &messages, true, &ollama::stage_options(1024), cancelled, on_token,
    )
    .await
    .context("Reviewer stage failed")?;
    let output = parse_reviewer_output(&resp.content).context("Failed to parse Reviewer output")?;
    Ok(ReviewerOutput {
        approved: output.approved,
        issues: output.issues,
        suggested_positive: output.suggested_positive,
        suggested_negative: output.suggested_negative,
        duration_ms: start.elapsed().as_millis() as u64,
        model: model.to_string(),
    })
}
