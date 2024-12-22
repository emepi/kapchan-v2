use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub file_name: String,
    pub file_type: String,
}