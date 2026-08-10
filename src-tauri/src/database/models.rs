use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: String,
    pub name: String,
    pub format: String,
    pub windows_format: String,
    pub object_type: String,
    pub xml: String,
    pub checksum: String,
    pub filemaker_version: Option<String>,
    pub notes: String,
    pub favorite: bool,
    pub in_library: bool,
    pub in_history: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_used_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveClipboardItem {
    pub id: Option<String>,
    pub name: String,
    pub format: String,
    pub windows_format: String,
    pub object_type: String,
    pub xml: String,
    pub filemaker_version: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub in_library: bool,
    #[serde(default = "default_true")]
    pub in_history: bool,
}

fn default_true() -> bool {
    true
}
