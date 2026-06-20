use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotebookVersion {
    pub version_id: String,
    pub timestamp: String,
    pub author: String,
    pub message: String,
    pub cell_count: usize,
    pub parent_version: Option<String>,
    pub branch: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub versions: Vec<NotebookVersion>,
    pub current_version: String,
    pub current_branch: String,
    pub branches: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: String,
    pub to_version: String,
    pub cells_added: usize,
    pub cells_removed: usize,
    pub cells_modified: usize,
    pub changes: Vec<CellChange>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellChange {
    pub cell_id: String,
    pub change_type: String, // "added", "removed", "modified"
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}

pub struct VersionManager {
    notebook_id: String,
    versions_dir: String,
}

impl VersionManager {
    pub fn new(notebook_id: String, base_dir: &str) -> Self {
        let versions_dir = format!("{}/.prismnote/versions/{}", base_dir, notebook_id);
        let _ = fs::create_dir_all(&versions_dir);

        VersionManager {
            notebook_id,
            versions_dir,
        }
    }

    pub fn create_version(
        &self,
        notebook_content: &serde_json::Value,
        message: &str,
        author: &str,
    ) -> Result<String> {
        let version_id = uuid::Uuid::new_v4().to_string();
        let timestamp = Local::now().to_rfc3339();

        // Save version content
        let version_path = format!("{}/{}.json", self.versions_dir, version_id);
        let content = serde_json::to_string_pretty(notebook_content)?;
        fs::write(&version_path, content)?;

        // Update version metadata
        let metadata_path = format!("{}/.prismnote/versions/{}/.metadata.json", self.versions_dir, self.notebook_id);
        let mut metadata = self.load_metadata().unwrap_or(VersionMetadata {
            versions: vec![],
            current_version: version_id.clone(),
            current_branch: "main".to_string(),
            branches: HashMap::new(),
        });

        let cell_count = notebook_content["cells"]
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0);

        let version = NotebookVersion {
            version_id: version_id.clone(),
            timestamp,
            author: author.to_string(),
            message: message.to_string(),
            cell_count,
            parent_version: Some(metadata.current_version.clone()),
            branch: metadata.current_branch.clone(),
        };

        metadata.versions.push(version);
        metadata.current_version = version_id.clone();

        let metadata_content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, metadata_content)?;

        Ok(version_id)
    }

    pub fn rollback_to_version(&self, version_id: &str) -> Result<serde_json::Value> {
        let version_path = format!("{}/{}.json", self.versions_dir, version_id);
        let content = fs::read_to_string(&version_path)?;
        let notebook: serde_json::Value = serde_json::from_str(&content)?;

        // Update current version metadata
        let mut metadata = self.load_metadata()?;
        metadata.current_version = version_id.to_string();
        let metadata_path = format!("{}/.metadata.json", self.versions_dir);
        let metadata_content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, metadata_content)?;

        Ok(notebook)
    }

    pub fn get_version_diff(&self, from_id: &str, to_id: &str) -> Result<VersionDiff> {
        let from_content = self.load_version(from_id)?;
        let to_content = self.load_version(to_id)?;

        let empty_vec = vec![];
        let from_cells = from_content["cells"].as_array().unwrap_or(&empty_vec);
        let to_cells = to_content["cells"].as_array().unwrap_or(&empty_vec);

        let mut cells_added = 0;
        let mut cells_removed = 0;
        let mut cells_modified = 0;
        let mut changes = vec![];

        // Simple diff: count cells by ID
        let from_ids: std::collections::HashSet<_> = from_cells
            .iter()
            .filter_map(|c| c["id"].as_str())
            .collect();

        let to_ids: std::collections::HashSet<_> = to_cells
            .iter()
            .filter_map(|c| c["id"].as_str())
            .collect();

        // Added cells
        for to_cell in to_cells {
            if let Some(cell_id) = to_cell["id"].as_str() {
                if !from_ids.contains(cell_id) {
                    cells_added += 1;
                    changes.push(CellChange {
                        cell_id: cell_id.to_string(),
                        change_type: "added".to_string(),
                        old_content: None,
                        new_content: Some(serde_json::to_string(to_cell)?),
                    });
                }
            }
        }

        // Removed cells
        for from_cell in from_cells {
            if let Some(cell_id) = from_cell["id"].as_str() {
                if !to_ids.contains(cell_id) {
                    cells_removed += 1;
                    changes.push(CellChange {
                        cell_id: cell_id.to_string(),
                        change_type: "removed".to_string(),
                        old_content: Some(serde_json::to_string(from_cell)?),
                        new_content: None,
                    });
                }
            }
        }

        Ok(VersionDiff {
            from_version: from_id.to_string(),
            to_version: to_id.to_string(),
            cells_added,
            cells_removed,
            cells_modified,
            changes,
        })
    }

    pub fn list_versions(&self) -> Result<Vec<NotebookVersion>> {
        let metadata = self.load_metadata()?;
        Ok(metadata.versions)
    }

    pub fn create_branch(&self, branch_name: &str) -> Result<()> {
        let mut metadata = self.load_metadata()?;
        metadata.branches.insert(
            branch_name.to_string(),
            metadata.current_version.clone(),
        );

        let metadata_path = format!("{}/.metadata.json", self.versions_dir);
        let content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, content)?;

        Ok(())
    }

    pub fn switch_branch(&self, branch_name: &str) -> Result<()> {
        let mut metadata = self.load_metadata()?;

        if let Some(version_id) = metadata.branches.get(branch_name) {
            metadata.current_branch = branch_name.to_string();
            metadata.current_version = version_id.clone();

            let metadata_path = format!("{}/.metadata.json", self.versions_dir);
            let content = serde_json::to_string_pretty(&metadata)?;
            fs::write(&metadata_path, content)?;

            Ok(())
        } else {
            Err(anyhow::anyhow!("Branch {} not found", branch_name))
        }
    }

    fn load_version(&self, version_id: &str) -> Result<serde_json::Value> {
        let version_path = format!("{}/{}.json", self.versions_dir, version_id);
        let content = fs::read_to_string(&version_path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn load_metadata(&self) -> Result<VersionMetadata> {
        let metadata_path = format!("{}/.metadata.json", self.versions_dir);
        if Path::new(&metadata_path).exists() {
            let content = fs::read_to_string(&metadata_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(VersionMetadata {
                versions: vec![],
                current_version: "initial".to_string(),
                current_branch: "main".to_string(),
                branches: HashMap::new(),
            })
        }
    }
}
