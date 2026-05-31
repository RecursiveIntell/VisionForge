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
        "options": {
            "num_predict": 512,
            "repeat_penalty": 1.2,
            "repeat_last_n": 128,
        },
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

    let json: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse Ollama response")?;
    let content = json
        .get("response")
        .and_then(|v| v.as_str())
        .unwrap_or("[]");

    parse_tags(content)
}

/// Parse the LLM response into a list of tags.
/// Handles `<think>` blocks, markdown code fences, JSON objects with
/// a "tags" key, bare JSON arrays, and comma-separated fallback.
fn parse_tags(response: &str) -> Result<Vec<String>> {
    let trimmed = response.trim();

    // Try JSON array directly
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(trimmed) {
        return Ok(clean_tags(arr));
    }

    // Strip <think>...</think> blocks from reasoning models
    let cleaned = strip_think_tags(trimmed);
    let cleaned = cleaned.trim();

    // Try cleaned text as JSON array
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(cleaned) {
        return Ok(clean_tags(arr));
    }

    // Try as JSON object with a "tags" key (e.g. {"tags": [...]})
    if let Some(tags) = try_extract_tags_from_object(cleaned) {
        return Ok(clean_tags(tags));
    }

    // Try extracting from markdown code blocks
    if let Some(tags) = extract_tags_from_code_block(cleaned) {
        return Ok(clean_tags(tags));
    }

    // Try bracket matching (prefer last occurrence to skip stray brackets)
    if let Some(tags) = find_json_array(cleaned) {
        return Ok(clean_tags(tags));
    }

    // Fallback: try comma-separated
    let tags: Vec<String> = cleaned
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim().to_lowercase())
        .filter(|s| !s.is_empty() && s.len() < 50)
        .collect();

    if tags.is_empty() {
        anyhow::bail!("Could not parse tags from LLM response: {}", cleaned);
    }

    Ok(tags)
}

/// Strip `<think>...</think>` blocks emitted by reasoning models
fn strip_think_tags(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("<think>") {
        if let Some(end) = result[start..].find("</think>") {
            result = format!("{}{}", &result[..start], &result[start + end + 8..]);
        } else {
            // No closing tag — strip from <think> to end
            result = result[..start].to_string();
            break;
        }
    }
    result
}

/// Try parsing as a JSON object and extracting an array from a "tags" key
fn try_extract_tags_from_object(text: &str) -> Option<Vec<String>> {
    let val: serde_json::Value = serde_json::from_str(text).ok()?;
    let arr = val.get("tags").and_then(|v| v.as_array())?;
    let tags = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    Some(tags)
}

/// Extract a JSON array from markdown code blocks (```json or ```)
fn extract_tags_from_code_block(text: &str) -> Option<Vec<String>> {
    for marker in ["```json", "```"] {
        let mut search_from = 0;
        while let Some(start) = text[search_from..].find(marker) {
            let abs_start = search_from + start + marker.len();
            let content_start = text[abs_start..].find('\n').map(|p| abs_start + p + 1)?;
            if let Some(end) = text[content_start..].find("```") {
                let candidate = text[content_start..content_start + end].trim();
                if let Ok(arr) = serde_json::from_str::<Vec<String>>(candidate) {
                    return Some(arr);
                }
                // Also try object with "tags" key inside code block
                if let Some(tags) = try_extract_tags_from_object(candidate) {
                    return Some(tags);
                }
            }
            search_from = abs_start;
        }
    }
    None
}

/// Find a JSON array by bracket matching, preferring later occurrences
fn find_json_array(text: &str) -> Option<Vec<String>> {
    let starts: Vec<usize> = text.match_indices('[').map(|(i, _)| i).collect();
    let ends: Vec<usize> = text.match_indices(']').map(|(i, _)| i).collect();

    for &start in starts.iter().rev() {
        for &end in ends.iter().rev() {
            if end <= start {
                continue;
            }
            let candidate = &text[start..=end];
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(candidate) {
                return Some(arr);
            }
        }
    }
    None
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
    fn test_parse_tags_with_think_blocks() {
        let input = r#"<think>
Let me analyze this image. I see a portrait with dark lighting...
[considering visual elements]
</think>

["portrait", "dark lighting", "woman"]"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["portrait", "dark lighting", "woman"]);
    }

    #[test]
    fn test_parse_tags_with_incomplete_think() {
        let input = r#"<think>
Still thinking about the image...
["portrait", "fantasy", "dramatic"]"#;
        // <think> never closed — should strip from <think> to end, then fail
        // gracefully. The JSON array is inside the think block so it gets stripped.
        // This should fall through to the comma fallback or fail.
        let result = parse_tags(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tags_markdown_code_block() {
        let input = r#"Here are the tags for this image:

```json
["portrait", "fantasy", "oil painting"]
```"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["portrait", "fantasy", "oil painting"]);
    }

    #[test]
    fn test_parse_tags_object_with_tags_key() {
        let input = r#"{"tags": ["portrait", "dark", "moody"]}"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["portrait", "dark", "moody"]);
    }

    #[test]
    fn test_parse_tags_think_then_code_block() {
        let input = r#"<think>
Analyzing the image elements...
</think>

```json
["landscape", "sunset", "mountains"]
```"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["landscape", "sunset", "mountains"]);
    }

    #[test]
    fn test_parse_tags_think_then_object() {
        let input = r#"<think>
Looking at this...
</think>
{"tags": ["cat", "cute", "indoor"]}"#;
        let tags = parse_tags(input).unwrap();
        assert_eq!(tags, vec!["cat", "cute", "indoor"]);
    }

    #[test]
    fn test_clean_tags_filters_empty() {
        let tags = vec!["good".to_string(), "".to_string(), "  ".to_string()];
        let cleaned = clean_tags(tags);
        assert_eq!(cleaned, vec!["good"]);
    }
}
