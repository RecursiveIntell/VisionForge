use crate::types::config::AppConfig;
use anyhow::{Context, Result};
use std::path::PathBuf;

pub fn data_dir() -> PathBuf {
    let home = dirs_home();
    home.join(".visionforge")
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

/// Returns the image base directory. Uses the custom directory from config
/// if set and non-empty, otherwise falls back to ~/.visionforge/images.
/// Expands `~` to the user's home directory (shell-style tilde expansion).
pub fn image_dir(config: &AppConfig) -> PathBuf {
    let custom = &config.storage.image_directory;
    if custom.is_empty() {
        data_dir().join("images")
    } else {
        expand_tilde(custom)
    }
}

/// Expand a leading `~` or `~/` to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        dirs_home()
    } else if let Some(rest) = path.strip_prefix("~/") {
        dirs_home().join(rest)
    } else {
        PathBuf::from(path)
    }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn load_or_create_default() -> Result<AppConfig> {
    let path = config_path();

    if path.exists() {
        load_config(&path)
    } else {
        let config = AppConfig::default();
        save_config_to_disk(&config)?;
        Ok(config)
    }
}

pub fn load_config(path: &std::path::Path) -> Result<AppConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config at {}", path.display()))?;
    let config: TomlConfig = toml::from_str(&content).context("Failed to parse config.toml")?;
    Ok(config.into_app_config())
}

pub fn save_config_to_disk(config: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
    }

    let toml_config = TomlConfig::from_app_config(config);
    let content =
        toml::to_string_pretty(&toml_config).context("Failed to serialize config to TOML")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;
    Ok(())
}

// TOML-compatible config structure (snake_case keys for TOML convention)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TomlConfig {
    #[serde(default)]
    comfyui: TomlComfyUi,
    #[serde(default)]
    ollama: TomlOllama,
    #[serde(default)]
    models: TomlModels,
    #[serde(default)]
    pipeline: TomlPipeline,
    #[serde(default)]
    hardware: TomlHardware,
    #[serde(default)]
    presets: std::collections::HashMap<String, TomlPreset>,
    #[serde(default)]
    storage: TomlStorage,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct TomlStorage {
    #[serde(default)]
    image_directory: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TomlComfyUi {
    #[serde(default = "default_comfyui_endpoint")]
    endpoint: String,
}

impl Default for TomlComfyUi {
    fn default() -> Self {
        Self {
            endpoint: default_comfyui_endpoint(),
        }
    }
}

fn default_comfyui_endpoint() -> String {
    "http://192.168.50.69:8188".to_string()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TomlOllama {
    #[serde(default = "default_ollama_endpoint")]
    endpoint: String,
}

impl Default for TomlOllama {
    fn default() -> Self {
        Self {
            endpoint: default_ollama_endpoint(),
        }
    }
}

fn default_ollama_endpoint() -> String {
    "http://localhost:11434".to_string()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TomlModels {
    #[serde(default = "default_ideator")]
    ideator: String,
    #[serde(default = "default_composer")]
    composer: String,
    #[serde(default = "default_judge")]
    judge: String,
    #[serde(default = "default_prompt_engineer")]
    prompt_engineer: String,
    #[serde(default = "default_reviewer")]
    reviewer: String,
    #[serde(default = "default_tagger")]
    tagger: String,
    #[serde(default = "default_captioner")]
    captioner: String,
}

impl Default for TomlModels {
    fn default() -> Self {
        Self {
            ideator: default_ideator(),
            composer: default_composer(),
            judge: default_judge(),
            prompt_engineer: default_prompt_engineer(),
            reviewer: default_reviewer(),
            tagger: default_tagger(),
            captioner: default_captioner(),
        }
    }
}

fn default_ideator() -> String {
    "mistral:7b".to_string()
}
fn default_composer() -> String {
    "llama3.1:8b".to_string()
}
fn default_judge() -> String {
    "qwen2.5:7b".to_string()
}
fn default_prompt_engineer() -> String {
    "mistral:7b".to_string()
}
fn default_reviewer() -> String {
    "qwen2.5:7b".to_string()
}
fn default_tagger() -> String {
    "llava:7b".to_string()
}
fn default_captioner() -> String {
    "llava:7b".to_string()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TomlPipeline {
    #[serde(default = "default_true")]
    enable_ideator: bool,
    #[serde(default = "default_true")]
    enable_composer: bool,
    #[serde(default = "default_true")]
    enable_judge: bool,
    #[serde(default = "default_true")]
    enable_prompt_engineer: bool,
    #[serde(default)]
    enable_reviewer: bool,
    #[serde(default)]
    auto_approve: bool,
}

impl Default for TomlPipeline {
    fn default() -> Self {
        Self {
            enable_ideator: true,
            enable_composer: true,
            enable_judge: true,
            enable_prompt_engineer: true,
            enable_reviewer: false,
            auto_approve: false,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TomlHardware {
    #[serde(default = "default_cooldown")]
    cooldown_seconds: u32,
    #[serde(default = "default_max_consecutive")]
    max_consecutive_generations: u32,
    #[serde(default)]
    enable_ha_power_monitoring: bool,
    #[serde(default = "default_ha_entity")]
    ha_entity_id: String,
    #[serde(default = "default_ha_watts")]
    ha_max_watts: u32,
}

impl Default for TomlHardware {
    fn default() -> Self {
        Self {
            cooldown_seconds: default_cooldown(),
            max_consecutive_generations: default_max_consecutive(),
            enable_ha_power_monitoring: false,
            ha_entity_id: default_ha_entity(),
            ha_max_watts: default_ha_watts(),
        }
    }
}

fn default_cooldown() -> u32 {
    30
}
fn default_max_consecutive() -> u32 {
    5
}
fn default_ha_entity() -> String {
    "sensor.gpu_power_draw".to_string()
}
fn default_ha_watts() -> u32 {
    180
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TomlPreset {
    steps: u32,
    cfg: f64,
    width: u32,
    height: u32,
    sampler: String,
    scheduler: String,
}

impl TomlConfig {
    fn into_app_config(self) -> AppConfig {
        use crate::types::config::*;

        let mut presets = std::collections::HashMap::new();
        for (name, p) in self.presets {
            presets.insert(
                name,
                QualityPreset {
                    steps: p.steps,
                    cfg: p.cfg,
                    width: p.width,
                    height: p.height,
                    sampler: p.sampler,
                    scheduler: p.scheduler,
                },
            );
        }

        // Ensure default presets exist
        let defaults = AppConfig::default();
        for (name, preset) in defaults.presets {
            presets.entry(name).or_insert(preset);
        }

        AppConfig {
            comfyui: ComfyUiConfig {
                endpoint: self.comfyui.endpoint,
            },
            ollama: OllamaConfig {
                endpoint: self.ollama.endpoint,
            },
            models: ModelAssignments {
                ideator: self.models.ideator,
                composer: self.models.composer,
                judge: self.models.judge,
                prompt_engineer: self.models.prompt_engineer,
                reviewer: self.models.reviewer,
                tagger: self.models.tagger,
                captioner: self.models.captioner,
            },
            pipeline: PipelineSettings {
                enable_ideator: self.pipeline.enable_ideator,
                enable_composer: self.pipeline.enable_composer,
                enable_judge: self.pipeline.enable_judge,
                enable_prompt_engineer: self.pipeline.enable_prompt_engineer,
                enable_reviewer: self.pipeline.enable_reviewer,
                auto_approve: self.pipeline.auto_approve,
            },
            hardware: HardwareSettings {
                cooldown_seconds: self.hardware.cooldown_seconds,
                max_consecutive_generations: self.hardware.max_consecutive_generations,
                enable_ha_power_monitoring: self.hardware.enable_ha_power_monitoring,
                ha_entity_id: self.hardware.ha_entity_id,
                ha_max_watts: self.hardware.ha_max_watts,
            },
            storage: crate::types::config::StorageSettings {
                image_directory: self.storage.image_directory,
            },
            presets,
        }
    }

    fn from_app_config(config: &AppConfig) -> Self {
        let mut presets = std::collections::HashMap::new();
        for (name, p) in &config.presets {
            presets.insert(
                name.clone(),
                TomlPreset {
                    steps: p.steps,
                    cfg: p.cfg,
                    width: p.width,
                    height: p.height,
                    sampler: p.sampler.clone(),
                    scheduler: p.scheduler.clone(),
                },
            );
        }

        TomlConfig {
            comfyui: TomlComfyUi {
                endpoint: config.comfyui.endpoint.clone(),
            },
            ollama: TomlOllama {
                endpoint: config.ollama.endpoint.clone(),
            },
            models: TomlModels {
                ideator: config.models.ideator.clone(),
                composer: config.models.composer.clone(),
                judge: config.models.judge.clone(),
                prompt_engineer: config.models.prompt_engineer.clone(),
                reviewer: config.models.reviewer.clone(),
                tagger: config.models.tagger.clone(),
                captioner: config.models.captioner.clone(),
            },
            pipeline: TomlPipeline {
                enable_ideator: config.pipeline.enable_ideator,
                enable_composer: config.pipeline.enable_composer,
                enable_judge: config.pipeline.enable_judge,
                enable_prompt_engineer: config.pipeline.enable_prompt_engineer,
                enable_reviewer: config.pipeline.enable_reviewer,
                auto_approve: config.pipeline.auto_approve,
            },
            hardware: TomlHardware {
                cooldown_seconds: config.hardware.cooldown_seconds,
                max_consecutive_generations: config.hardware.max_consecutive_generations,
                enable_ha_power_monitoring: config.hardware.enable_ha_power_monitoring,
                ha_entity_id: config.hardware.ha_entity_id.clone(),
                ha_max_watts: config.hardware.ha_max_watts,
            },
            storage: TomlStorage {
                image_directory: config.storage.image_directory.clone(),
            },
            presets,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_serializes() {
        let config = AppConfig::default();
        let toml_config = TomlConfig::from_app_config(&config);
        let serialized = toml::to_string_pretty(&toml_config).unwrap();
        assert!(serialized.contains("[comfyui]"));
        assert!(serialized.contains("[ollama]"));
        assert!(serialized.contains("[models]"));
        assert!(serialized.contains("[pipeline]"));
        assert!(serialized.contains("[hardware]"));
    }

    #[test]
    fn test_config_roundtrip() {
        let config = AppConfig::default();
        let toml_config = TomlConfig::from_app_config(&config);
        let serialized = toml::to_string_pretty(&toml_config).unwrap();
        let deserialized: TomlConfig = toml::from_str(&serialized).unwrap();
        let roundtripped = deserialized.into_app_config();

        assert_eq!(roundtripped.comfyui.endpoint, config.comfyui.endpoint);
        assert_eq!(roundtripped.ollama.endpoint, config.ollama.endpoint);
        assert_eq!(roundtripped.models.ideator, config.models.ideator);
        assert_eq!(
            roundtripped.pipeline.enable_ideator,
            config.pipeline.enable_ideator
        );
        assert_eq!(
            roundtripped.hardware.cooldown_seconds,
            config.hardware.cooldown_seconds
        );
        assert_eq!(roundtripped.presets.len(), config.presets.len());
    }

    #[test]
    fn test_expand_tilde() {
        let home = super::dirs_home();
        assert_eq!(super::expand_tilde("~"), home);
        assert_eq!(super::expand_tilde("~/Pictures"), home.join("Pictures"));
        assert_eq!(
            super::expand_tilde("~/Pictures/SD"),
            home.join("Pictures/SD")
        );
        // Non-tilde paths pass through unchanged
        assert_eq!(
            super::expand_tilde("/tmp/images"),
            PathBuf::from("/tmp/images")
        );
    }

    #[test]
    fn test_image_dir_expands_tilde() {
        let mut config = AppConfig::default();
        config.storage.image_directory = "~/Pictures/SD".to_string();
        let dir = super::image_dir(&config);
        assert!(dir.to_str().unwrap().contains("Pictures/SD"));
        // Must NOT contain a literal ~
        assert!(!dir.to_str().unwrap().contains('~'));
    }

    #[test]
    fn test_partial_toml_uses_defaults() {
        let partial = r#"
[comfyui]
endpoint = "http://myhost:8188"
"#;
        let toml_config: TomlConfig = toml::from_str(partial).unwrap();
        let config = toml_config.into_app_config();

        assert_eq!(config.comfyui.endpoint, "http://myhost:8188");
        assert_eq!(config.ollama.endpoint, "http://localhost:11434");
        assert_eq!(config.models.ideator, "mistral:7b");
        assert!(config.pipeline.enable_ideator);
    }
}
