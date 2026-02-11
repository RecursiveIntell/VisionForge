use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::path::Path;
use std::time::Duration;

const CAPTION_PROMPT: &str = r#"Describe this image in 1-2 sentences. Focus on the main subject, art style, composition, lighting, and mood. Be specific and concise. Do not start with "This image shows" or "The image depicts". Just describe what you see directly."#;

/// Generate a descriptive caption for an image using Ollama's vision model.
pub async fn caption_image(
    client: &Client,
    endpoint: &str,
    model: &str,
    image_path: &Path,
) -> Result<String> {
    let image_b64 = read_image_base64(image_path)?;

    let body = json!({
        "model": model,
        "prompt": CAPTION_PROMPT,
        "images": [image_b64],
        "stream": false,
    });

    let url = format!("{}/api/generate", endpoint);
    let resp = client
        .post(&url)
        .timeout(Duration::from_secs(120))
        .json(&body)
        .send()
        .await
        .with_context(|| format!("Cannot connect to Ollama at {} for captioning", endpoint))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned {} for captioning: {}", status, text);
    }

    let json: serde_json::Value = resp.json().await.context("Failed to parse Ollama response")?;
    let caption = json
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if caption.is_empty() {
        anyhow::bail!("Ollama returned empty caption");
    }

    Ok(caption)
}

fn read_image_base64(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("Failed to read image at {}", path.display()))?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &bytes,
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_caption_prompt_not_empty() {
        assert!(!super::CAPTION_PROMPT.is_empty());
        assert!(super::CAPTION_PROMPT.len() > 50);
    }
}
