use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub sha256: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub file: FileMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub success: bool,
    pub count: usize,
    pub files: Vec<FileMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub files_count: usize,
    pub total_bytes: u64,
}
