use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationRequest {
    pub positive_prompt: String,
    pub negative_prompt: String,
    pub checkpoint: String,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub cfg_scale: f64,
    pub sampler: String,
    pub scheduler: String,
    pub seed: i64,
    pub batch_size: u32,
}

/// Typed representation of the settings_json stored in QueueJob.
/// Supports both camelCase and snake_case field names via serde aliases.
#[derive(Debug, Clone, Deserialize)]
pub struct GenerationSettings {
    pub checkpoint: String,

    #[serde(default = "default_width")]
    pub width: u32,

    #[serde(default = "default_height")]
    pub height: u32,

    #[serde(default = "default_steps")]
    pub steps: u32,

    #[serde(alias = "cfgScale", alias = "cfg_scale", default = "default_cfg")]
    pub cfg_scale: f64,

    #[serde(default = "default_sampler")]
    pub sampler: String,

    #[serde(default = "default_scheduler")]
    pub scheduler: String,

    #[serde(default = "default_seed")]
    pub seed: i64,

    #[serde(
        alias = "batchSize",
        alias = "batch_size",
        default = "default_batch_size"
    )]
    pub batch_size: u32,
}

fn default_width() -> u32 {
    512
}
fn default_height() -> u32 {
    768
}
fn default_steps() -> u32 {
    25
}
fn default_cfg() -> f64 {
    7.5
}
fn default_sampler() -> String {
    "dpmpp_2m".to_string()
}
fn default_scheduler() -> String {
    "karras".to_string()
}
fn default_seed() -> i64 {
    -1
}
fn default_batch_size() -> u32 {
    1
}

impl GenerationSettings {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.checkpoint.is_empty() {
            anyhow::bail!("Checkpoint is required. Please select a checkpoint before queueing.");
        }
        if self.width < 64 || self.width > 4096 {
            anyhow::bail!("Width must be between 64 and 4096, got {}", self.width);
        }
        if self.height < 64 || self.height > 4096 {
            anyhow::bail!("Height must be between 64 and 4096, got {}", self.height);
        }
        if self.steps < 1 || self.steps > 150 {
            anyhow::bail!("Steps must be between 1 and 150, got {}", self.steps);
        }
        if self.cfg_scale < 0.0 || self.cfg_scale > 30.0 {
            anyhow::bail!("CFG scale must be between 0 and 30, got {}", self.cfg_scale);
        }
        if self.batch_size < 1 || self.batch_size > 16 {
            anyhow::bail!(
                "Batch size must be between 1 and 16, got {}",
                self.batch_size
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GenerationStatusKind {
    Queued,
    Generating,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationStatus {
    pub prompt_id: String,
    pub status: GenerationStatusKind,
    pub progress: Option<f64>,
    pub current_step: Option<u32>,
    pub total_steps: Option<u32>,
    pub image_filenames: Option<Vec<String>>,
    pub error: Option<String>,
}
