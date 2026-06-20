use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub cells: Vec<Cell>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub id: String,
    pub cell_type: String, // "code" or "markdown"
    pub source: Vec<String>,
    pub outputs: Vec<Output>,
    pub execution_count: Option<usize>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    pub output_type: String,
    pub data: Option<serde_json::Value>,
    pub text: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ExecuteCellRequest {
    pub cell_id: String,
}

#[derive(Serialize)]
pub struct ExecuteCellResponse {
    pub execution_count: usize,
    pub outputs: Vec<Output>,
}
