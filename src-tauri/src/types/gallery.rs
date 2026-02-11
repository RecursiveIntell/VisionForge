use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageEntry {
    pub id: String,
    pub filename: String,
    pub created_at: String,
    pub positive_prompt: Option<String>,
    pub negative_prompt: Option<String>,
    pub original_idea: Option<String>,
    pub checkpoint: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub steps: Option<u32>,
    pub cfg_scale: Option<f64>,
    pub sampler: Option<String>,
    pub scheduler: Option<String>,
    pub seed: Option<i64>,
    pub pipeline_log: Option<String>,
    pub selected_concept: Option<u32>,
    pub auto_approved: bool,
    pub caption: Option<String>,
    pub caption_edited: bool,
    pub rating: Option<u32>,
    pub favorite: bool,
    pub deleted: bool,
    pub user_note: Option<String>,
    pub tags: Option<Vec<TagEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagEntry {
    pub id: i64,
    pub name: String,
    pub source: Option<String>,
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct GalleryFilter {
    pub search: Option<String>,
    pub tags: Option<Vec<String>>,
    pub checkpoint: Option<String>,
    pub min_rating: Option<u32>,
    pub favorite_only: Option<bool>,
    pub show_deleted: Option<bool>,
    pub auto_approved: Option<bool>,
    pub sort_by: Option<GallerySortField>,
    pub sort_order: Option<SortOrder>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GallerySortField {
    CreatedAt,
    Rating,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    Asc,
    Desc,
}
