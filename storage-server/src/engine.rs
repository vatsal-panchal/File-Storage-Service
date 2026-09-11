use crate::error::AppError;
use crate::models::FileMetadata;
use chrono::Utc;
use hex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use uuid::Uuid;

pub struct StorageEngine {
    base_dir: PathBuf,
    uploads_dir: PathBuf,
    index_file: PathBuf,
    files: RwLock<HashMap<String, FileMetadata>>,
}

impl StorageEngine {
    pub fn new(base_dir: PathBuf) -> Result<Self, AppError> {
        let uploads_dir = base_dir.join("uploads");
        let index_file = base_dir.join("index.json");

        fs::create_dir_all(&uploads_dir)?;

        let mut initial_files = HashMap::new();
        if index_file.exists() {
            let data = fs::read_to_string(&index_file)?;
            if let Ok(deserialized) = serde_json::from_str::<HashMap<String, FileMetadata>>(&data) {
                initial_files = deserialized;
            }
        }

        Ok(Self {
            base_dir,
            uploads_dir,
            index_file,
            files: RwLock::new(initial_files),
        })
    }

    fn persist(&self) -> Result<(), AppError> {
        let map = self.files.read().map_err(|e| {
            AppError::Internal(format!("Lock read error: {}", e))
        })?;
        let json = serde_json::to_string_pretty(&*map)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        fs::write(&self.index_file, json)?;
        Ok(())
    }

    pub fn save_file(
        &self,
        original_filename: String,
        content_type: Option<String>,
        data: &[u8],
    ) -> Result<FileMetadata, AppError> {
        let file_id = Uuid::new_v4().to_string();
        let safe_filename = std::path::Path::new(&original_filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed_file")
            .to_string();

        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash_result = hasher.finalize();
        let sha256_hex = hex::encode(hash_result);

        let stored_path = self.uploads_dir.join(&file_id);
        fs::write(&stored_path, data)?;

        let metadata = FileMetadata {
            id: file_id.clone(),
            filename: safe_filename,
            size: data.len() as u64,
            sha256: sha256_hex,
            content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            created_at: Utc::now(),
        };

        {
            let mut map = self.files.write().map_err(|e| {
                AppError::Internal(format!("Lock write error: {}", e))
            })?;
            map.insert(file_id, metadata.clone());
        }

        self.persist()?;

        Ok(metadata)
    }

    pub fn get_file(&self, id: &str) -> Result<(FileMetadata, Vec<u8>), AppError> {
        let metadata = self.get_metadata(id)?;
        let stored_path = self.uploads_dir.join(id);
        if !stored_path.exists() {
            return Err(AppError::NotFound(format!("File with id '{}' not found on disk", id)));
        }
        let bytes = fs::read(stored_path)?;
        Ok((metadata, bytes))
    }

    pub fn get_metadata(&self, id: &str) -> Result<FileMetadata, AppError> {
        let map = self.files.read().map_err(|e| {
            AppError::Internal(format!("Lock read error: {}", e))
        })?;
        map.get(id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("File with id '{}' not found", id)))
    }

    pub fn list_files(&self) -> Vec<FileMetadata> {
        self.search_files(None, None)
    }

    pub fn search_files(&self, query: Option<&str>, ext: Option<&str>) -> Vec<FileMetadata> {
        if let Ok(map) = self.files.read() {
            let mut list: Vec<FileMetadata> = map
                .values()
                .filter(|file| {
                    let matches_query = match query {
                        Some(q) => file.filename.to_lowercase().contains(&q.to_lowercase()),
                        None => true,
                    };
                    let matches_ext = match ext {
                        Some(e) => {
                            let expected = e.trim_start_matches('.').to_lowercase();
                            std::path::Path::new(&file.filename)
                                .extension()
                                .and_then(|ext_str| ext_str.to_str())
                                .map(|actual| actual.to_lowercase() == expected)
                                .unwrap_or(false)
                        }
                        None => true,
                    };
                    matches_query && matches_ext
                })
                .cloned()
                .collect();
            list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            list
        } else {
            Vec::new()
        }
    }

    pub fn file_exists(&self, id: &str) -> bool {
        if let Ok(map) = self.files.read() {
            map.contains_key(id)
        } else {
            false
        }
    }

    pub fn delete_file(&self, id: &str) -> Result<(), AppError> {
        {
            let mut map = self.files.write().map_err(|e| {
                AppError::Internal(format!("Lock write error: {}", e))
            })?;
            if map.remove(id).is_none() {
                return Err(AppError::NotFound(format!("File with id '{}' not found", id)));
            }
        }

        let stored_path = self.uploads_dir.join(id);
        if stored_path.exists() {
            let _ = fs::remove_file(stored_path);
        }

        self.persist()?;
        Ok(())
    }

    pub fn get_stats(&self) -> (usize, u64) {
        if let Ok(map) = self.files.read() {
            let count = map.len();
            let total_bytes: u64 = map.values().map(|f| f.size).sum();
            (count, total_bytes)
        } else {
            (0, 0)
        }
    }
}
