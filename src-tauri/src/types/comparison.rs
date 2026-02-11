use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub id: String,
    pub image_a_id: String,
    pub image_b_id: String,
    pub variable_changed: String,
    pub note: Option<String>,
    pub created_at: Option<String>,
}
