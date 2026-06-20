use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SearchCategory {
    #[serde(rename = "notebook")]
    Notebook,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "table")]
    Table,
    #[serde(rename = "variable")]
    Variable,
    #[serde(rename = "history")]
    History,
    #[serde(rename = "comment")]
    Comment,
    #[serde(rename = "chat")]
    Chat,
    #[serde(rename = "connection")]
    Connection,
}

impl std::fmt::Display for SearchCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SearchCategory::Notebook => write!(f, "notebook"),
            SearchCategory::File => write!(f, "file"),
            SearchCategory::Table => write!(f, "table"),
            SearchCategory::Variable => write!(f, "variable"),
            SearchCategory::History => write!(f, "history"),
            SearchCategory::Comment => write!(f, "comment"),
            SearchCategory::Chat => write!(f, "chat"),
            SearchCategory::Connection => write!(f, "connection"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub category: SearchCategory,
    pub content: String,
    pub context: Option<String>,
    pub path: Option<String>,
    pub timestamp: Option<String>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchableItem {
    pub id: String,
    pub title: String,
    pub category: SearchCategory,
    pub content: String,
    pub context: Option<String>,
    pub path: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}
