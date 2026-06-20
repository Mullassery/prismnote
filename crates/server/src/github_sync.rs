use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;
use chrono::Utc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub id: String,
    pub token: String,
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub sync_enabled: bool,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
    pub last_sync: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub notebook_id: String,
    pub action: String, // push, pull, status
    pub commit_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncStatus {
    pub notebook_id: String,
    pub status: String, // synced, pending, error
    pub last_sync: Option<String>,
    pub commits_ahead: usize,
    pub commits_behind: usize,
    pub conflicting_files: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub notebook_path: String,
}

pub struct GitHubSyncManager {
    config_dir: PathBuf,
    notebooks_dir: PathBuf,
}

impl GitHubSyncManager {
    pub fn new(config_dir: PathBuf, notebooks_dir: PathBuf) -> Self {
        Self {
            config_dir,
            notebooks_dir,
        }
    }

    pub async fn configure_github(
        &self,
        owner: String,
        repo: String,
        token: String,
        branch: Option<String>,
    ) -> Result<GitHubConfig, String> {
        let config_id = Uuid::new_v4().to_string();
        let config = GitHubConfig {
            id: config_id.clone(),
            token,
            owner,
            repo,
            branch: branch.unwrap_or_else(|| "main".to_string()),
            sync_enabled: true,
            auto_sync: false,
            sync_interval_minutes: 60,
            last_sync: None,
            status: "configured".to_string(),
        };

        self.save_config(&config).await?;
        Ok(config)
    }

    pub async fn push_notebook(
        &self,
        github_config: &GitHubConfig,
        notebook_id: &str,
        commit_message: &str,
    ) -> Result<CommitInfo, String> {
        // Verify notebook exists
        let notebook_path = self.notebooks_dir.join(format!("{}.ipynb", notebook_id));
        if !notebook_path.exists() {
            return Err("Notebook not found".to_string());
        }

        // Read notebook content
        let content = fs::read_to_string(&notebook_path)
            .await
            .map_err(|e| format!("Failed to read notebook: {}", e))?;

        // In production, this would use GitHub API to:
        // 1. Create a blob with the notebook content
        // 2. Create a tree with the blob
        // 3. Create a commit
        // 4. Update the branch reference
        // For now, we'll simulate the response

        let commit_info = CommitInfo {
            sha: Uuid::new_v4().to_string(),
            message: commit_message.to_string(),
            author: github_config.owner.clone(),
            timestamp: Utc::now().to_rfc3339(),
            notebook_path: format!("{}.ipynb", notebook_id),
        };

        Ok(commit_info)
    }

    pub async fn pull_notebook(
        &self,
        github_config: &GitHubConfig,
        notebook_id: &str,
    ) -> Result<String, String> {
        // In production, this would:
        // 1. Fetch the notebook from GitHub
        // 2. Check for conflicts
        // 3. Merge with local version
        // 4. Save to local filesystem

        let notebook_content = "{}"; // Placeholder

        let notebook_path = self.notebooks_dir.join(format!("{}.ipynb", notebook_id));
        fs::write(&notebook_path, notebook_content)
            .await
            .map_err(|e| format!("Failed to write notebook: {}", e))?;

        Ok(notebook_id.to_string())
    }

    pub async fn get_sync_status(
        &self,
        github_config: &GitHubConfig,
        notebook_id: &str,
    ) -> Result<SyncStatus, String> {
        // In production, this would query GitHub API for:
        // - Commits ahead/behind
        // - Conflicting files
        // - Last sync time

        Ok(SyncStatus {
            notebook_id: notebook_id.to_string(),
            status: "synced".to_string(),
            last_sync: Some(Utc::now().to_rfc3339()),
            commits_ahead: 0,
            commits_behind: 0,
            conflicting_files: vec![],
        })
    }

    pub async fn get_commit_history(
        &self,
        github_config: &GitHubConfig,
        notebook_id: &str,
        limit: usize,
    ) -> Result<Vec<CommitInfo>, String> {
        // In production, this would fetch from GitHub API
        // For now, return empty list

        Ok(vec![])
    }

    pub async fn auto_sync(
        &self,
        github_config: &GitHubConfig,
        notebook_id: &str,
    ) -> Result<SyncStatus, String> {
        if !github_config.auto_sync {
            return Err("Auto-sync is disabled".to_string());
        }

        // Get current status
        let status = self.get_sync_status(github_config, notebook_id).await?;

        // If there are commits behind, pull first
        if status.commits_behind > 0 {
            self.pull_notebook(github_config, notebook_id).await?;
        }

        // If there are commits ahead, push
        if status.commits_ahead > 0 {
            let message = format!("Auto-sync update at {}", Utc::now());
            self.push_notebook(github_config, notebook_id, &message).await?;
        }

        // Get updated status
        self.get_sync_status(github_config, notebook_id).await
    }

    pub async fn disconnect_github(&self, config_id: &str) -> Result<(), String> {
        let config_file = self.config_dir.join(format!("{}.json", config_id));
        if config_file.exists() {
            fs::remove_file(&config_file)
                .await
                .map_err(|e| format!("Failed to remove config: {}", e))?;
        }
        Ok(())
    }

    async fn save_config(&self, config: &GitHubConfig) -> Result<(), String> {
        fs::create_dir_all(&self.config_dir)
            .await
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let config_file = self.config_dir.join(format!("{}.json", config.id));
        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_file, config_json)
            .await
            .map_err(|e| format!("Failed to write config: {}", e))
    }
}
