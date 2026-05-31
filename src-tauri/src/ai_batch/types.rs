use serde::{Deserialize, Serialize};

/// The kind of AI operation in a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BatchOpKind {
    Tag,
    Caption,
}

/// Per-item status within a batch job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BatchItemStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// A single image within a batch job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchItem {
    pub image_id: String,
    pub filename: String,
    pub status: BatchItemStatus,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Overwrite policy for batch operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OverwritePolicy {
    Skip,
    Overwrite,
}

/// A batch job containing multiple images for a single operation type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchJob {
    pub id: String,
    pub op: BatchOpKind,
    pub model: String,
    pub overwrite_policy: OverwritePolicy,
    pub items: Vec<BatchItem>,
    pub status: BatchJobStatus,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub reordered: bool,
    pub reorder_note: Option<String>,
}

/// Overall batch job status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BatchJobStatus {
    Queued,
    Running,
    Completed,
    CompletedWithErrors,
    Cancelled,
}

/// Summary of a completed batch (for notification).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCompletionSummary {
    pub job_id: String,
    pub op: BatchOpKind,
    pub model: String,
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration_ms: u64,
    pub avg_duration_ms: u64,
}

/// ETA estimation context, keyed by (model, op, size_bucket).
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EtaKey {
    pub model: String,
    pub op: BatchOpKind,
    pub size_bucket: SizeBucket,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SizeBucket {
    Small,
    Medium,
    Large,
    Unknown,
}

impl SizeBucket {
    pub fn from_dimensions(width: Option<u32>, height: Option<u32>) -> Self {
        match (width, height) {
            (Some(w), Some(h)) => {
                let pixels = w as u64 * h as u64;
                if pixels < 500_000 {
                    Self::Small
                } else if pixels < 2_000_000 {
                    Self::Medium
                } else {
                    Self::Large
                }
            }
            _ => Self::Unknown,
        }
    }
}

/// Request to create a new batch job (received from frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest {
    pub op: BatchOpKind,
    pub image_ids: Vec<String>,
    pub overwrite_policy: OverwritePolicy,
}

/// Preview information shown before confirming a batch job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchPreview {
    pub model: String,
    pub total: usize,
    pub would_process: usize,
    pub would_skip: usize,
    pub op: BatchOpKind,
}
