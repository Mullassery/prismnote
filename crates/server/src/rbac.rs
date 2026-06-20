use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Role {
    #[serde(rename = "owner")]
    Owner,
    #[serde(rename = "editor")]
    Editor,
    #[serde(rename = "commenter")]
    Commenter,
    #[serde(rename = "viewer")]
    Viewer,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserPermission {
    pub user_id: String,
    pub username: String,
    pub role: Role,
    pub granted_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotebookACL {
    pub notebook_id: String,
    pub owner: String,
    pub permissions: Vec<UserPermission>,
    pub public: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLog {
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub timestamp: String,
    pub details: String,
}

pub struct RBACManager {
    notebook_id: String,
    acl_dir: String,
}

impl RBACManager {
    pub fn new(notebook_id: String, base_dir: &str) -> Self {
        let acl_dir = format!("{}/.prismnote/acl", base_dir);
        let _ = fs::create_dir_all(&acl_dir);

        RBACManager {
            notebook_id,
            acl_dir,
        }
    }

    pub fn initialize_notebook(&self, owner_id: &str, owner_name: &str) -> Result<NotebookACL> {
        let acl = NotebookACL {
            notebook_id: self.notebook_id.clone(),
            owner: owner_id.to_string(),
            permissions: vec![UserPermission {
                user_id: owner_id.to_string(),
                username: owner_name.to_string(),
                role: Role::Owner,
                granted_at: chrono::Local::now().to_rfc3339(),
            }],
            public: false,
            created_at: chrono::Local::now().to_rfc3339(),
        };

        self.save_acl(&acl)?;
        Ok(acl)
    }

    pub fn grant_access(
        &self,
        user_id: &str,
        username: &str,
        role: Role,
    ) -> Result<NotebookACL> {
        let mut acl = self.load_acl()?;

        // Remove existing permission if any
        acl.permissions.retain(|p| p.user_id != user_id);

        // Add new permission
        acl.permissions.push(UserPermission {
            user_id: user_id.to_string(),
            username: username.to_string(),
            role,
            granted_at: chrono::Local::now().to_rfc3339(),
        });

        self.save_acl(&acl)?;
        Ok(acl)
    }

    pub fn revoke_access(&self, user_id: &str) -> Result<NotebookACL> {
        let mut acl = self.load_acl()?;
        acl.permissions.retain(|p| p.user_id != user_id);
        self.save_acl(&acl)?;
        Ok(acl)
    }

    pub fn check_permission(&self, user_id: &str, action: &str) -> Result<bool> {
        let acl = self.load_acl()?;

        let permission = acl
            .permissions
            .iter()
            .find(|p| p.user_id == user_id);

        match permission {
            Some(perm) => Ok(self.has_permission(&perm.role, action)),
            None => {
                if acl.public && (action == "view" || action == "read") {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub fn get_user_role(&self, user_id: &str) -> Result<Option<Role>> {
        let acl = self.load_acl()?;
        Ok(acl
            .permissions
            .iter()
            .find(|p| p.user_id == user_id)
            .map(|p| p.role.clone()))
    }

    pub fn log_audit(&self, user_id: &str, action: &str, resource: &str, details: &str) -> Result<()> {
        let log = AuditLog {
            user_id: user_id.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            timestamp: chrono::Local::now().to_rfc3339(),
            details: details.to_string(),
        };

        let audit_path = format!("{}/{}.log", self.acl_dir, self.notebook_id);
        let mut logs: Vec<AuditLog> = if Path::new(&audit_path).exists() {
            let content = fs::read_to_string(&audit_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            vec![]
        };

        logs.push(log);
        let content = serde_json::to_string_pretty(&logs)?;
        fs::write(&audit_path, content)?;

        Ok(())
    }

    pub fn get_audit_logs(&self) -> Result<Vec<AuditLog>> {
        let audit_path = format!("{}/{}.log", self.acl_dir, self.notebook_id);
        if Path::new(&audit_path).exists() {
            let content = fs::read_to_string(&audit_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(vec![])
        }
    }

    pub fn set_public(&self, public: bool) -> Result<NotebookACL> {
        let mut acl = self.load_acl()?;
        acl.public = public;
        self.save_acl(&acl)?;
        Ok(acl)
    }

    pub fn list_permissions(&self) -> Result<Vec<UserPermission>> {
        let acl = self.load_acl()?;
        Ok(acl.permissions)
    }

    fn has_permission(&self, role: &Role, action: &str) -> bool {
        match role {
            Role::Owner => true,
            Role::Editor => !matches!(action, "delete" | "share" | "admin"),
            Role::Commenter => matches!(action, "view" | "comment"),
            Role::Viewer => matches!(action, "view"),
        }
    }

    fn save_acl(&self, acl: &NotebookACL) -> Result<()> {
        let acl_path = format!("{}/{}.acl", self.acl_dir, self.notebook_id);
        let content = serde_json::to_string_pretty(acl)?;
        fs::write(&acl_path, content)?;
        Ok(())
    }

    fn load_acl(&self) -> Result<NotebookACL> {
        let acl_path = format!("{}/{}.acl", self.acl_dir, self.notebook_id);
        if Path::new(&acl_path).exists() {
            let content = fs::read_to_string(&acl_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Err(anyhow::anyhow!("ACL not initialized for notebook {}", self.notebook_id))
        }
    }
}
