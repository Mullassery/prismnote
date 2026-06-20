use crate::ai::AIRequest;
use crate::models::{ExecuteCellRequest, ExecuteCellResponse, Notebook, Output};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;
use chrono;

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
        prismnote_metadata: Some(crate::models::NotebookMetadata {
            ignored_libraries: vec![],
            library_suggestions_enabled: true,
        }),
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
        Some(k) => {
            // Check for SQL cell marker
            let is_sql_cell = code.trim().starts_with("--sql") || code.trim().starts_with("%sql");

            let result = if is_sql_cell {
                execute_sql_cell(&code, &state).await
            } else {
                match k.execute(&code).await {
                    Ok((_stdout, outputs)) => Ok(outputs),
                    Err(e) => Err(e),
                }
            };

            match result {
                Ok(outputs) => {
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
            }
        }
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

async fn execute_sql_cell(code: &str, _state: &Arc<AppState>) -> anyhow::Result<Vec<serde_json::Value>> {
    let sql_code = code
        .trim()
        .strip_prefix("--sql")
        .or_else(|| code.trim().strip_prefix("%sql"))
        .unwrap_or(code)
        .trim();

    // Placeholder for SQL execution
    // Full implementation would parse connection ID and execute via database manager

    Ok(vec![json!({
        "output_type": "stream",
        "name": "stdout",
        "text": format!("SQL Query (v0.2 feature): {}\n\n[Full SQL execution coming in v0.2 - configure database connections]", sql_code)
    })])
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

// Database connectors
#[derive(Serialize)]
pub struct DatabaseList {
    pub databases: Vec<crate::db::DatabaseConnection>,
}

pub async fn list_databases() -> Json<DatabaseList> {
    // TODO: Load from ~/.prismnote/databases.json
    Json(DatabaseList {
        databases: vec![],
    })
}

pub async fn create_database(
    Json(mut req): Json<crate::db::DatabaseConnection>,
) -> (StatusCode, Json<crate::db::DatabaseConnection>) {
    req.id = Uuid::new_v4().to_string();
    req.created_at = chrono::Local::now().to_rfc3339();

    if let Err(_) = crate::db::DatabaseManager::validate_connection(&req) {
        return (StatusCode::BAD_REQUEST, Json(req));
    }

    // TODO: Save to ~/.prismnote/databases.json
    (StatusCode::CREATED, Json(req))
}

pub async fn test_database(
    Path(_id): Path<String>,
    Json(req): Json<crate::db::DatabaseConnection>,
) -> (StatusCode, Json<serde_json::Value>) {
    match crate::db::DatabaseManager::test_connection(&req).await {
        Ok(message) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": message
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "error",
                "message": e.to_string()
            })),
        ),
    }
}

pub async fn execute_database_query(
    Path(_id): Path<String>,
    Json(_req): Json<crate::db::QueryRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // TODO: Load connection from store
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "Database connectors not yet implemented. Available: PostgreSQL, MySQL, SQLite, DuckDB, MongoDB"
        })),
    )
}

pub async fn delete_database(Path(_id): Path<String>) -> StatusCode {
    // TODO: Delete from ~/.prismnote/databases.json
    StatusCode::NO_CONTENT
}

// Library recommendations
pub async fn suggest_libraries(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
    Json(req): Json<crate::library_advisor::SuggestLibrariesRequest>,
) -> (StatusCode, Json<crate::library_advisor::SuggestionsResponse>) {
    let advisor = crate::library_advisor::LibraryAdvisor::new(state.ai_engine.clone());

    match advisor.suggest_libraries(
        &req.notebook_code,
        req.installed_packages,
        req.ignored_libraries,
    ).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(crate::library_advisor::SuggestionsResponse {
                suggestions: vec![],
                detected_intent: "Error analyzing code".to_string(),
                context_summary: "Try again later".to_string(),
            }),
        ),
    }
}

pub async fn ignore_library(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<crate::library_advisor::IgnoreLibraryRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let path = format!("{}/{}.ipynb", state.notebooks_dir, id);

    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(mut ipynb) => {
                let now = chrono::Local::now().to_rfc3339();

                // Ensure prismnote metadata exists
                if !ipynb["metadata"]["prismnote"].is_object() {
                    ipynb["metadata"]["prismnote"] = json!({});
                }

                if !ipynb["metadata"]["prismnote"]["ignored_libraries"].is_array() {
                    ipynb["metadata"]["prismnote"]["ignored_libraries"] = json!([]);
                }

                // Add to ignored list
                ipynb["metadata"]["prismnote"]["ignored_libraries"].as_array_mut().unwrap().push(json!({
                    "name": req.library_name,
                    "reason": req.reason,
                    "ignored_at": now,
                }));

                match std::fs::write(&path, serde_json::to_string_pretty(&ipynb).unwrap_or_default()) {
                    Ok(_) => (StatusCode::OK, Json(json!({"status": "ignored"}))),
                    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to save"}))),
                }
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Invalid notebook"}))),
        },
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({"error": "Notebook not found"}))),
    }
}

pub async fn get_ignored_libraries(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let path = format!("{}/{}.ipynb", state.notebooks_dir, id);

    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(ipynb) => {
                let ignored = ipynb["metadata"]["prismnote"]["ignored_libraries"].clone();
                (StatusCode::OK, Json(json!({"ignored": ignored})))
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"ignored": []}))),
        },
        Err(_) => (StatusCode::NOT_FOUND, Json(json!({"ignored": []}))),
    }
}

// SQL cell execution
#[derive(Deserialize)]
pub struct ExecuteSQLRequest {
    pub query: String,
    pub connection_id: String,
}

#[derive(Serialize)]
pub struct ExecuteSQLResponse {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}

pub async fn execute_sql(
    Json(req): Json<ExecuteSQLRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let optimizations = crate::sql_executor::SQLExecutor::analyze_query(&req.query);

    // Format result as HTML table
    let query_result = crate::sql_executor::SQLExecutor::execute_query(&req.query, &req.connection_id)
        .await
        .unwrap_or_else(|_| crate::sql_executor::QueryResult {
            columns: vec![],
            rows: vec![],
            row_count: 0,
            execution_time_ms: 0,
            estimated_memory_bytes: 0,
        });

    let result_html = crate::sql_executor::SQLExecutor::format_result_as_html(&query_result);

    (
        StatusCode::OK,
        Json(json!({
            "html": result_html,
            "optimizations": optimizations,
            "row_count": query_result.row_count,
            "execution_time_ms": query_result.execution_time_ms,
        })),
    )
}

pub async fn get_query_optimizations(
    Json(req): Json<ExecuteSQLRequest>,
) -> Json<serde_json::Value> {
    let optimizations = crate::sql_executor::SQLExecutor::analyze_query(&req.query);
    Json(json!({
        "optimizations": optimizations,
        "total_issues": optimizations.len(),
        "high_priority": optimizations.iter().filter(|o| o.severity == "high").count(),
    }))
}

// Spark session management
#[derive(Deserialize)]
pub struct CreateSparkSessionRequest {
    pub app_name: String,
    pub executor_memory: Option<String>,
    pub executor_cores: Option<u32>,
}

pub async fn create_spark_session(
    Json(req): Json<CreateSparkSessionRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let config = crate::spark_manager::SparkConfig {
        app_name: req.app_name,
        master: "local[*]".to_string(),
        executor_memory: req.executor_memory.unwrap_or_else(|| "2g".to_string()),
        driver_memory: "1g".to_string(),
        executor_cores: req.executor_cores.unwrap_or(4),
        executor_instances: 1,
        shuffle_partitions: 200,
    };

    let mut manager = crate::spark_manager::SparkManager::new();
    match manager.create_session(config) {
        Ok(session) => (StatusCode::CREATED, Json(json!(session))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn list_spark_sessions() -> Json<serde_json::Value> {
    let manager = crate::spark_manager::SparkManager::new();
    let sessions = manager.list_sessions();
    Json(json!({
        "sessions": sessions,
        "total": sessions.len(),
    }))
}

pub async fn get_spark_session(
    Path(app_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = crate::spark_manager::SparkManager::new();
    match manager.get_session(&app_id) {
        Some(session) => (StatusCode::OK, Json(json!(session))),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Session not found"})),
        ),
    }
}

// Execution pipeline
#[derive(Deserialize)]
pub struct BuildExecutionPlanRequest {
    pub cells: Vec<crate::execution_pipeline::CellNode>,
}

pub async fn build_execution_plan(
    Path(notebook_id): Path<String>,
    Json(req): Json<BuildExecutionPlanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut pipeline = crate::execution_pipeline::ExecutionPipeline::new();
    match pipeline.build_plan(notebook_id, req.cells) {
        Ok(plan) => (
            StatusCode::OK,
            Json(json!({
                "execution_order": plan.execution_order,
                "total_cells": plan.nodes.len(),
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn get_execution_statistics(
    Path(notebook_id): Path<String>,
) -> Json<serde_json::Value> {
    let pipeline = crate::execution_pipeline::ExecutionPipeline::new();
    let stats = pipeline.get_execution_statistics(&notebook_id);
    Json(json!(stats))
}
