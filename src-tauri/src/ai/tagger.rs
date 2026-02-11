use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::path::Path;
use std::time::Duration;

const TAG_SYSTEM_PROMPT: &str = r#"You are an image tagging assistant. Analyze the provided image and return a JSON array of relevant tags. Each tag should be a single word or short phrase (2-3 words max) that describes a key visual element, style, subject, or mood in the image.

Return ONLY a JSON array of strings. Example: ["portrait", "fantasy", "dark lighting", "woman", "medieval", "oil painting"]

Return between 5 and 15 tags. Focus on:
- Subject matter (person, animal, landscape, object)
- Art style (photorealistic, anime, oil painting, digital art)
- Mood/atmosphere (dark, bright, serene, dramatic)
- Colors (warm tones, blue, monochrome)
- Composition (close-up, wide shot, symmetrical)
- Notable elements (fire, water, armor, flowers)"#;

/// Auto-tag an image using Ollama's vision model.
/// Returns a list of tag strings.
pub async fn tag_image(
    client: &Client,
    endpoint: &str,
    model: &str,
    image_path: &Path,
) -> Result<Vec<String>> {
    let image_b64 = read_image_base64(image_path)?;

    let body = json!({
        "model": model,
        "prompt": TAG_SYSTEM_PROMPT,
        "images": [image_b64],
        "stream": false,
        "format": "json",
    });

    let url = format!("{}/api/generate", endpoint);
    let resp = client
        .post(&url)
        .timeout(Duration::from_secs(120))
        .json(&body)
        .send()
        .await
        .with_context(|| format!("Cannot connect to Ollama at {} for tagging", endpoint))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        anyhow::bail!("Ollama returned {} for tagging: {}", status, text);
    }

    let json: serde_json::Value = resp.json().await.context("Failed to parse Ollama response")?;
    let content = json
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("[]");

    parse_tags(content)
}

/// Parse the LLM response into a list of tags.
/// Handles both JSON arrays and comma-separated fallback.
fn parse_tags(response: &str) -> Result<Vec<String>> {
    let trimmed = response.trim();

    // Try JSON array first
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
        return Ok(clean_tags(arr));
    }

    // Try extracting JSON array from surrounding text
    if let Some(start) = trimmed.find('[') {
        if let Some(end) = trimmed.rfind(']') {
            let slice = &trimmed[start..=end];
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(slice) {
                return Ok(clean_tags(arr));
            }
        }
    }

    // Fallback: try comma-separated
    let tags: Vec<String> = trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_lowercase())
        .filter(|s| !s.is_empty() && s.len() < 50)
        .collect();

    if tags.is_empty() {
        anyhow::bail!("Could not parse tags from LLM response: {}", trimmed);
    }

    Ok(tags)
}

fn clean_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty() && t.len() < 50)
        .collect()
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
    use super::*;

    #[test]
    fn test_parse_tags_json_array() {
        let input = r#"["portrait", "fantasy", "dark lighting"]"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["portrait", "fantasy", "dark lighting"]);
    }

    #[test]
    fn test_parse_tags_with_surrounding_text() {
        let input = r#"Here are the tags: ["cat", "cute", "indoor"]"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["cat", "cute", "indoor"]);
    }

    #[test]
    fn test_parse_tags_comma_fallback() {
        let input = "portrait, fantasy, dark lighting";
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["portrait", "fantasy", "dark lighting"]);
    }

    #[test]
    fn test_parse_tags_cleans_whitespace() {
        let input = r#"["  Portrait  ", " FANTASY ", "Dark Lighting"]"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["portrait", "fantasy", "dark lighting"]);
    }

    #[test]
    fn test_parse_tags_empty_fails() {
        let input = "";
        assert!(parse_tags(input).is_err());
    }

    #[test]
    fn test_clean_tags_filters_empty() {
        let tags = vec!["good".to_string(), "".to_string(), "  ".to_string()];
        let cleaned = clean_tags(tags);
        assert_eq!(cleaned, vec!["good"]);
    }
}
