use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use crate::types::generation::{GenerationStatus, GenerationStatusKind};

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub current_step: u32,
    pub total_steps: u32,
}

pub async fn check_health(client: &Client, endpoint: &str) -> Result<bool> {
    let url = format!("{}/system_stats", endpoint);
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .with_context(|| {
            format!(
                "Cannot connect to ComfyUI at {} — is the service running?",
                endpoint
            )
        })?;
    Ok(resp.status().is_success())
}

pub async fn queue_prompt(
    client: &Client,
    endpoint: &str,
    workflow: &Value,
    client_id: &str,
) -> Result<String> {
    let url = format!("{}/prompt", endpoint);

    let body = serde_json::json!({
        "prompt": workflow,
        "client_id": client_id,
    });

    let resp = client
        .post(&url)
        .timeout(Duration::from_secs(30))
        .json(&body)
        .send()
        .await
        .with_context(|| {
            format!(
                "Cannot connect to ComfyUI at {} — is the service running?",
                endpoint
            )
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        anyhow::bail!(
            "ComfyUI returned {} when queuing prompt: {}",
            status,
            body_text
        );
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse ComfyUI /prompt response")?;

    // Check for node_errors
    if let Some(errors) = json.get("node_errors") {
        if let Some(obj) = errors.as_object() {
            if !obj.is_empty() {
                anyhow::bail!(
                    "ComfyUI workflow has node errors: {}",
                    serde_json::to_string_pretty(errors).unwrap_or_default()
                );
            }
        }
    }

    let prompt_id = json
        .get("prompt_id")
        .and_then(|v| v.as_str())
        .context("ComfyUI response missing prompt_id")?
        .to_string();

    Ok(prompt_id)
}

pub async fn get_history(
    client: &Client,
    endpoint: &str,
    prompt_id: &str,
) -> Result<Option<PromptHistory>> {
    let url = format!("{}/history/{}", endpoint, prompt_id);

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch ComfyUI history")?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse ComfyUI history response")?;

    let entry = match json.get(prompt_id) {
        Some(e) => e,
        None => return Ok(None),
    };

    let status_str = entry
        .pointer("/status/status_str")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let completed = entry
        .pointer("/status/completed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut image_filenames = Vec::new();
    if let Some(outputs) = entry.get("outputs").and_then(|o| o.as_object()) {
        for (_node_id, node_output) in outputs {
            if let Some(images) = node_output.get("images").and_then(|i| i.as_array()) {
                for img in images {
                    if let Some(filename) = img.get("filename").and_then(|f| f.as_str()) {
                        let subfolder = img.get("subfolder").and_then(|s| s.as_str()).unwrap_or("");
                        let img_type = img.get("type").and_then(|t| t.as_str()).unwrap_or("output");
                        image_filenames.push(ImageRef {
                            filename: filename.to_string(),
                            subfolder: subfolder.to_string(),
                            img_type: img_type.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(Some(PromptHistory {
        status: status_str.to_string(),
        completed,
        image_filenames,
    }))
}

pub async fn get_image(
    client: &Client,
    endpoint: &str,
    filename: &str,
    subfolder: &str,
    img_type: &str,
) -> Result<Vec<u8>> {
    let url = reqwest::Url::parse_with_params(
        &format!("{}/view", endpoint),
        &[
            ("filename", filename),
            ("subfolder", subfolder),
            ("type", img_type),
        ],
    )
    .with_context(|| format!("Failed to build URL for image {}", filename))?;

    let resp = client
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .with_context(|| format!("Failed to fetch image {} from ComfyUI", filename))?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "ComfyUI returned {} when fetching image {}",
            resp.status(),
            filename
        );
    }

    let bytes = resp
        .bytes()
        .await
        .context("Failed to read image bytes from ComfyUI")?;

    Ok(bytes.to_vec())
}

pub async fn get_queue_status(client: &Client, endpoint: &str) -> Result<QueueStatus> {
    let url = format!("{}/queue", endpoint);

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .context("Failed to fetch ComfyUI queue status")?;

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse ComfyUI queue response")?;

    let running = json
        .get("queue_running")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u32)
        .unwrap_or(0);

    let pending = json
        .get("queue_pending")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u32)
        .unwrap_or(0);

    Ok(QueueStatus { running, pending })
}

pub async fn free_memory(client: &Client, endpoint: &str, unload_models: bool) -> Result<()> {
    let url = format!("{}/free", endpoint);

    let body = if unload_models {
        serde_json::json!({"unload_models": true})
    } else {
        serde_json::json!({"free_memory": true})
    };

    client
        .post(&url)
        .timeout(Duration::from_secs(30))
        .json(&body)
        .send()
        .await
        .context("Failed to send free memory request to ComfyUI")?;

    Ok(())
}

pub async fn interrupt(client: &Client, endpoint: &str) -> Result<()> {
    let url = format!("{}/interrupt", endpoint);
    client
        .post(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .context("Failed to send interrupt to ComfyUI")?;
    Ok(())
}

fn gen_status_failed(prompt_id: &str, error: &str) -> GenerationStatus {
    GenerationStatus {
        prompt_id: prompt_id.to_string(),
        status: GenerationStatusKind::Failed,
        progress: None,
        current_step: None,
        total_steps: None,
        image_filenames: None,
        error: Some(error.to_string()),
    }
}

async fn fetch_completed_status(
    client: &Client,
    endpoint: &str,
    prompt_id: &str,
) -> Result<GenerationStatus> {
    if let Some(history) = get_history(client, endpoint, prompt_id).await? {
        let filenames: Vec<String> = history
            .image_filenames
            .iter()
            .map(|r| r.filename.clone())
            .collect();
        Ok(GenerationStatus {
            prompt_id: prompt_id.to_string(),
            status: if history.completed {
                GenerationStatusKind::Completed
            } else {
                GenerationStatusKind::Failed
            },
            progress: Some(1.0),
            current_step: None,
            total_steps: None,
            image_filenames: if filenames.is_empty() {
                None
            } else {
                Some(filenames)
            },
            error: if !history.completed {
                Some("Generation failed".to_string())
            } else {
                None
            },
        })
    } else {
        Ok(gen_status_failed(
            prompt_id,
            "No history found after generation",
        ))
    }
}

/// Poll history until the prompt completes or fails (fallback when WS unavailable)
pub async fn wait_for_completion(
    client: &Client,
    endpoint: &str,
    prompt_id: &str,
    poll_interval: Duration,
    timeout: Duration,
) -> Result<GenerationStatus> {
    let start = std::time::Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Ok(gen_status_failed(prompt_id, "Generation timed out"));
        }
        if let Some(history) = get_history(client, endpoint, prompt_id).await? {
            if history.completed {
                return fetch_completed_status(client, endpoint, prompt_id).await;
            } else if history.status == "error" {
                return Ok(gen_status_failed(prompt_id, "ComfyUI generation failed"));
            }
        }
        tokio::time::sleep(poll_interval).await;
    }
}

/// Wait for completion using ComfyUI's WebSocket for real-time step progress.
/// Calls `on_progress` for each sampling step. Falls back to polling on WS failure.
pub async fn wait_for_completion_ws<F>(
    client: &Client,
    endpoint: &str,
    prompt_id: &str,
    client_id: &str,
    timeout: Duration,
    mut on_progress: F,
) -> Result<GenerationStatus>
where
    F: FnMut(ProgressUpdate),
{
    let ws_url = format!(
        "{}/ws?clientId={}",
        endpoint
            .replace("http://", "ws://")
            .replace("https://", "wss://"),
        client_id
    );
    let (mut ws, _) = match tokio_tungstenite::connect_async(&ws_url).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[comfyui] WS failed: {}, falling back to polling", e);
            return wait_for_completion(
                client,
                endpoint,
                prompt_id,
                Duration::from_secs(2),
                timeout,
            )
            .await;
        }
    };

    let start = std::time::Instant::now();
    while let Ok(Some(msg)) = tokio::time::timeout(Duration::from_secs(30), ws.next()).await {
        if start.elapsed() > timeout {
            return Ok(gen_status_failed(prompt_id, "Generation timed out"));
        }
        let text = match msg {
            Ok(m) if m.is_text() => m.into_text().unwrap_or_default(),
            Ok(_) => continue,
            Err(_) => break,
        };
        let json: Value = match serde_json::from_str(&text) {
            Ok(j) => j,
            Err(_) => continue,
        };
        let msg_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let data = json.get("data");
        let pid = data
            .and_then(|d| d.get("prompt_id"))
            .and_then(|v| v.as_str());
        if pid.is_some() && pid != Some(prompt_id) {
            continue;
        }
        match msg_type {
            "progress" => {
                if let Some(d) = data {
                    let val = d.get("value").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let max = d.get("max").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                    on_progress(ProgressUpdate {
                        current_step: val,
                        total_steps: max,
                    });
                }
            }
            "executing"
                if data
                    .and_then(|d| d.get("node"))
                    .map(|v| v.is_null())
                    .unwrap_or(false) =>
            {
                return fetch_completed_status(client, endpoint, prompt_id).await;
            }
            "execution_error" => {
                let err = data
                    .and_then(|d| d.get("exception_message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Ok(gen_status_failed(
                    prompt_id,
                    &format!("ComfyUI error: {}", err),
                ));
            }
            _ => {}
        }
    }
    // WS closed unexpectedly — fall back to polling
    wait_for_completion(client, endpoint, prompt_id, Duration::from_secs(2), timeout).await
}

#[derive(Debug, Clone)]
pub struct ImageRef {
    pub filename: String,
    pub subfolder: String,
    pub img_type: String,
}

#[derive(Debug, Clone)]
pub struct PromptHistory {
    pub status: String,
    pub completed: bool,
    pub image_filenames: Vec<ImageRef>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStatus {
    pub running: u32,
    pub pending: u32,
}

#[cfg(test)]
#[path = "client_test.rs"]
mod tests;
