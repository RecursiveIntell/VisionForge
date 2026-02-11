use rand::Rng;
use serde_json::{json, Value};

use crate::types::generation::GenerationRequest;

/// Build a txt2img workflow for ComfyUI from generation settings.
/// Returns the workflow JSON (the value for the "prompt" field in the /prompt request).
pub fn build_txt2img(request: &GenerationRequest) -> Value {
    // ComfyUI requires seed >= 0; -1 means "random"
    let seed = if request.seed < 0 {
        rand::rng().random_range(0..i64::MAX)
    } else {
        request.seed
    };

    json!({
        "1": {
            "class_type": "CheckpointLoaderSimple",
            "inputs": {
                "ckpt_name": request.checkpoint
            }
        },
        "2": {
            "class_type": "EmptyLatentImage",
            "inputs": {
                "width": request.width,
                "height": request.height,
                "batch_size": request.batch_size
            }
        },
        "3": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": request.positive_prompt,
                "clip": ["1", 1]
            }
        },
        "4": {
            "class_type": "CLIPTextEncode",
            "inputs": {
                "text": request.negative_prompt,
                "clip": ["1", 1]
            }
        },
        "5": {
            "class_type": "KSampler",
            "inputs": {
                "seed": seed,
                "steps": request.steps,
                "cfg": request.cfg_scale,
                "sampler_name": request.sampler,
                "scheduler": request.scheduler,
                "denoise": 1.0,
                "model": ["1", 0],
                "positive": ["3", 0],
                "negative": ["4", 0],
                "latent_image": ["2", 0]
            }
        },
        "6": {
            "class_type": "VAEDecode",
            "inputs": {
                "samples": ["5", 0],
                "vae": ["1", 2]
            }
        },
        "7": {
            "class_type": "SaveImage",
            "inputs": {
                "filename_prefix": "VisionForge",
                "images": ["6", 0]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request() -> GenerationRequest {
        GenerationRequest {
            positive_prompt: "masterpiece, best quality, a cat".to_string(),
            negative_prompt: "lowres, blurry".to_string(),
            checkpoint: "dreamshaper_8.safetensors".to_string(),
            width: 512,
            height: 768,
            steps: 25,
            cfg_scale: 7.5,
            sampler: "dpmpp_2m".to_string(),
            scheduler: "karras".to_string(),
            seed: 12345,
            batch_size: 1,
        }
    }

    #[test]
    fn test_build_txt2img_has_all_nodes() {
        let workflow = build_txt2img(&make_request());
        assert!(workflow.get("1").is_some()); // CheckpointLoader
        assert!(workflow.get("2").is_some()); // EmptyLatentImage
        assert!(workflow.get("3").is_some()); // CLIPTextEncode positive
        assert!(workflow.get("4").is_some()); // CLIPTextEncode negative
        assert!(workflow.get("5").is_some()); // KSampler
        assert!(workflow.get("6").is_some()); // VAEDecode
        assert!(workflow.get("7").is_some()); // SaveImage
    }

    #[test]
    fn test_checkpoint_loader() {
        let workflow = build_txt2img(&make_request());
        let node = &workflow["1"];
        assert_eq!(node["class_type"], "CheckpointLoaderSimple");
        assert_eq!(node["inputs"]["ckpt_name"], "dreamshaper_8.safetensors");
    }

    #[test]
    fn test_ksampler_settings() {
        let workflow = build_txt2img(&make_request());
        let node = &workflow["5"];
        assert_eq!(node["class_type"], "KSampler");
        assert_eq!(node["inputs"]["seed"], 12345);
        assert_eq!(node["inputs"]["steps"], 25);
        assert_eq!(node["inputs"]["cfg"], 7.5);
        assert_eq!(node["inputs"]["sampler_name"], "dpmpp_2m");
        assert_eq!(node["inputs"]["scheduler"], "karras");
        assert_eq!(node["inputs"]["denoise"], 1.0);
    }

    #[test]
    fn test_clip_text_encode() {
        let workflow = build_txt2img(&make_request());
        let positive = &workflow["3"];
        assert_eq!(positive["inputs"]["text"], "masterpiece, best quality, a cat");
        assert_eq!(positive["inputs"]["clip"], json!(["1", 1]));

        let negative = &workflow["4"];
        assert_eq!(negative["inputs"]["text"], "lowres, blurry");
    }

    #[test]
    fn test_empty_latent_image() {
        let workflow = build_txt2img(&make_request());
        let node = &workflow["2"];
        assert_eq!(node["inputs"]["width"], 512);
        assert_eq!(node["inputs"]["height"], 768);
        assert_eq!(node["inputs"]["batch_size"], 1);
    }

    #[test]
    fn test_node_connections() {
        let workflow = build_txt2img(&make_request());

        // KSampler connects to checkpoint model, positive, negative, latent
        assert_eq!(workflow["5"]["inputs"]["model"], json!(["1", 0]));
        assert_eq!(workflow["5"]["inputs"]["positive"], json!(["3", 0]));
        assert_eq!(workflow["5"]["inputs"]["negative"], json!(["4", 0]));
        assert_eq!(workflow["5"]["inputs"]["latent_image"], json!(["2", 0]));

        // VAEDecode connects to KSampler output and checkpoint VAE
        assert_eq!(workflow["6"]["inputs"]["samples"], json!(["5", 0]));
        assert_eq!(workflow["6"]["inputs"]["vae"], json!(["1", 2]));

        // SaveImage connects to VAEDecode output
        assert_eq!(workflow["7"]["inputs"]["images"], json!(["6", 0]));
    }

    #[test]
    fn test_save_image_prefix() {
        let workflow = build_txt2img(&make_request());
        assert_eq!(workflow["7"]["inputs"]["filename_prefix"], "VisionForge");
    }

    #[test]
    fn test_workflow_is_valid_json() {
        let workflow = build_txt2img(&make_request());
        let json_str = serde_json::to_string(&workflow).unwrap();
        assert!(json_str.len() > 100);
        // Can re-parse
        let _: Value = serde_json::from_str(&json_str).unwrap();
    }
}
