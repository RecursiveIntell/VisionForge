use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedEntry {
    pub id: Option<i64>,
    pub seed_value: i64,
    pub comment: String,
    pub checkpoint: Option<String>,
    pub sample_image_id: Option<String>,
    pub created_at: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeedCheckpointNote {
    pub seed_id: i64,
    pub checkpoint: String,
    pub note: String,
    pub sample_image_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct SeedFilter {
    pub search: Option<String>,
    pub checkpoint: Option<String>,
    pub tags: Option<Vec<String>>,
}
