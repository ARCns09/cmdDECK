use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CmdEntry {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub command: String,
    pub category: Option<String>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub confirmation_required: bool,
    pub working_directory: Option<String>,
}
