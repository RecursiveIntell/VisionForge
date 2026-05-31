use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QueuePriority {
    High = 0,
    Normal = 1,
    Low = 2,
}

impl QueuePriority {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::High,
            2 => Self::Low,
            _ => Self::Normal,
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            Self::High => 0,
            Self::Normal => 1,
            Self::Low => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum QueueJobStatus {
    Pending,
    Generating,
    Completed,
    Failed,
    Cancelled,
}

impl QueueJobStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Generating => "generating",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "generating" => Some(Self::Generating),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueJob {
    pub id: String,
    pub priority: QueuePriority,
    pub status: QueueJobStatus,
    pub positive_prompt: String,
    pub negative_prompt: String,
    pub settings_json: String,
    pub pipeline_log: Option<String>,
    pub original_idea: Option<String>,
    pub linked_comparison_id: Option<String>,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub result_image_id: Option<String>,
}
