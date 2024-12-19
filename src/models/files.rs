use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub file_name: String,
    pub file_type: String,
}