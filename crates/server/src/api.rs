use crate::ai::AIRequest;
use crate::models::{ExecuteCellRequest, ExecuteCellResponse, Notebook, Output};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize)]
pub struct NotebookList {
    pub notebooks: Vec<Notebook>,
}

#[derive(Deserialize)]
pub struct CreateNotebookRequest {
    pub name: String,
}

pub async fn list_notebooks(
    State(state): State<Arc<AppState>>,
) -> Json<NotebookList> {
    let dir = &state.notebooks_dir;
    let mut notebooks = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "ipynb") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(ipynb) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Ok(nb) = crate::files::from_ipynb(ipynb) {
                                    notebooks.push(nb);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    notebooks.sort_by(|a, b| a.name.cmp(&b.name));
    Json(NotebookList { notebooks })
}

pub async fn create_notebook(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateNotebookRequest>,
) -> (StatusCode, Json<Notebook>) {
    let id = Uuid::new_v4().to_string();
    let notebook = Notebook {
        id: id.clone(),
        name: req.name,
        cells: vec![],
        metadata: serde_json::json!({}),
    };

    let path = format!("{}/{}.ipynb", state.notebooks_dir, id);
    let ipynb = crate::files::to_ipynb(&notebook);
    let _ = std::fs::write(&path, serde_json::to_string_pretty(&ipynb).unwrap_or_default());

    (StatusCode::CREATED, Json(notebook))
}

pub async fn get_notebook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Option<Notebook>>) {
    let path = format!("{}/{}.ipynb", state.notebooks_dir, id);
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(ipynb) => match crate::files::from_ipynb(ipynb) {
                Ok(nb) => (StatusCode::OK, Json(Some(nb))),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
            },
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
        },
        Err(_) => (StatusCode::NOT_FOUND, Json(None)),
    }
}

pub async fn delete_notebook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> StatusCode {
    let path = format!("{}/{}.ipynb", state.notebooks_dir, id);
    match std::fs::remove_file(&path) {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

#[derive(Deserialize)]
pub struct UpdateNotebookRequest {
    pub notebook: Notebook,
}

pub async fn update_notebook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateNotebookRequest>,
) -> (StatusCode, Json<Notebook>) {
    let path = format!("{}/{}.ipynb", state.notebooks_dir, id);
    let ipynb = crate::files::to_ipynb(&req.notebook);

    match std::fs::write(&path, serde_json::to_string_pretty(&ipynb).unwrap_or_default()) {
        Ok(_) => (StatusCode::OK, Json(req.notebook)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(req.notebook)),
    }
}

pub async fn execute_cell(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteCellRequest>,
) -> (StatusCode, Json<ExecuteCellResponse>) {
    // Load notebook
    let path = format!("{}/{}.ipynb", state.notebooks_dir, id);
    let notebook = match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(ipynb) => match crate::files::from_ipynb(ipynb) {
                Ok(nb) => nb,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ExecuteCellResponse {
                            execution_count: 0,
                            outputs: vec![],
                        }),
                    )
                }
            },
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ExecuteCellResponse {
                        execution_count: 0,
                        outputs: vec![],
                    }),
                )
            }
        },
        Err(_) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ExecuteCellResponse {
                    execution_count: 0,
                    outputs: vec![],
                }),
            )
        }
    };

    // Find cell
    let cell = match notebook.cells.iter().find(|c| c.id == req.cell_id) {
        Some(c) => c,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ExecuteCellResponse {
                    execution_count: 0,
                    outputs: vec![],
                }),
            )
        }
    };

    // Only execute code cells
    if cell.cell_type != "code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(ExecuteCellResponse {
                execution_count: 0,
                outputs: vec![],
            }),
        );
    }

    let code = if let Some(source) = cell.source.first() {
        source.clone()
    } else {
        String::new()
    };

    if code.trim().is_empty() {
        return (
            StatusCode::OK,
            Json(ExecuteCellResponse {
                execution_count: 1,
                outputs: vec![],
            }),
        );
    }

    // Execute
    let mut kernel = state.kernel.lock().await;
    match kernel.as_mut() {
        Some(k) => match k.execute(&code).await {
            Ok((_stdout, outputs)) => {
                let response = ExecuteCellResponse {
                    execution_count: 1,
                    outputs: outputs
                        .into_iter()
                        .map(|out| Output {
                            output_type: out.get("output_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("stream")
                                .to_string(),
                            data: out.get("text").cloned(),
                            text: out
                                .get("text")
                                .and_then(|v| v.as_str())
                                .map(|s| vec![s.to_string()]),
                            metadata: None,
                        })
                        .collect(),
                };
                (StatusCode::OK, Json(response))
            }
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(ExecuteCellResponse {
                    execution_count: 1,
                    outputs: vec![Output {
                        output_type: "error".to_string(),
                        data: None,
                        text: Some(vec![e.to_string()]),
                        metadata: None,
                    }],
                }),
            ),
        },
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ExecuteCellResponse {
                execution_count: 0,
                outputs: vec![Output {
                    output_type: "error".to_string(),
                    data: None,
                    text: Some(vec!["Kernel not available. Install ipykernel: pip install ipykernel".to_string()]),
                    metadata: None,
                }],
            }),
        ),
    }
}

#[derive(Serialize)]
pub struct AIConfigResponse {
    pub configured: bool,
    pub provider: Option<String>,
}

pub async fn get_ai_config(
    State(state): State<Arc<AppState>>,
) -> Json<AIConfigResponse> {
    let configured = state.ai_engine.is_some();
    let provider = if let Some(_) = &state.ai_engine {
        Some("configured".to_string())
    } else {
        None
    };
    Json(AIConfigResponse { configured, provider })
}

#[derive(Deserialize)]
pub struct AIConfigRequest {
    pub provider: String,
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
    pub claude_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    pub openai_model: Option<String>,
}

pub async fn set_ai_config(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AIConfigRequest>,
) -> (StatusCode, Json<AIConfigResponse>) {
    // Store config in environment or file
    if let Ok(_) = std::env::var("PRISMNOTE_AI_PROVIDER") {
        (StatusCode::OK, Json(AIConfigResponse {
            configured: true,
            provider: Some(req.provider),
        }))
    } else {
        (StatusCode::BAD_REQUEST, Json(AIConfigResponse {
            configured: false,
            provider: None,
        }))
    }
}

#[derive(Serialize)]
pub struct AIResponseData {
    pub suggestion: String,
}

pub async fn ai_explain(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AIRequest>,
) -> (StatusCode, Json<AIResponseData>) {
    match &state.ai_engine {
        Some(engine) => match engine.explain(&req.code).await {
            Ok(suggestion) => (StatusCode::OK, Json(AIResponseData { suggestion })),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AIResponseData {
                    suggestion: "Error getting AI response".to_string(),
                }),
            ),
        },
        None => (
            StatusCode::BAD_REQUEST,
            Json(AIResponseData {
                suggestion: "AI not configured".to_string(),
            }),
        ),
    }
}

pub async fn ai_fix(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AIRequest>,
) -> (StatusCode, Json<AIResponseData>) {
    match &state.ai_engine {
        Some(engine) => {
            let error = req.error.as_deref().unwrap_or("Unknown error");
            match engine.fix_error(&req.code, error).await {
                Ok(suggestion) => (StatusCode::OK, Json(AIResponseData { suggestion })),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AIResponseData {
                        suggestion: "Error getting AI response".to_string(),
                    }),
                ),
            }
        }
        None => (
            StatusCode::BAD_REQUEST,
            Json(AIResponseData {
                suggestion: "AI not configured".to_string(),
            }),
        ),
    }
}

pub async fn ai_complete(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AIRequest>,
) -> (StatusCode, Json<AIResponseData>) {
    match &state.ai_engine {
        Some(engine) => match engine.complete_code(&req.code, req.context.as_deref()).await {
            Ok(suggestion) => (StatusCode::OK, Json(AIResponseData { suggestion })),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AIResponseData {
                    suggestion: "Error getting AI response".to_string(),
                }),
            ),
        },
        None => (
            StatusCode::BAD_REQUEST,
            Json(AIResponseData {
                suggestion: "AI not configured".to_string(),
            }),
        ),
    }
}
