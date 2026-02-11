use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Instant;

use crate::pipeline::ollama::{self, ChatMessage};
use crate::pipeline::prompts::{self, CheckpointContext};
use crate::types::pipeline::{
    ComposerOutput, IdeatorOutput, JudgeOutput, JudgeRanking, PromptEngineerOutput, PromptPair,
    ReviewerOutput,
};

pub async fn run_ideator(
    client: &Client,
    endpoint: &str,
    model: &str,
    idea: &str,
    num_concepts: u32,
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

    let resp = ollama::chat(client, endpoint, model, &messages, false)
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

pub async fn run_composer(
    client: &Client,
    endpoint: &str,
    model: &str,
    concept: &str,
    concept_index: usize,
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

    let resp = ollama::chat(client, endpoint, model, &messages, false)
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

pub async fn run_judge(
    client: &Client,
    endpoint: &str,
    model: &str,
    original_idea: &str,
    concepts: &[String],
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

    let resp = ollama::chat(client, endpoint, model, &messages, true)
        .await
        .context("Judge stage failed")?;

    let rankings = parse_judge_rankings(&resp.content)
        .context("Failed to parse Judge output as rankings")?;

    if rankings.is_empty() {
        anyhow::bail!(
            "Judge returned no rankings. Raw response: {}",
            &resp.content[..resp.content.len().min(200)]
        );
    }

    Ok(JudgeOutput {
        input: concepts.to_vec(),
        output: rankings,
        duration_ms: start.elapsed().as_millis() as u64,
        model: model.to_string(),
    })
}

pub async fn run_prompt_engineer(
    client: &Client,
    endpoint: &str,
    model: &str,
    description: &str,
    checkpoint_ctx: Option<CheckpointContext>,
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

    let resp = ollama::chat(client, endpoint, model, &messages, true)
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

pub async fn run_reviewer(
    client: &Client,
    endpoint: &str,
    model: &str,
    original_idea: &str,
    positive: &str,
    negative: &str,
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

    let resp = ollama::chat(client, endpoint, model, &messages, true)
        .await
        .context("Reviewer stage failed")?;

    let output = parse_reviewer_output(&resp.content)
        .context("Failed to parse Reviewer output")?;

    Ok(ReviewerOutput {
        approved: output.approved,
        issues: output.issues,
        suggested_positive: output.suggested_positive,
        suggested_negative: output.suggested_negative,
        duration_ms: start.elapsed().as_millis() as u64,
        model: model.to_string(),
    })
}

pub(super) fn parse_numbered_list(text: &str) -> Vec<String> {
    let mut concepts = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if line starts a new numbered item (e.g., "1. ", "2. ", "1) ", "2) ")
        // Only match digits immediately followed by ". " or ") " at the start
        let prefix_end = trimmed
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(trimmed.len());
        let after_digits = &trimmed[prefix_end..];
        let is_new_item = prefix_end > 0
            && (after_digits.starts_with(". ") || after_digits.starts_with(") "));

        if is_new_item {
            if !current.is_empty() {
                concepts.push(current.trim().to_string());
            }
            // Strip the number prefix (digits + delimiter)
            let content = &trimmed[prefix_end + 2..];
            current = content.trim().to_string();
        } else {
            // Continuation of previous item
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(trimmed);
        }
    }

    if !current.is_empty() {
        concepts.push(current.trim().to_string());
    }

    concepts
}

pub(super) fn parse_judge_rankings(text: &str) -> Result<Vec<JudgeRanking>> {
    let json = extract_json_from_text(text)?;
    let arr = json
        .as_array()
        .context("Judge output is not a JSON array")?;

    let mut rankings = Vec::new();
    for item in arr {
        let rank = item
            .get("rank")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let concept_index = item
            .get("concept_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let score = item
            .get("score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;
        let reasoning = item
            .get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        rankings.push(JudgeRanking {
            rank,
            concept_index,
            score,
            reasoning,
        });
    }

    rankings.sort_by_key(|r| r.rank);
    Ok(rankings)
}

pub(super) fn parse_prompt_pair(text: &str) -> Result<PromptPair> {
    let json = extract_json_from_text(text)?;

    let positive = json
        .get("positive")
        .and_then(|v| v.as_str())
        .context("Missing 'positive' field in Prompt Engineer output")?
        .to_string();

    let negative = json
        .get("negative")
        .and_then(|v| v.as_str())
        .context("Missing 'negative' field in Prompt Engineer output")?
        .to_string();

    Ok(PromptPair { positive, negative })
}

pub(super) struct ParsedReviewer {
    pub(super) approved: bool,
    pub(super) issues: Option<Vec<String>>,
    pub(super) suggested_positive: Option<String>,
    pub(super) suggested_negative: Option<String>,
}

pub(super) fn parse_reviewer_output(text: &str) -> Result<ParsedReviewer> {
    let json = extract_json_from_text(text)?;

    let approved = json
        .get("approved")
        .and_then(|v| v.as_bool())
        .unwrap_or(true); // Default to approved if parsing fails

    let issues = json.get("issues").and_then(|v| {
        v.as_array().map(|arr| {
            arr.iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
    });

    let suggested_positive = json
        .get("suggested_positive")
        .and_then(|v| v.as_str())
        .map(String::from);

    let suggested_negative = json
        .get("suggested_negative")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(ParsedReviewer {
        approved,
        issues,
        suggested_positive,
        suggested_negative,
    })
}

pub(super) fn extract_json_from_text(text: &str) -> Result<Value> {
    // Try direct parse first
    if let Ok(json) = serde_json::from_str::<Value>(text.trim()) {
        return Ok(json);
    }

    // Try to find JSON array or object in the text
    let trimmed = text.trim();
    for (start_char, end_char) in [('[', ']'), ('{', '}')] {
        if let Some(start) = trimmed.find(start_char) {
            if let Some(end) = trimmed.rfind(end_char) {
                if end > start {
                    let candidate = &trimmed[start..=end];
                    if let Ok(json) = serde_json::from_str::<Value>(candidate) {
                        return Ok(json);
                    }
                }
            }
        }
    }

    anyhow::bail!(
        "Could not extract valid JSON from LLM response: {}",
        &text[..text.len().min(200)]
    )
}

#[cfg(test)]
#[path = "stages_test.rs"]
mod tests;
