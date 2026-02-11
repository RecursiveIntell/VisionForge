use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineResult {
    pub original_idea: String,
    pub pipeline_config: PipelineConfig,
    pub stages: PipelineStages,
    pub user_edits: Option<UserEdits>,
    pub auto_approved: bool,
    pub generation_settings: Option<GenerationSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineConfig {
    pub stages_enabled: [bool; 5],
    pub models_used: ModelsUsed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsUsed {
    pub ideator: Option<String>,
    pub composer: Option<String>,
    pub judge: Option<String>,
    pub prompt_engineer: Option<String>,
    pub reviewer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PipelineStages {
    pub ideator: Option<IdeatorOutput>,
    pub composer: Option<ComposerOutput>,
    pub judge: Option<JudgeOutput>,
    pub prompt_engineer: Option<PromptEngineerOutput>,
    pub reviewer: Option<ReviewerOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdeatorOutput {
    pub input: String,
    pub output: Vec<String>,
    pub duration_ms: u64,
    pub model: String,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComposerOutput {
    pub input_concept_index: usize,
    pub input: String,
    pub output: String,
    pub duration_ms: u64,
    pub model: String,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JudgeRanking {
    pub rank: u32,
    pub concept_index: usize,
    pub score: u32,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JudgeOutput {
    pub input: Vec<String>,
    pub output: Vec<JudgeRanking>,
    pub duration_ms: u64,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPair {
    pub positive: String,
    pub negative: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptEngineerOutput {
    pub input: String,
    pub checkpoint_context: Option<String>,
    pub output: PromptPair,
    pub duration_ms: u64,
    pub model: String,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewerOutput {
    pub approved: bool,
    pub issues: Option<Vec<String>>,
    pub suggested_positive: Option<String>,
    pub suggested_negative: Option<String>,
    pub duration_ms: u64,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEdits {
    pub prompt_edited: bool,
    pub edit_diff: Option<EditDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditDiff {
    pub positive_added: Vec<String>,
    pub positive_removed: Vec<String>,
    pub negative_added: Vec<String>,
    pub negative_removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationSettings {
    pub checkpoint: String,
    pub seed: i64,
    pub steps: u32,
    pub cfg: f64,
    pub sampler: String,
    pub scheduler: String,
    pub width: u32,
    pub height: u32,
}
