use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

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

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: Option<u64>,
    pub digest: Option<String>,
}

pub async fn check_health(client: &Client, endpoint: &str) -> Result<bool> {
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
            Some(OllamaModel {
                name,
                size,
                digest,
            })
        })
        .collect();

    Ok(models)
}

pub async fn chat(
    client: &Client,
    endpoint: &str,
    model: &str,
    messages: &[ChatMessage],
    format_json: bool,
) -> Result<ChatResponse> {
    let url = format!("{}/api/chat", endpoint);

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false,
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

pub async fn generate(
    client: &Client,
    endpoint: &str,
    model: &str,
    prompt: &str,
    format_json: bool,
) -> Result<ChatResponse> {
    let url = format!("{}/api/generate", endpoint);

    let mut body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
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
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: "system".to_string(),
            content: "You are a helper.".to_string(),
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["role"], "system");
        assert_eq!(json["content"], "You are a helper.");
    }

    #[test]
    fn test_parse_chat_response() {
        let json: Value = serde_json::from_str(
            r#"{
                "model": "mistral:7b",
                "message": {"role": "assistant", "content": "Hello world"},
                "done": true,
                "total_duration": 5000000000,
                "prompt_eval_count": 42,
                "eval_count": 10
            }"#,
        )
        .unwrap();

        let content = json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("");
        assert_eq!(content, "Hello world");
        assert_eq!(json.get("total_duration").and_then(|v| v.as_u64()), Some(5000000000));
    }

    #[test]
    fn test_parse_generate_response() {
        let json: Value = serde_json::from_str(
            r#"{
                "model": "mistral:7b",
                "response": "1. Concept one\n2. Concept two",
                "done": true,
                "total_duration": 3000000000,
                "prompt_eval_count": 50,
                "eval_count": 200
            }"#,
        )
        .unwrap();

        let content = json
            .get("response")
            .and_then(|c| c.as_str())
            .unwrap_or("");
        assert!(content.contains("Concept one"));
        assert!(content.contains("Concept two"));
    }

    #[test]
    fn test_parse_error_response() {
        let json: Value = serde_json::from_str(
            r#"{"error": "model not found"}"#,
        )
        .unwrap();

        let error = json.get("error").and_then(|v| v.as_str());
        assert_eq!(error, Some("model not found"));
    }

    #[test]
    fn test_parse_models_response() {
        let json: Value = serde_json::from_str(
            r#"{
                "models": [
                    {"name": "mistral:7b", "size": 4000000000, "digest": "abc123"},
                    {"name": "llama3.1:8b", "size": 5000000000, "digest": "def456"}
                ]
            }"#,
        )
        .unwrap();

        let models: Vec<OllamaModel> = json
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

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].name, "mistral:7b");
        assert_eq!(models[1].name, "llama3.1:8b");
    }
}
