use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn normalize_endpoint(endpoint: &str) -> &str {
    endpoint.trim_end_matches('/')
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub total_duration_ns: Option<u64>,
    pub prompt_eval_count: Option<u64>,
    pub eval_count: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct OllamaOptions {
    pub num_predict: Option<u32>,
    pub repeat_penalty: Option<f64>,
    pub repeat_last_n: Option<u32>,
    /// Control thinking/reasoning mode for supported models.
    /// Some(true) = force thinking on, Some(false) = force thinking off,
    /// None = omit parameter (model uses its default behavior).
    pub think: Option<bool>,
}

/// Default options for pipeline stages: repeat_penalty=1.2, repeat_last_n=128, with
/// a per-stage num_predict cap to prevent runaway generation.
pub fn stage_options(num_predict: u32) -> OllamaOptions {
    OllamaOptions {
        num_predict: Some(num_predict),
        repeat_penalty: Some(1.2),
        repeat_last_n: Some(128),
        think: None,
    }
}

/// Create stage options with an explicit thinking mode.
pub fn stage_options_with_thinking(num_predict: u32, think: Option<bool>) -> OllamaOptions {
    OllamaOptions {
        num_predict: Some(num_predict),
        repeat_penalty: Some(1.2),
        repeat_last_n: Some(128),
        think,
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: Option<u64>,
    pub digest: Option<String>,
}

pub async fn check_health(client: &Client, endpoint: &str) -> Result<bool> {
    let endpoint = normalize_endpoint(endpoint);
    let resp = client
        .get(endpoint)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .with_context(|| {
            format!(
                "Cannot connect to Ollama at {} — is the service running?",
                endpoint
            )
        })?;
    Ok(resp.status().is_success())
}

pub async fn list_models(client: &Client, endpoint: &str) -> Result<Vec<OllamaModel>> {
    let endpoint = normalize_endpoint(endpoint);
    let url = format!("{}/api/tags", endpoint);
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .with_context(|| format!("Cannot connect to Ollama at {}", endpoint))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned {} listing models: {}", status, body);
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse Ollama /api/tags response")?;

    let models = json
        .get("models")
        .and_then(|m| m.as_array())
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| {
            let name = m.get("name")?.as_str()?.to_string();
            let size = m.get("size").and_then(|s| s.as_u64());
            let digest = m.get("digest").and_then(|d| d.as_str().map(String::from));
            Some(OllamaModel { name, size, digest })
        })
        .collect();

    Ok(models)
}

/// Built-in list of model family name patterns known to support thinking mode.
/// Matched case-insensitively against the model name before the ":" tag separator.
const KNOWN_THINKING_MODEL_PATTERNS: &[&str] = &[
    "qwen3",
    "qwq",
    "deepseek-r1",
    "phi4-reasoning",
    "phi-4-reasoning",
    "marco-o1",
    "gpt-oss",
    "skywork-or1",
    "smallthinker",
    "granite3-moe",
];

/// Check if a model name matches a known thinking model pattern.
pub fn is_known_thinking_model(model_name: &str) -> bool {
    let base = model_name.split(':').next().unwrap_or(model_name);
    let base_lower = base.to_lowercase();
    KNOWN_THINKING_MODEL_PATTERNS
        .iter()
        .any(|pattern| base_lower.contains(&pattern.to_lowercase()))
}

/// Probe a specific model via `/api/show` to check if it supports thinking.
/// Falls back to the known-models list if the probe fails.
pub async fn probe_model_thinking(
    client: &Client,
    endpoint: &str,
    model_name: &str,
) -> bool {
    if is_known_thinking_model(model_name) {
        return true;
    }

    let endpoint = normalize_endpoint(endpoint);
    let url = format!("{}/api/show", endpoint);
    let body = serde_json::json!({ "name": model_name });

    let resp = match client
        .post(&url)
        .timeout(Duration::from_secs(5))
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return false,
    };

    let json: Value = match resp.json().await {
        Ok(j) => j,
        Err(_) => return false,
    };

    if let Some(template) = json.get("template").and_then(|t| t.as_str()) {
        let tpl_lower = template.to_lowercase();
        if tpl_lower.contains("<think>")
            || tpl_lower.contains("thinking")
            || tpl_lower.contains(".thinking")
        {
            return true;
        }
    }

    if let Some(caps) = json.get("capabilities").and_then(|c| c.as_array()) {
        for cap in caps {
            if let Some(s) = cap.as_str() {
                if s.eq_ignore_ascii_case("thinking") {
                    return true;
                }
            }
        }
    }

    false
}

/// Batch-detect thinking capability for all provided models.
pub async fn detect_thinking_models(
    client: &Client,
    endpoint: &str,
    model_names: &[String],
) -> Vec<String> {
    let mut thinking_models = Vec::new();
    for name in model_names {
        if probe_model_thinking(client, endpoint, name).await {
            thinking_models.push(name.clone());
        }
    }
    thinking_models
}

pub async fn chat(
    client: &Client,
    endpoint: &str,
    model: &str,
    messages: &[ChatMessage],
    format_json: bool,
) -> Result<ChatResponse> {
    chat_with_options(client, endpoint, model, messages, format_json, &OllamaOptions::default()).await
}

pub async fn chat_with_options(
    client: &Client,
    endpoint: &str,
    model: &str,
    messages: &[ChatMessage],
    format_json: bool,
    opts: &OllamaOptions,
) -> Result<ChatResponse> {
    let endpoint = normalize_endpoint(endpoint);
    let url = format!("{}/api/chat", endpoint);

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false,
        "keep_alive": "30m",
    });

    if format_json {
        body["format"] = serde_json::json!("json");
    }

    let options = build_options(opts);
    if !options.is_empty() {
        body["options"] = serde_json::json!(options);
    }

    // Apply thinking mode — this is a top-level parameter, not inside "options"
    if let Some(think) = opts.think {
        body["think"] = serde_json::json!(think);
    }

    let resp = client
        .post(&url)
        .timeout(Duration::from_secs(300))
        .json(&body)
        .send()
        .await
        .with_context(|| {
            format!(
                "Cannot connect to Ollama at {} — is the service running?",
                endpoint
            )
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned {} for chat: {}", status, body);
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse Ollama chat response")?;

    if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
        anyhow::bail!("Ollama error: {}", error);
    }

    let content = json
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    let total_duration_ns = json.get("total_duration").and_then(|v| v.as_u64());
    let prompt_eval_count = json.get("prompt_eval_count").and_then(|v| v.as_u64());
    let eval_count = json.get("eval_count").and_then(|v| v.as_u64());

    Ok(ChatResponse {
        content,
        total_duration_ns,
        prompt_eval_count,
        eval_count,
    })
}

/// Streaming variant of chat that calls `on_token` for each token chunk.
/// Returns the full accumulated response when done.
pub async fn chat_streaming<F>(
    client: &Client,
    endpoint: &str,
    model: &str,
    messages: &[ChatMessage],
    format_json: bool,
    on_token: F,
) -> Result<ChatResponse>
where
    F: FnMut(&str),
{
    chat_streaming_with_options(
        client,
        endpoint,
        model,
        messages,
        format_json,
        &OllamaOptions::default(),
        None,
        on_token,
    )
    .await
}

pub async fn chat_streaming_with_options<F>(
    client: &Client,
    endpoint: &str,
    model: &str,
    messages: &[ChatMessage],
    format_json: bool,
    opts: &OllamaOptions,
    cancelled: Option<Arc<AtomicBool>>,
    mut on_token: F,
) -> Result<ChatResponse>
where
    F: FnMut(&str),
{
    let endpoint = normalize_endpoint(endpoint);
    let url = format!("{}/api/chat", endpoint);

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "keep_alive": "30m",
    });

    if format_json {
        body["format"] = serde_json::json!("json");
    }

    let options = build_options(opts);
    if !options.is_empty() {
        body["options"] = serde_json::json!(options);
    }

    // Apply thinking mode — this is a top-level parameter, not inside "options"
    if let Some(think) = opts.think {
        body["think"] = serde_json::json!(think);
    }

    let resp = client
        .post(&url)
        .timeout(Duration::from_secs(300))
        .json(&body)
        .send()
        .await
        .with_context(|| {
            format!(
                "Cannot connect to Ollama at {} — is the service running?",
                endpoint
            )
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned {} for chat: {}", status, body);
    }

    let mut stream = resp.bytes_stream();
    let mut accumulated_content = String::new();
    let mut total_duration_ns: Option<u64> = None;
    let mut prompt_eval_count: Option<u64> = None;
    let mut eval_count: Option<u64> = None;
    let mut line_buffer = String::new();
    const MAX_BUFFER_SIZE: usize = 1_048_576; // 1MB

    while let Some(chunk) = stream.next().await {
        if let Some(ref flag) = cancelled {
            if flag.load(Ordering::Relaxed) {
                anyhow::bail!("Pipeline cancelled by user");
            }
        }
        let chunk = chunk.context("Error reading stream chunk")?;
        let text = String::from_utf8_lossy(&chunk);
        line_buffer.push_str(&text);

        // Guard against unbounded buffer accumulation
        if line_buffer.len() > MAX_BUFFER_SIZE {
            anyhow::bail!(
                "Ollama response exceeded maximum buffer size ({}MB). Response may be malformed.",
                MAX_BUFFER_SIZE / 1_048_576
            );
        }

        // Ollama sends newline-delimited JSON
        while let Some(newline_pos) = line_buffer.find('\n') {
            let line = line_buffer[..newline_pos].trim().to_string();
            line_buffer = line_buffer[newline_pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if let Ok(json) = serde_json::from_str::<Value>(&line) {
                if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
                    anyhow::bail!("Ollama error: {}", error);
                }

                if let Some(content) = json
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                {
                    if !content.is_empty() {
                        accumulated_content.push_str(content);
                        if accumulated_content.len() > MAX_BUFFER_SIZE {
                            anyhow::bail!("Ollama accumulated response exceeded {}MB limit", MAX_BUFFER_SIZE / 1_048_576);
                        }
                        on_token(content);
                    }
                }

                if json.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                    total_duration_ns = json.get("total_duration").and_then(|v| v.as_u64());
                    prompt_eval_count = json.get("prompt_eval_count").and_then(|v| v.as_u64());
                    eval_count = json.get("eval_count").and_then(|v| v.as_u64());
                }
            }
        }
    }

    // Process any remaining buffer
    let remaining = line_buffer.trim().to_string();
    if !remaining.is_empty() {
        if let Ok(json) = serde_json::from_str::<Value>(&remaining) {
            if let Some(content) = json
                .get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
            {
                if !content.is_empty() {
                    accumulated_content.push_str(content);
                    on_token(content);
                }
            }
            if json.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
                total_duration_ns = json.get("total_duration").and_then(|v| v.as_u64());
                prompt_eval_count = json.get("prompt_eval_count").and_then(|v| v.as_u64());
                eval_count = json.get("eval_count").and_then(|v| v.as_u64());
            }
        }
    }

    Ok(ChatResponse {
        content: accumulated_content,
        total_duration_ns,
        prompt_eval_count,
        eval_count,
    })
}

fn build_options(opts: &OllamaOptions) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    if let Some(n) = opts.num_predict {
        map.insert("num_predict".into(), Value::Number(n.into()));
    }
    if let Some(rp) = opts.repeat_penalty {
        map.insert(
            "repeat_penalty".into(),
            serde_json::Number::from_f64(rp)
                .map(Value::Number)
                .unwrap_or(Value::Null),
        );
    }
    if let Some(rn) = opts.repeat_last_n {
        map.insert("repeat_last_n".into(), Value::Number(rn.into()));
    }
    map
}

/// Unload a model from VRAM by setting keep_alive to 0.
pub async fn unload_model(client: &Client, endpoint: &str, model: &str) -> Result<()> {
    let endpoint = normalize_endpoint(endpoint);
    let url = format!("{}/api/generate", endpoint);
    let body = serde_json::json!({
        "model": model,
        "prompt": "",
        "keep_alive": 0,
    });

    let _ = client
        .post(&url)
        .timeout(Duration::from_secs(10))
        .json(&body)
        .send()
        .await;

    Ok(())
}

pub async fn generate(
    client: &Client,
    endpoint: &str,
    model: &str,
    prompt: &str,
    format_json: bool,
) -> Result<ChatResponse> {
    let endpoint = normalize_endpoint(endpoint);
    let url = format!("{}/api/generate", endpoint);

    let mut body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "keep_alive": "30m",
    });

    if format_json {
        body["format"] = serde_json::json!("json");
    }

    let resp = client
        .post(&url)
        .timeout(Duration::from_secs(300))
        .json(&body)
        .send()
        .await
        .with_context(|| {
            format!(
                "Cannot connect to Ollama at {} — is the service running?",
                endpoint
            )
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned {} for generate: {}", status, body);
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse Ollama generate response")?;

    if let Some(error) = json.get("error").and_then(|v| v.as_str()) {
        anyhow::bail!("Ollama error: {}", error);
    }

    let content = json
        .get("response")
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    let total_duration_ns = json.get("total_duration").and_then(|v| v.as_u64());
    let prompt_eval_count = json.get("prompt_eval_count").and_then(|v| v.as_u64());
    let eval_count = json.get("eval_count").and_then(|v| v.as_u64());

    Ok(ChatResponse {
        content,
        total_duration_ns,
        prompt_eval_count,
        eval_count,
    })
}

#[cfg(test)]
#[path = "ollama_test.rs"]
mod tests;
