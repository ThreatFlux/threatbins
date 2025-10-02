use crate::error::{OfficeError, Result};
use crate::files::{DocumentFormat, FileResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct FileStorage {
    files: HashMap<String, StoredFile>,
    base_path: Option<PathBuf>,
    max_storage_size: usize,
    current_size: usize,
}

#[derive(Debug, Clone)]
pub struct StoredFile {
    pub id: String,
    pub result: FileResult,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub access_count: usize,
    pub tags: Vec<String>,
}

impl FileStorage {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            base_path: None,
            max_storage_size: 1024 * 1024 * 100, // 100MB default
            current_size: 0,
        }
    }

    pub fn with_base_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            files: HashMap::new(),
            base_path: Some(path.as_ref().to_path_buf()),
            max_storage_size: 1024 * 1024 * 100,
            current_size: 0,
        }
    }

    pub fn store_file(&mut self, file_result: FileResult) -> Result<String> {
        self.store_file_with_tags(file_result, Vec::new())
    }

    pub fn store_file_with_tags(
        &mut self,
        file_result: FileResult,
        tags: Vec<String>,
    ) -> Result<String> {
        if self.current_size + file_result.size() > self.max_storage_size {
            self.cleanup_old_files()?;

            if self.current_size + file_result.size() > self.max_storage_size {
                return Err(OfficeError::StorageFull);
            }
        }

        let id = Uuid::new_v4().to_string();
        let stored_file = StoredFile {
            id: id.clone(),
            result: file_result.clone(),
            created_at: chrono::Utc::now(),
            access_count: 0,
            tags,
        };

        self.current_size += file_result.size();
        self.files.insert(id.clone(), stored_file);

        if let Some(ref base_path) = self.base_path {
            self.persist_to_disk(&id, &file_result, base_path)?;
        }

        Ok(id)
    }

    pub fn retrieve_file(&mut self, id: &str) -> Result<FileResult> {
        if let Some(stored_file) = self.files.get_mut(id) {
            stored_file.access_count += 1;
            Ok(stored_file.result.clone())
        } else if let Some(ref base_path) = self.base_path {
            self.load_from_disk(id, base_path)
        } else {
            Err(OfficeError::FileNotFound(id.to_string()))
        }
    }

    pub fn list_files(&self) -> Vec<FileInfo> {
        self.files
            .values()
            .map(|stored_file| FileInfo {
                id: stored_file.id.clone(),
                filename: stored_file.result.filename.clone(),
                format: stored_file.result.format.clone(),
                size: stored_file.result.size(),
                created_at: stored_file.created_at,
                access_count: stored_file.access_count,
                tags: stored_file.tags.clone(),
            })
            .collect()
    }

    pub fn delete_file(&mut self, id: &str) -> Result<()> {
        if let Some(stored_file) = self.files.remove(id) {
            self.current_size = self.current_size.saturating_sub(stored_file.result.size());

            if let Some(ref base_path) = self.base_path {
                self.remove_from_disk(id, base_path)?;
            }

            Ok(())
        } else {
            Err(OfficeError::FileNotFound(id.to_string()))
        }
    }

    pub fn search_files(&self, query: &str) -> Vec<FileInfo> {
        self.files
            .values()
            .filter(|stored_file| {
                stored_file.result.filename.contains(query)
                    || stored_file
                        .result
                        .metadata
                        .title
                        .as_ref()
                        .map_or(false, |t| t.contains(query))
                    || stored_file
                        .result
                        .metadata
                        .keywords
                        .iter()
                        .any(|k| k.contains(query))
                    || stored_file.tags.iter().any(|t| t.contains(query))
            })
            .map(|stored_file| FileInfo {
                id: stored_file.id.clone(),
                filename: stored_file.result.filename.clone(),
                format: stored_file.result.format.clone(),
                size: stored_file.result.size(),
                created_at: stored_file.created_at,
                access_count: stored_file.access_count,
                tags: stored_file.tags.clone(),
            })
            .collect()
    }

    pub fn get_storage_stats(&self) -> StorageStats {
        let files_by_format = self
            .files
            .values()
            .fold(HashMap::new(), |mut acc, stored_file| {
                *acc.entry(stored_file.result.format.clone()).or_insert(0) += 1;
                acc
            });

        StorageStats {
            total_files: self.files.len(),
            current_size: self.current_size,
            max_size: self.max_storage_size,
            files_by_format,
            oldest_file: self
                .files
                .values()
                .min_by_key(|f| f.created_at)
                .map(|f| f.created_at),
            newest_file: self
                .files
                .values()
                .max_by_key(|f| f.created_at)
                .map(|f| f.created_at),
        }
    }

    pub fn set_max_storage_size(&mut self, size: usize) {
        self.max_storage_size = size;
    }

    pub fn clear_all(&mut self) -> Result<()> {
        if let Some(ref base_path) = self.base_path {
            for id in self.files.keys() {
                self.remove_from_disk(id, base_path)?;
            }
        }

        self.files.clear();
        self.current_size = 0;
        Ok(())
    }

    fn cleanup_old_files(&mut self) -> Result<()> {
        let mut files_by_age: Vec<_> = self
            .files
            .iter()
            .map(|(id, stored_file)| (id.clone(), stored_file.created_at))
            .collect();
        files_by_age.sort_by_key(|(_, created_at)| *created_at);

        let target_size = self.max_storage_size / 2;
        let mut ids_to_delete = Vec::new();

        for (id, _) in files_by_age {
            if self.current_size <= target_size {
                break;
            }
            if let Some(_stored_file) = self.files.get(&id) {
                ids_to_delete.push(id);
                if ids_to_delete.len() * 1024 >= self.current_size - target_size {
                    break;
                }
            }
        }

        for id in ids_to_delete {
            self.delete_file(&id)?;
        }

        Ok(())
    }

    fn persist_to_disk(&self, id: &str, file_result: &FileResult, base_path: &Path) -> Result<()> {
        let file_path = base_path.join(format!("{}.dat", id));
        let metadata_path = base_path.join(format!("{}.meta", id));

        std::fs::create_dir_all(base_path).map_err(|e| OfficeError::Io(e))?;

        std::fs::write(&file_path, &file_result.content).map_err(|e| OfficeError::Io(e))?;

        let metadata_json = serde_json::to_string_pretty(file_result).map_err(|_| {
            OfficeError::SerializationError("Failed to serialize file metadata".to_string())
        })?;

        std::fs::write(&metadata_path, metadata_json).map_err(|e| OfficeError::Io(e))?;

        Ok(())
    }

    fn load_from_disk(&self, id: &str, base_path: &Path) -> Result<FileResult> {
        let file_path = base_path.join(format!("{}.dat", id));
        let metadata_path = base_path.join(format!("{}.meta", id));

        if !file_path.exists() || !metadata_path.exists() {
            return Err(OfficeError::FileNotFound(id.to_string()));
        }

        let content = std::fs::read(&file_path).map_err(|e| OfficeError::Io(e))?;

        let metadata_json =
            std::fs::read_to_string(&metadata_path).map_err(|e| OfficeError::Io(e))?;

        let mut file_result: FileResult = serde_json::from_str(&metadata_json).map_err(|_| {
            OfficeError::InvalidFormat("Failed to deserialize file metadata".to_string())
        })?;

        file_result.content = content;
        Ok(file_result)
    }

    fn remove_from_disk(&self, id: &str, base_path: &Path) -> Result<()> {
        let file_path = base_path.join(format!("{}.dat", id));
        let metadata_path = base_path.join(format!("{}.meta", id));

        if file_path.exists() {
            std::fs::remove_file(&file_path).map_err(|e| OfficeError::Io(e))?;
        }

        if metadata_path.exists() {
            std::fs::remove_file(&metadata_path).map_err(|e| OfficeError::Io(e))?;
        }

        Ok(())
    }
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub id: String,
    pub filename: String,
    pub format: DocumentFormat,
    pub size: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub access_count: usize,
    pub tags: Vec<String>,
}

#[derive(Debug)]
pub struct StorageStats {
    pub total_files: usize,
    pub current_size: usize,
    pub max_size: usize,
    pub files_by_format: HashMap<DocumentFormat, usize>,
    pub oldest_file: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_file: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let storage = FileStorage::new();
        assert_eq!(storage.files.len(), 0);
        assert_eq!(storage.current_size, 0);
    }

    #[test]
    fn test_storage_stats() {
        let storage = FileStorage::new();
        let stats = storage.get_storage_stats();

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.current_size, 0);
    }
}
