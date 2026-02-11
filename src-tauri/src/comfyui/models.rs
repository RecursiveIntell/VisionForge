use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// Discover available checkpoints from ComfyUI via /object_info endpoint.
/// This queries the CheckpointLoaderSimple node to find which checkpoints are installed.
pub async fn list_checkpoints(client: &Client, endpoint: &str) -> Result<Vec<String>> {
    let url = format!("{}/object_info/CheckpointLoaderSimple", endpoint);

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .with_context(|| {
            format!(
                "Cannot connect to ComfyUI at {} — is the service running?",
                endpoint
            )
        })?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "ComfyUI returned {} when fetching checkpoint list",
            resp.status()
        );
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse ComfyUI object_info response")?;

    let checkpoints = json
        .pointer("/CheckpointLoaderSimple/input/required/ckpt_name/0")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    Ok(checkpoints)
}

/// Discover available samplers from ComfyUI
pub async fn list_samplers(client: &Client, endpoint: &str) -> Result<Vec<String>> {
    let url = format!("{}/object_info/KSampler", endpoint);

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch KSampler info from ComfyUI")?;

    if !resp.status().is_success() {
        return Ok(Vec::new());
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse KSampler object_info")?;

    let samplers = json
        .pointer("/KSampler/input/required/sampler_name/0")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(samplers)
}

/// Discover available schedulers from ComfyUI
pub async fn list_schedulers(client: &Client, endpoint: &str) -> Result<Vec<String>> {
    let url = format!("{}/object_info/KSampler", endpoint);

    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to fetch KSampler info from ComfyUI")?;

    if !resp.status().is_success() {
        return Ok(Vec::new());
    }

    let json: Value = resp
        .json()
        .await
        .context("Failed to parse KSampler object_info")?;

    let schedulers = json
        .pointer("/KSampler/input/required/scheduler/0")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(schedulers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checkpoint_object_info() {
        let json: Value = serde_json::from_str(r#"{
            "CheckpointLoaderSimple": {
                "input": {
                    "required": {
                        "ckpt_name": [
                            ["dreamshaper_8.safetensors", "deliberate_v3.safetensors", "sdxl_base.safetensors"]
                        ]
                    }
                }
            }
        }"#).unwrap();

        let checkpoints = json
            .pointer("/CheckpointLoaderSimple/input/required/ckpt_name/0")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        assert_eq!(checkpoints.len(), 3);
        assert_eq!(checkpoints[0], "dreamshaper_8.safetensors");
        assert_eq!(checkpoints[1], "deliberate_v3.safetensors");
        assert_eq!(checkpoints[2], "sdxl_base.safetensors");
    }

    #[test]
    fn test_parse_sampler_object_info() {
        let json: Value = serde_json::from_str(
            r#"{
            "KSampler": {
                "input": {
                    "required": {
                        "sampler_name": [["euler", "euler_ancestral", "dpmpp_2m", "dpmpp_sde"]],
                        "scheduler": [["normal", "karras", "exponential"]]
                    }
                }
            }
        }"#,
        )
        .unwrap();

        let samplers = json
            .pointer("/KSampler/input/required/sampler_name/0")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        assert_eq!(samplers.len(), 4);
        assert!(samplers.contains(&"dpmpp_2m".to_string()));

        let schedulers = json
            .pointer("/KSampler/input/required/scheduler/0")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        assert_eq!(schedulers.len(), 3);
        assert!(schedulers.contains(&"karras".to_string()));
    }

    #[test]
    fn test_empty_object_info() {
        let json: Value = serde_json::from_str(r#"{}"#).unwrap();

        let checkpoints = json
            .pointer("/CheckpointLoaderSimple/input/required/ckpt_name/0")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        assert!(checkpoints.is_empty());
    }
}
