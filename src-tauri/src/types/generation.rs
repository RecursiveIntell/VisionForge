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
