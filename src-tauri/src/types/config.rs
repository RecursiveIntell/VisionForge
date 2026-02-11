use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub comfyui: ComfyUiConfig,
    pub ollama: OllamaConfig,
    pub models: ModelAssignments,
    pub pipeline: PipelineSettings,
    pub hardware: HardwareSettings,
    pub presets: HashMap<String, QualityPreset>,
    #[serde(default)]
    pub storage: StorageSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComfyUiConfig {
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaConfig {
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelAssignments {
    pub ideator: String,
    pub composer: String,
    pub judge: String,
    pub prompt_engineer: String,
    pub reviewer: String,
    pub tagger: String,
    pub captioner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineSettings {
    pub enable_ideator: bool,
    pub enable_composer: bool,
    pub enable_judge: bool,
    pub enable_prompt_engineer: bool,
    pub enable_reviewer: bool,
    pub auto_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareSettings {
    pub cooldown_seconds: u32,
    pub max_consecutive_generations: u32,
    pub enable_ha_power_monitoring: bool,
    pub ha_entity_id: String,
    pub ha_max_watts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StorageSettings {
    /// Custom image directory. Empty string means use default (~/.visionforge/images).
    #[serde(default)]
    pub image_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityPreset {
    pub steps: u32,
    pub cfg: f64,
    pub width: u32,
    pub height: u32,
    pub sampler: String,
    pub scheduler: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut presets = HashMap::new();
        presets.insert(
            "quick_draft".to_string(),
            QualityPreset {
                steps: 12,
                cfg: 7.0,
                width: 512,
                height: 512,
                sampler: "euler_ancestral".to_string(),
                scheduler: "normal".to_string(),
            },
        );
        presets.insert(
            "quality".to_string(),
            QualityPreset {
                steps: 25,
                cfg: 7.5,
                width: 512,
                height: 768,
                sampler: "dpmpp_2m".to_string(),
                scheduler: "karras".to_string(),
            },
        );
        presets.insert(
            "max_effort".to_string(),
            QualityPreset {
                steps: 40,
                cfg: 8.0,
                width: 768,
                height: 768,
                sampler: "dpmpp_sde".to_string(),
                scheduler: "karras".to_string(),
            },
        );

        Self {
            comfyui: ComfyUiConfig {
                endpoint: "http://192.168.50.69:8188".to_string(),
            },
            ollama: OllamaConfig {
                endpoint: "http://localhost:11434".to_string(),
            },
            models: ModelAssignments {
                ideator: "mistral:7b".to_string(),
                composer: "llama3.1:8b".to_string(),
                judge: "qwen2.5:7b".to_string(),
                prompt_engineer: "mistral:7b".to_string(),
                reviewer: "qwen2.5:7b".to_string(),
                tagger: "llava:7b".to_string(),
                captioner: "llava:7b".to_string(),
            },
            pipeline: PipelineSettings {
                enable_ideator: true,
                enable_composer: true,
                enable_judge: true,
                enable_prompt_engineer: true,
                enable_reviewer: false,
                auto_approve: false,
            },
            hardware: HardwareSettings {
                cooldown_seconds: 30,
                max_consecutive_generations: 5,
                enable_ha_power_monitoring: false,
                ha_entity_id: "sensor.gpu_power_draw".to_string(),
                ha_max_watts: 180,
            },
            presets,
            storage: StorageSettings::default(),
        }
    }
}
