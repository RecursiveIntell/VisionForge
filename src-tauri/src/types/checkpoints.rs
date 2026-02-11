use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointProfile {
    pub id: Option<i64>,
    pub filename: String,
    pub display_name: Option<String>,
    pub base_model: Option<String>,
    pub created_at: Option<String>,
    pub strengths: Option<Vec<String>>,
    pub weaknesses: Option<Vec<String>>,
    pub preferred_cfg: Option<f64>,
    pub cfg_range_low: Option<f64>,
    pub cfg_range_high: Option<f64>,
    pub preferred_sampler: Option<String>,
    pub preferred_scheduler: Option<String>,
    pub optimal_resolution: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTerm {
    pub id: Option<i64>,
    pub checkpoint_id: i64,
    pub term: String,
    pub effect: String,
    pub strength: TermStrength,
    pub example_image_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TermStrength {
    Strong,
    Moderate,
    Weak,
    Broken,
}

impl TermStrength {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Strong => "strong",
            Self::Moderate => "moderate",
            Self::Weak => "weak",
            Self::Broken => "broken",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "strong" => Some(Self::Strong),
            "moderate" => Some(Self::Moderate),
            "weak" => Some(Self::Weak),
            "broken" => Some(Self::Broken),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckpointObservation {
    pub id: Option<i64>,
    pub checkpoint_id: i64,
    pub observation: String,
    pub source: ObservationSource,
    pub comparison_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObservationSource {
    User,
    AbComparison,
    PipelineNote,
    AutoRating,
}

impl ObservationSource {
    pub fn as_str(&self) -> &str {
        match self {
            Self::User => "user",
            Self::AbComparison => "ab_comparison",
            Self::PipelineNote => "pipeline_note",
            Self::AutoRating => "auto_rating",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "user" => Some(Self::User),
            "ab_comparison" => Some(Self::AbComparison),
            "pipeline_note" => Some(Self::PipelineNote),
            "auto_rating" => Some(Self::AutoRating),
            _ => None,
        }
    }
}
