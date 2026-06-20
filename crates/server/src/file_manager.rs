use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
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

// Cloud Storage Integration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudStorageMount {
    pub id: String,
    pub provider: String, // s3, gcs, azure, gdrive
    pub bucket_or_path: String,
    pub mount_point: String,
    pub credentials: HashMap<String, String>,
    pub mounted_at: String,
    pub status: String, // active, inactive, error
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudStorageMountRequest {
    pub provider: String,
    pub bucket_or_path: String,
    pub mount_point: String,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub project_id: Option<String>,
    pub credentials_json: Option<String>,
}

pub struct CloudStorageManager {
    config_dir: PathBuf,
    mounts: HashMap<String, CloudStorageMount>,
}

impl CloudStorageManager {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            mounts: HashMap::new(),
        }
    }

    pub async fn mount_cloud_storage(
        &mut self,
        request: CloudStorageMountRequest,
    ) -> Result<CloudStorageMount, String> {
        // Validate provider
        let valid_providers = vec!["s3", "gcs", "azure", "gdrive"];
        if !valid_providers.contains(&request.provider.as_str()) {
            return Err(format!(
                "Invalid provider '{}'. Must be one of: {}",
                request.provider,
                valid_providers.join(", ")
            ));
        }

        // Create mount configuration
        let mount_id = Uuid::new_v4().to_string();
        let mut credentials = HashMap::new();

        // Store credentials securely (in production, use vault)
        match request.provider.as_str() {
            "s3" => {
                if let Some(key) = request.access_key {
                    credentials.insert("access_key".to_string(), key);
                }
                if let Some(secret) = request.secret_key {
                    credentials.insert("secret_key".to_string(), secret);
                }
            }
            "gcs" => {
                if let Some(creds) = request.credentials_json {
                    credentials.insert("credentials".to_string(), creds);
                }
                if let Some(project) = request.project_id {
                    credentials.insert("project_id".to_string(), project);
                }
            }
            "azure" => {
                if let Some(key) = request.access_key {
                    credentials.insert("account_key".to_string(), key);
                }
                if let Some(secret) = request.secret_key {
                    credentials.insert("sas_token".to_string(), secret);
                }
            }
            "gdrive" => {
                if let Some(creds) = request.credentials_json {
                    credentials.insert("credentials".to_string(), creds);
                }
            }
            _ => {}
        }

        let mount = CloudStorageMount {
            id: mount_id.clone(),
            provider: request.provider,
            bucket_or_path: request.bucket_or_path,
            mount_point: request.mount_point,
            credentials,
            mounted_at: Utc::now().to_rfc3339(),
            status: "active".to_string(),
        };

        // Persist mount configuration
        self.save_mount(&mount).await?;
        self.mounts.insert(mount_id, mount.clone());

        Ok(mount)
    }

    pub async fn list_mounts(&self) -> Result<Vec<CloudStorageMount>, String> {
        Ok(self.mounts.values().cloned().collect())
    }

    pub async fn unmount_storage(&mut self, mount_id: String) -> Result<(), String> {
        if !self.mounts.contains_key(&mount_id) {
            return Err("Mount not found".to_string());
        }

        let config_file = self.config_dir.join(format!("{}.json", mount_id));
        if config_file.exists() {
            fs::remove_file(&config_file)
                .await
                .map_err(|e| e.to_string())?;
        }

        self.mounts.remove(&mount_id);
        Ok(())
    }

    pub async fn get_mount(&self, mount_id: &str) -> Option<CloudStorageMount> {
        self.mounts.get(mount_id).cloned()
    }

    async fn save_mount(&self, mount: &CloudStorageMount) -> Result<(), String> {
        fs::create_dir_all(&self.config_dir)
            .await
            .map_err(|e| e.to_string())?;

        let config_file = self.config_dir.join(format!("{}.json", mount.id));
        let config_json = serde_json::to_string_pretty(mount)
            .map_err(|e| e.to_string())?;

        fs::write(&config_file, config_json)
            .await
            .map_err(|e| e.to_string())
    }

    async fn load_mounts(&mut self) -> Result<(), String> {
        if !self.config_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&self.config_dir)
            .await
            .map_err(|e| e.to_string())?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    if let Ok(content) = fs::read_to_string(entry.path()).await {
                        if let Ok(mount) = serde_json::from_str::<CloudStorageMount>(&content) {
                            self.mounts.insert(mount.id.clone(), mount);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
