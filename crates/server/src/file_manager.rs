use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub notebook_id: String,
    pub filename: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub uploaded_by: String,
    pub uploaded_at: String,
    pub file_path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadProgress {
    pub upload_id: String,
    pub filename: String,
    pub bytes_uploaded: u64,
    pub total_bytes: u64,
    pub percentage: f32,
    pub status: String,
}

pub struct FileManager {
    pub base_dir: PathBuf,
    pub max_file_size: u64,
}

impl FileManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            max_file_size: 500 * 1024 * 1024, // 500MB default
        }
    }

    pub async fn upload_file(
        &self,
        notebook_id: &str,
        filename: &str,
        content: Vec<u8>,
        mime_type: String,
        user_id: String,
    ) -> Result<FileMetadata, String> {
        // Validate file size
        if content.len() as u64 > self.max_file_size {
            return Err(format!(
                "File size {} exceeds maximum {}",
                content.len(),
                self.max_file_size
            ));
        }

        // Create notebook-specific directory
        let notebook_dir = self.base_dir.join("files").join(notebook_id);
        fs::create_dir_all(&notebook_dir)
            .await
            .map_err(|e| e.to_string())?;

        // Generate unique filename
        let file_id = Uuid::new_v4().to_string();
        let file_extension = filename.split('.').last().unwrap_or("");
        let safe_filename = format!("{}_{}", file_id, filename);
        let file_path = notebook_dir.join(&safe_filename);

        // Write file
        fs::write(&file_path, &content)
            .await
            .map_err(|e| e.to_string())?;

        Ok(FileMetadata {
            id: file_id,
            notebook_id: notebook_id.to_string(),
            filename: filename.to_string(),
            size_bytes: content.len() as u64,
            mime_type,
            uploaded_by: user_id,
            uploaded_at: Utc::now().to_rfc3339(),
            file_path: safe_filename,
        })
    }

    pub async fn download_file(
        &self,
        notebook_id: &str,
        file_id: &str,
    ) -> Result<Vec<u8>, String> {
        let notebook_dir = self.base_dir.join("files").join(notebook_id);

        // Find file with matching ID
        let mut entries = fs::read_dir(&notebook_dir)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let filename = entry.file_name();
            if filename
                .to_string_lossy()
                .starts_with(&format!("{}_", file_id))
            {
                let content = fs::read(entry.path())
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok(content);
            }
        }

        Err("File not found".to_string())
    }

    pub async fn delete_file(
        &self,
        notebook_id: &str,
        file_id: &str,
    ) -> Result<(), String> {
        let notebook_dir = self.base_dir.join("files").join(notebook_id);

        let mut entries = fs::read_dir(&notebook_dir)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let filename = entry.file_name();
            if filename
                .to_string_lossy()
                .starts_with(&format!("{}_", file_id))
            {
                fs::remove_file(entry.path())
                    .await
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
        }

        Err("File not found".to_string())
    }

    pub async fn list_files(&self, notebook_id: &str) -> Result<Vec<FileMetadata>, String> {
        let notebook_dir = self.base_dir.join("files").join(notebook_id);

        if !notebook_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        let mut entries = fs::read_dir(&notebook_dir)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            if let Ok(metadata) = entry.metadata().await {
                let filename = entry.file_name();
                files.push(FileMetadata {
                    id: filename.to_string_lossy().split('_').next().unwrap_or("").to_string(),
                    notebook_id: notebook_id.to_string(),
                    filename: filename.to_string_lossy().to_string(),
                    size_bytes: metadata.len(),
                    mime_type: "application/octet-stream".to_string(),
                    uploaded_by: "unknown".to_string(),
                    uploaded_at: Utc::now().to_rfc3339(),
                    file_path: filename.to_string_lossy().to_string(),
                });
            }
        }

        Ok(files)
    }
}
