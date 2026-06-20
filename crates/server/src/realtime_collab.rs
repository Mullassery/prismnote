use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CollabMessage {
    CellEdit {
        cell_id: String,
        content: String,
        cursor_position: usize,
        user_id: String,
    },
    CellInsert {
        index: usize,
        cell_type: String,
        user_id: String,
    },
    CellDelete {
        cell_id: String,
        user_id: String,
    },
    CursorMove {
        cell_id: String,
        user_id: String,
        position: usize,
        user_name: String,
    },
    SelectionChange {
        cell_id: String,
        user_id: String,
        start: usize,
        end: usize,
    },
    CommentAdd {
        cell_id: String,
        content: String,
        user_id: String,
        user_name: String,
        timestamp: String,
    },
    CommentReply {
        comment_id: String,
        content: String,
        user_id: String,
        user_name: String,
    },
}

#[derive(Clone, Debug)]
pub struct UserPresence {
    pub user_id: String,
    pub user_name: String,
    pub current_cell: Option<String>,
    pub cursor_position: usize,
    pub color: String,
    pub last_active: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub cell_id: String,
    pub content: String,
    pub author_id: String,
    pub author_name: String,
    pub created_at: String,
    pub replies: Vec<Comment>,
    pub resolved: bool,
}

pub struct RealtimeSession {
    pub notebook_id: String,
    pub users: Arc<RwLock<HashMap<String, UserPresence>>>,
    pub comments: Arc<RwLock<Vec<Comment>>>,
    pub tx: mpsc::UnboundedSender<CollabMessage>,
}

impl RealtimeSession {
    pub fn new(notebook_id: String, tx: mpsc::UnboundedSender<CollabMessage>) -> Self {
        Self {
            notebook_id,
            users: Arc::new(RwLock::new(HashMap::new())),
            comments: Arc::new(RwLock::new(Vec::new())),
            tx,
        }
    }

    pub async fn add_user(&self, presence: UserPresence) {
        let mut users = self.users.write().await;
        users.insert(presence.user_id.clone(), presence);
    }

    pub async fn remove_user(&self, user_id: &str) {
        let mut users = self.users.write().await;
        users.remove(user_id);
    }

    pub async fn broadcast(&self, message: CollabMessage) {
        let _ = self.tx.send(message);
    }

    pub async fn add_comment(&self, comment: Comment) {
        let mut comments = self.comments.write().await;
        comments.push(comment);
    }

    pub async fn resolve_comment(&self, comment_id: &str) {
        let mut comments = self.comments.write().await;
        if let Some(comment) = comments.iter_mut().find(|c| c.id == comment_id) {
            comment.resolved = true;
        }
    }

    pub async fn get_users(&self) -> Vec<UserPresence> {
        let users = self.users.read().await;
        users.values().cloned().collect()
    }

    pub async fn get_comments(&self) -> Vec<Comment> {
        let comments = self.comments.read().await;
        comments.clone()
    }
}

#[derive(Clone)]
pub struct RealtimeManager {
    pub sessions: Arc<RwLock<HashMap<String, Arc<RealtimeSession>>>>,
}

impl RealtimeManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(
        &self,
        notebook_id: String,
        tx: mpsc::UnboundedSender<CollabMessage>,
    ) -> Arc<RealtimeSession> {
        let session = Arc::new(RealtimeSession::new(notebook_id.clone(), tx));
        let mut sessions = self.sessions.write().await;
        sessions.insert(notebook_id, session.clone());
        session
    }

    pub async fn get_session(&self, notebook_id: &str) -> Option<Arc<RealtimeSession>> {
        let sessions = self.sessions.read().await;
        sessions.get(notebook_id).cloned()
    }

    pub async fn remove_session(&self, notebook_id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(notebook_id);
    }
}

impl Clone for Comment {
    fn clone(&self) -> Self {
        Comment {
            id: self.id.clone(),
            cell_id: self.cell_id.clone(),
            content: self.content.clone(),
            author_id: self.author_id.clone(),
            author_name: self.author_name.clone(),
            created_at: self.created_at.clone(),
            replies: self.replies.clone(),
            resolved: self.resolved,
        }
    }
}
