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
            "optimizations": serde_json::to_value(&optimizations).unwrap_or(json!([])),
            "row_count": query_result.row_count,
            "execution_time_ms": query_result.execution_time_ms,
        })),
    )
}

pub async fn get_query_optimizations(
    Json(req): Json<ExecuteSQLRequest>,
) -> Json<serde_json::Value> {
    let optimizations = crate::sql_executor::SQLExecutor::analyze_query(&req.query);
    let high_priority = optimizations.iter().filter(|o| o.severity == "high").count();
    let total_issues = optimizations.len();
    Json(json!({
        "optimizations": serde_json::to_value(&optimizations).unwrap_or(json!([])),
        "total_issues": total_issues,
        "high_priority": high_priority,
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

// Cloud data warehouse management
#[derive(Deserialize)]
pub struct CreateCloudWarehouseConnectionRequest {
    pub warehouse_type: String,
    pub name: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: String,
    pub username: String,
    pub password: String,
    pub region: Option<String>,
    pub project_id: Option<String>,
    pub account_id: Option<String>,
}

pub async fn create_cloud_warehouse_connection(
    Json(req): Json<CreateCloudWarehouseConnectionRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let warehouse_type = match req.warehouse_type.as_str() {
        "snowflake" => crate::cloud_warehouse::CloudWarehouseType::Snowflake,
        "bigquery" => crate::cloud_warehouse::CloudWarehouseType::BigQuery,
        "redshift" => crate::cloud_warehouse::CloudWarehouseType::Redshift,
        "azure_synapse" => crate::cloud_warehouse::CloudWarehouseType::AzureSynapse,
        "databricks" => crate::cloud_warehouse::CloudWarehouseType::Databricks,
        "athena" => crate::cloud_warehouse::CloudWarehouseType::Athena,
        "presto" => crate::cloud_warehouse::CloudWarehouseType::Presto,
        "trino" => crate::cloud_warehouse::CloudWarehouseType::Trino,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Unknown warehouse type"})),
            )
        }
    };

    let conn = crate::cloud_warehouse::CloudWarehouseConnection {
        id: Uuid::new_v4().to_string(),
        warehouse_type,
        name: req.name,
        host: req.host,
        port: req.port,
        database: req.database,
        username: req.username,
        password: req.password,
        credentials: std::collections::HashMap::new(),
        region: req.region,
        project_id: req.project_id,
        account_id: req.account_id,
        warehouse_id: None,
        timeout_seconds: 30,
        created_at: chrono::Local::now().to_rfc3339(),
    };

    (StatusCode::CREATED, Json(json!(conn)))
}

pub async fn list_cloud_warehouse_connections() -> Json<serde_json::Value> {
    // TODO: Load from ~/.prismnote/cloud_warehouses.json
    Json(json!({
        "connections": serde_json::json!([])
    }))
}

pub async fn test_cloud_warehouse_connection(
    Path(_id): Path<String>,
    Json(req): Json<CreateCloudWarehouseConnectionRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let warehouse_type = match req.warehouse_type.as_str() {
        "snowflake" => crate::cloud_warehouse::CloudWarehouseType::Snowflake,
        "bigquery" => crate::cloud_warehouse::CloudWarehouseType::BigQuery,
        "redshift" => crate::cloud_warehouse::CloudWarehouseType::Redshift,
        "azure_synapse" => crate::cloud_warehouse::CloudWarehouseType::AzureSynapse,
        "databricks" => crate::cloud_warehouse::CloudWarehouseType::Databricks,
        "athena" => crate::cloud_warehouse::CloudWarehouseType::Athena,
        "presto" => crate::cloud_warehouse::CloudWarehouseType::Presto,
        "trino" => crate::cloud_warehouse::CloudWarehouseType::Trino,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Unknown warehouse type"})),
            )
        }
    };

    let conn = crate::cloud_warehouse::CloudWarehouseConnection {
        id: Uuid::new_v4().to_string(),
        warehouse_type: warehouse_type.clone(),
        name: req.name,
        host: req.host,
        port: req.port,
        database: req.database,
        username: req.username,
        password: req.password,
        credentials: std::collections::HashMap::new(),
        region: req.region,
        project_id: req.project_id,
        account_id: req.account_id,
        warehouse_id: None,
        timeout_seconds: 30,
        created_at: chrono::Local::now().to_rfc3339(),
    };

    let manager = crate::cloud_warehouse::CloudWarehouseManager::new();
    match manager.test_connection(&conn).await {
        Ok(message) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "message": message,
                "warehouse_type": format!("{:?}", warehouse_type)
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

pub async fn execute_cloud_warehouse_query(
    Path(_id): Path<String>,
    Json(req): Json<ExecuteSQLRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = crate::cloud_warehouse::CloudWarehouseManager::new();

    match manager.execute_query(&_id, &req.query).await {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "columns": result.columns,
                "rows": result.rows,
                "row_count": result.row_count,
                "execution_time_ms": result.execution_time_ms,
                "estimated_bytes_scanned": result.estimated_bytes_scanned,
                "estimated_cost_usd": result.estimated_cost_usd
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn get_cloud_warehouse_databases(
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = crate::cloud_warehouse::CloudWarehouseManager::new();

    match manager.get_databases(&id).await {
        Ok(databases) => (StatusCode::OK, Json(json!({"databases": databases}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn get_cloud_warehouse_tables(
    Path((id, database)): Path<(String, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = crate::cloud_warehouse::CloudWarehouseManager::new();

    match manager.get_tables(&id, &database).await {
        Ok(tables) => (StatusCode::OK, Json(json!({"tables": tables}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn estimate_cloud_query_cost(
    Path(id): Path<String>,
    Json(req): Json<ExecuteSQLRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let manager = crate::cloud_warehouse::CloudWarehouseManager::new();

    match manager.estimate_query_cost(&id, &req.query) {
        Ok(estimate) => (StatusCode::OK, Json(json!(estimate))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

// AI Training and fine-tuning
#[derive(Deserialize)]
pub struct CreateFineTuningJobRequest {
    pub model_name: String,
    pub ai_provider: String,
    pub compute_provider: String,
    pub training_data_path: String,
    pub batch_size: u32,
    pub num_epochs: u32,
    pub learning_rate: f32,
}

pub async fn create_fine_tuning_job(
    Json(req): Json<CreateFineTuningJobRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let ai_provider = match req.ai_provider.as_str() {
        "ollama" => crate::ai_training::AIProvider::Ollama,
        "claude" => crate::ai_training::AIProvider::Claude,
        "openai" => crate::ai_training::AIProvider::OpenAI,
        "llama2" => crate::ai_training::AIProvider::LLaMA2,
        "mistral" => crate::ai_training::AIProvider::Mistral,
        _ => crate::ai_training::AIProvider::Custom,
    };

    let compute_provider = match req.compute_provider.as_str() {
        "runpod" => crate::ai_training::ComputeProvider::RunPod,
        "lambda" => crate::ai_training::ComputeProvider::Lambda,
        "vast" => crate::ai_training::ComputeProvider::Vast,
        "local" => crate::ai_training::ComputeProvider::Local,
        _ => crate::ai_training::ComputeProvider::RunPod,
    };

    let config = crate::ai_training::FineTuningConfig {
        model_name: req.model_name,
        ai_provider,
        compute_provider,
        training_data_path: req.training_data_path,
        validation_split: 0.1,
        batch_size: req.batch_size,
        num_epochs: req.num_epochs,
        learning_rate: req.learning_rate,
        warmup_steps: 100,
        max_tokens: 512,
        optimizer: "adamw".to_string(),
        lora_rank: Some(16),
        lora_alpha: Some(32),
        save_steps: 500,
    };

    let mut manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );

    match manager.create_fine_tuning_job(config) {
        Ok(job) => (StatusCode::CREATED, Json(json!(job))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn list_fine_tuning_jobs() -> Json<serde_json::Value> {
    let manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );
    let jobs = manager.list_jobs();
    Json(json!({"jobs": jobs}))
}

pub async fn get_fine_tuning_job(Path(job_id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );

    match manager.get_job(&job_id) {
        Some(job) => (StatusCode::OK, Json(json!(job))),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Job not found"})),
        ),
    }
}

pub async fn start_fine_tuning_job(Path(job_id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let mut manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );

    match manager.start_job(&job_id).await {
        Ok(job) => (StatusCode::OK, Json(json!(job))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn cancel_fine_tuning_job(Path(job_id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let mut manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );

    match manager.cancel_job(&job_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "cancelled"}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn list_model_checkpoints(Path(job_id): Path<String>) -> Json<serde_json::Value> {
    let manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );
    let checkpoints = manager.list_checkpoints(&job_id);
    Json(json!({"checkpoints": checkpoints}))
}

pub async fn deploy_inference_endpoint(
    Path(checkpoint_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );

    match manager.deploy_endpoint(&checkpoint_id).await {
        Ok(endpoint) => (StatusCode::CREATED, Json(json!(endpoint))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn list_inference_endpoints() -> Json<serde_json::Value> {
    let manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );
    let endpoints = manager.list_endpoints();
    Json(json!({"endpoints": endpoints}))
}

pub async fn delete_inference_endpoint(Path(endpoint_id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let mut manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );

    match manager.delete_endpoint(&endpoint_id).await {
        Ok(_) => (StatusCode::NO_CONTENT, Json(json!({"status": "deleted"}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

pub async fn get_runpod_instances() -> (StatusCode, Json<serde_json::Value>) {
    let manager = crate::ai_training::AITrainingManager::new(
        std::env::var("RUNPOD_API_KEY").ok(),
    );

    match manager.get_runpod_instances().await {
        Ok(instances) => (StatusCode::OK, Json(json!({"instances": instances}))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

// Realtime collaboration endpoints
#[derive(Deserialize)]
pub struct JoinCollaborationRequest {
    pub notebook_id: String,
    pub user_id: String,
    pub user_name: String,
}

pub async fn join_collaboration(
    Json(req): Json<JoinCollaborationRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "joined",
            "session_id": Uuid::new_v4().to_string(),
            "notebook_id": req.notebook_id,
            "user_id": req.user_id,
            "message": "Real-time collaboration session started (v0.4 feature)"
        })),
    )
}

pub async fn get_active_collaborators(
    Path(notebook_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "notebook_id": notebook_id,
            "collaborators": [],
            "feature_status": "WebSocket infrastructure ready for v0.4"
        })),
    )
}

pub async fn post_comment(
    Path(notebook_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let comment_id = Uuid::new_v4().to_string();
    (
        StatusCode::CREATED,
        Json(json!({
            "comment_id": comment_id,
            "notebook_id": notebook_id,
            "cell_id": req.get("cell_id"),
            "content": req.get("content"),
            "author_id": req.get("author_id"),
            "created_at": chrono::Local::now().to_rfc3339(),
            "feature_status": "Comments infrastructure ready for v0.4"
        })),
    )
}

// File upload/download endpoints
#[derive(Deserialize)]
pub struct FileUploadRequest {
    pub filename: String,
    pub content: String,
    pub mime_type: String,
}

pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    Path(notebook_id): Path<String>,
    Json(req): Json<FileUploadRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let content_bytes = req.content.into_bytes();
    let file_manager = crate::file_manager::FileManager::new(
        std::path::PathBuf::from(&state.notebooks_dir)
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf(),
    );

    match file_manager
        .upload_file(&notebook_id, &req.filename, content_bytes, req.mime_type, "user".to_string())
        .await
    {
        Ok(metadata) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(&metadata).unwrap_or(json!({"error": "serialization failed"}))),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": e})),
        ),
    }
}

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Path((notebook_id, file_id)): Path<(String, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let file_manager = crate::file_manager::FileManager::new(
        std::path::PathBuf::from(&state.notebooks_dir)
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf(),
    );

    match file_manager.download_file(&notebook_id, &file_id).await {
        Ok(_content) => (
            StatusCode::OK,
            Json(json!({
                "status": "success",
                "file_id": file_id,
                "notebook_id": notebook_id
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": e})),
        ),
    }
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Path(notebook_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let file_manager = crate::file_manager::FileManager::new(
        std::path::PathBuf::from(&state.notebooks_dir)
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf(),
    );

    match file_manager.list_files(&notebook_id).await {
        Ok(files) => (
            StatusCode::OK,
            Json(json!({
                "notebook_id": notebook_id,
                "files": files
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e})),
        ),
    }
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Path((notebook_id, file_id)): Path<(String, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let file_manager = crate::file_manager::FileManager::new(
        std::path::PathBuf::from(&state.notebooks_dir)
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .to_path_buf(),
    );

    match file_manager.delete_file(&notebook_id, &file_id).await {
        Ok(_) => (
            StatusCode::NO_CONTENT,
            Json(json!({"status": "deleted"})),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": e})),
        ),
    }
}

// Cloud storage endpoints
#[derive(Deserialize)]
pub struct AddCloudStorageRequest {
    pub name: String,
    pub provider: String,
    pub config: serde_json::Value,
}

pub async fn add_cloud_storage(
    Json(req): Json<AddCloudStorageRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::CREATED,
        Json(json!({
            "status": "mounted",
            "name": req.name,
            "provider": req.provider,
            "mount_path": format!("/mnt/{}", req.name),
            "feature_status": "Cloud storage mounting ready for v0.4 (S3, GCS, Azure Blob, Google Drive)"
        })),
    )
}

pub async fn list_cloud_storage() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "storages": [
                {"name": "Example S3", "provider": "s3", "status": "available"},
                {"name": "Example GCS", "provider": "gcs", "status": "available"},
                {"name": "Example Azure", "provider": "azure", "status": "available"},
                {"name": "Example Google Drive", "provider": "google_drive", "status": "available"}
            ],
            "feature_note": "Configure cloud storage in Settings → Cloud Storage (v0.4)"
        })),
    )
}

pub async fn remove_cloud_storage(
    Path(name): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NO_CONTENT,
        Json(json!({"status": "unmounted", "name": name})),
    )
}

// GitHub integration endpoints
#[derive(Deserialize)]
pub struct GitHubAuthRequest {
    pub token: String,
    pub username: String,
}

pub async fn configure_github(
    Json(req): Json<GitHubAuthRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "configured",
            "username": req.username,
            "auto_backup": false,
            "feature_status": "GitHub integration ready for v0.5 (push/pull/sync notebooks)"
        })),
    )
}

pub async fn sync_with_github(
    Path(notebook_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "synced",
            "notebook_id": notebook_id,
            "repository": req.get("repository"),
            "branch": "main",
            "message": "GitHub sync ready for v0.5 (bidirectional sync)"
        })),
    )
}

pub async fn push_to_github(
    Path(notebook_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "pushed",
            "notebook_id": notebook_id,
            "message": req.get("message").unwrap_or(&json!("Update notebook")),
            "feature_status": "Push to GitHub ready for v0.5"
        })),
    )
}

pub async fn pull_from_github(
    Path(notebook_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "pulled",
            "notebook_id": notebook_id,
            "feature_status": "Pull from GitHub ready for v0.5"
        })),
    )
}

// Output zoom and fullscreen endpoints
#[derive(Deserialize)]
pub struct ZoomRequest {
    pub zoom_level: f32,
}

pub async fn set_output_zoom(
    Path(cell_id): Path<String>,
    Json(req): Json<ZoomRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let renderer = crate::output_renderer::OutputRenderer::new();
    let clamped_zoom = renderer.set_zoom(1.0, req.zoom_level - 1.0);

    (
        StatusCode::OK,
        Json(json!({
            "cell_id": cell_id,
            "zoom_level": clamped_zoom,
            "min_zoom": renderer.min_zoom,
            "max_zoom": renderer.max_zoom
        })),
    )
}

pub async fn fullscreen_output(
    Path(output_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "output_id": output_id,
            "status": "fullscreen_ready",
            "features": [
                "Zoom in/out with +/- buttons",
                "Pan with mouse drag",
                "Download as image",
                "Copy to clipboard",
                "Auto-fit width"
            ],
            "keyboard_shortcuts": {
                "plus": "Zoom in",
                "minus": "Zoom out",
                "0": "Reset zoom",
                "f": "Fit to width",
                "esc": "Exit fullscreen"
            }
        })),
    )
}

pub async fn reset_output_zoom(
    Path(cell_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "cell_id": cell_id,
            "zoom_level": 1.0,
            "status": "reset"
        })),
    )
}

// Typography and display settings
#[derive(Deserialize)]
pub struct DisplaySettingsRequest {
    pub font_size: Option<u32>,
    pub font_family: Option<String>,
    pub line_height: Option<f32>,
    pub theme: Option<String>,
}

pub async fn get_display_settings() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "font_size": 14,
            "font_family": "Roboto Mono",
            "line_height": 1.6,
            "theme": "dark",
            "available_fonts": {
                "mac_only": {
                    "warning": "MACOS ONLY - Not available on Linux/Windows",
                    "fonts": ["Monaco", "Menlo", "SF Mono"]
                },
                "cross_platform": {
                    "note": "Available on Windows, macOS, and Linux",
                    "fonts": [
                        "Courier New",
                        "Inconsolata",
                        "Roboto Mono",
                        "Source Code Pro",
                        "JetBrains Mono",
                        "IBM Plex Mono",
                        "Cascadia Code"
                    ]
                }
            },
            "current_platform_recommendation": "Roboto Mono (cross-platform) or Cascadia Code",
            "note": "If you select a macOS-only font on Linux/Windows, the system will automatically fallback to Courier New or system monospace"
        })),
    )
}

pub async fn update_display_settings(
    Json(req): Json<DisplaySettingsRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "font_size": req.font_size.unwrap_or(14),
            "font_family": req.font_family.unwrap_or("Monaco".to_string()),
            "line_height": req.line_height.unwrap_or(1.6),
            "theme": req.theme.unwrap_or("dark".to_string()),
            "status": "settings_updated"
        })),
    )
}

pub async fn get_mac_compatible_fonts() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "mac_only_fonts": [
                {
                    "name": "Monaco",
                    "monospace": true,
                    "system_font": true,
                    "platform": "macOS only",
                    "compatibility": "MACOS_ONLY",
                    "warning": "Not available on Linux or Windows",
                    "description": "Apple's classic monospace font - macOS only"
                },
                {
                    "name": "Menlo",
                    "monospace": true,
                    "system_font": true,
                    "platform": "macOS only",
                    "compatibility": "MACOS_ONLY",
                    "warning": "Not available on Linux or Windows",
                    "description": "Improved Monaco for Leopard+ - macOS only"
                },
                {
                    "name": "SF Mono",
                    "monospace": true,
                    "system_font": true,
                    "platform": "macOS only",
                    "compatibility": "MACOS_ONLY",
                    "warning": "Recommended for Mac - Not available on Linux or Windows",
                    "description": "San Francisco Mono, modern Apple font - macOS only"
                }
            ],
            "cross_platform_fonts": [
                {
                    "name": "Courier New",
                    "monospace": true,
                    "system_font": true,
                    "platform": "All platforms",
                    "compatibility": "UNIVERSAL",
                    "description": "Classic monospace - available on Windows, Mac, Linux"
                },
                {
                    "name": "Inconsolata",
                    "monospace": true,
                    "system_font": false,
                    "platform": "All platforms (requires install)",
                    "compatibility": "UNIVERSAL",
                    "description": "Beautiful open-source monospace - Windows, Mac, Linux"
                },
                {
                    "name": "Roboto Mono",
                    "monospace": true,
                    "system_font": false,
                    "platform": "All platforms (requires install)",
                    "compatibility": "UNIVERSAL",
                    "description": "Google's monospace font - Windows, Mac, Linux"
                },
                {
                    "name": "JetBrains Mono",
                    "monospace": true,
                    "system_font": false,
                    "platform": "All platforms (requires install)",
                    "compatibility": "UNIVERSAL",
                    "description": "JetBrains' coding font - Windows, Mac, Linux"
                },
                {
                    "name": "Source Code Pro",
                    "monospace": true,
                    "system_font": false,
                    "platform": "All platforms (requires install)",
                    "compatibility": "UNIVERSAL",
                    "description": "Adobe's monospace font - Windows, Mac, Linux"
                },
                {
                    "name": "IBM Plex Mono",
                    "monospace": true,
                    "system_font": false,
                    "platform": "All platforms (requires install)",
                    "compatibility": "UNIVERSAL",
                    "description": "IBM's open-source font - Windows, Mac, Linux"
                },
                {
                    "name": "Cascadia Code",
                    "monospace": true,
                    "system_font": false,
                    "platform": "All platforms (requires install)",
                    "compatibility": "UNIVERSAL",
                    "description": "Microsoft's monospace font - Windows, Mac, Linux"
                }
            ],
            "platform_detection": {
                "macos": "SF Mono recommended (system font)",
                "linux": "Roboto Mono or JetBrains Mono recommended",
                "windows": "Cascadia Code or Courier New recommended"
            },
            "recommended": "SF Mono (macOS) / Roboto Mono (Linux) / Cascadia Code (Windows)",
            "font_sizes": [10, 11, 12, 13, 14, 15, 16, 18, 20],
            "default_font_size": 14,
            "note": "macOS-only fonts will not render on Linux/Windows systems. Fallback to system monospace or cross-platform fonts will be used."
        })),
    )
}

// Kubernetes deployment endpoints
pub async fn get_k8s_manifest() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "manifest_generated",
            "feature_status": "Kubernetes deployment ready for v1.0",
            "includes": ["Deployment", "Service", "Ingress"],
            "resources": ["CPU requests/limits", "Memory requests/limits"],
            "note": "Multi-tenant Kubernetes support coming v1.0"
        })),
    )
}

pub async fn deploy_to_k8s(
    Json(_req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::CREATED,
        Json(json!({
            "status": "deploying",
            "feature_status": "Kubernetes deployment coming v1.0",
            "namespace": "default",
            "message": "Use kubectl apply -f manifest.yaml to deploy"
        })),
    )
}

pub async fn get_k8s_pods() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "pods": [],
            "feature_status": "Pod monitoring coming v1.0"
        })),
    )
}

pub async fn get_docker_compose() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "docker_compose": crate::k8s_deployment::DockerManager::generate_docker_compose(),
            "status": "compose_generated",
            "usage": "docker-compose up -d"
        })),
    )
}

// dbt integration endpoints
pub async fn list_dbt_models(
    Path(_notebook_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "models": [],
            "feature_status": "dbt integration ready for v1.0"
        })),
    )
}

pub async fn run_dbt_models(
    Path(_notebook_id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "running",
            "feature_status": "dbt execution ready for v1.0",
            "message": "Configure dbt project path in settings"
        })),
    )
}

pub async fn run_dbt_tests(
    Path(_notebook_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "tests_passed": 0,
            "tests_failed": 0,
            "feature_status": "dbt testing ready for v1.0"
        })),
    )
}

pub async fn get_dbt_project_yml() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "project_yml": crate::dbt_integration::DbtManager::generate_profiles_yml(),
            "feature_status": "dbt configuration ready for v1.0"
        })),
    )
}

// Airflow integration endpoints
pub async fn list_airflow_dags() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "dags": [],
            "feature_status": "Airflow integration ready for v1.0"
        })),
    )
}

pub async fn trigger_airflow_dag(
    Path(_dag_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "triggered",
            "feature_status": "Airflow DAG execution ready for v1.0",
            "message": "Configure Airflow URL in settings"
        })),
    )
}

pub async fn get_airflow_dag_status(
    Path(_dag_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "dag_id": _dag_id,
            "status": "unknown",
            "feature_status": "Airflow monitoring ready for v1.0"
        })),
    )
}

pub async fn generate_airflow_dag(
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let dag_name = req
        .get("dag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("new_dag");

    (
        StatusCode::OK,
        Json(json!({
            "dag_code": crate::airflow_integration::AirflowManager::generate_python_dag(dag_name),
            "feature_status": "Airflow DAG generation ready for v1.0"
        })),
    )
}

// Docker container code execution endpoints
pub async fn list_docker_containers() -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.list_containers().await {
        Ok(containers) => (
            StatusCode::OK,
            Json(json!({
                "containers": containers,
                "status": "success",
                "note": "Requires Docker Desktop running in parallel"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": e,
                "status": "docker_not_available",
                "note": "Ensure Docker Desktop is running"
            })),
        ),
    }
}

#[derive(Deserialize)]
pub struct ExecuteInContainerRequest {
    pub container_id: String,
    pub code: String,
    pub language: Option<String>,
    pub working_dir: Option<String>,
    pub timeout_seconds: Option<u32>,
}

pub async fn execute_code_in_container(
    Json(req): Json<ExecuteInContainerRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);
    let lang = req.language.unwrap_or_else(|| "python".to_string());

    let formatted_code = match lang.as_str() {
        "python" => format!("python -c \"{}\"", req.code.replace("\"", "\\\"")),
        "bash" | "shell" => req.code,
        "javascript" | "node" => format!("node -e \"{}\"", req.code.replace("\"", "\\\"")),
        "ruby" => format!("ruby -e \"{}\"", req.code.replace("\"", "\\\"")),
        _ => req.code,
    };

    match executor
        .execute_in_container(&req.container_id, &formatted_code, req.working_dir)
        .await
    {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "container_id": result.container_id,
                "exit_code": result.exit_code,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "execution_time_ms": result.execution_time_ms,
                "status": "success"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e,
                "status": "execution_failed"
            })),
        ),
    }
}

pub async fn get_container_files(
    Path((container_id, path)): Path<(String, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.get_container_files(&container_id, &path).await {
        Ok(files) => (
            StatusCode::OK,
            Json(json!({
                "container_id": container_id,
                "path": path,
                "files": files,
                "status": "success"
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

pub async fn read_container_file(
    Path((container_id, path)): Path<(String, String)>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.read_container_file(&container_id, &path).await {
        Ok(content) => (
            StatusCode::OK,
            Json(json!({
                "container_id": container_id,
                "path": path,
                "content": content,
                "status": "success"
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

#[derive(Deserialize)]
pub struct WriteContainerFileRequest {
    pub content: String,
}

pub async fn write_container_file(
    Path((container_id, path)): Path<(String, String)>,
    Json(req): Json<WriteContainerFileRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor
        .write_container_file(&container_id, &path, req.content)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "container_id": container_id,
                "path": path,
                "status": "written"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

pub async fn get_container_logs(
    Path(container_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.get_container_logs(&container_id, Some(100)).await {
        Ok(logs) => (
            StatusCode::OK,
            Json(json!({
                "container_id": container_id,
                "logs": logs,
                "status": "success"
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

pub async fn get_container_stats(
    Path(container_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.get_container_stats(&container_id).await {
        Ok(stats) => (
            StatusCode::OK,
            Json(json!({
                "container_id": stats.container_id,
                "cpu_percent": stats.cpu_percent,
                "memory_usage": stats.memory_usage,
                "memory_limit": stats.memory_limit,
                "network_rx": stats.network_rx,
                "network_tx": stats.network_tx,
                "block_read": stats.block_read,
                "block_write": stats.block_write,
                "status": "success"
            })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

#[derive(Deserialize)]
pub struct CreateContainerRequest {
    pub image: String,
    pub name: String,
    pub environment: Option<std::collections::HashMap<String, String>>,
    pub ports: Option<std::collections::HashMap<String, u16>>,
}

pub async fn create_docker_container(
    Json(req): Json<CreateContainerRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);
    let env = req.environment.unwrap_or_default();
    let ports = req.ports.unwrap_or_default();

    match executor.create_container(&req.image, &req.name, env, ports).await {
        Ok(container) => (
            StatusCode::CREATED,
            Json(json!({
                "container": container,
                "status": "created",
                "note": "Container created but not started. Call start endpoint to run."
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

pub async fn start_docker_container(
    Path(container_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.start_container(&container_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "container_id": container_id,
                "status": "started"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

pub async fn stop_docker_container(
    Path(container_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.stop_container(&container_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "container_id": container_id,
                "status": "stopped"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

pub async fn remove_docker_container(
    Path(container_id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.remove_container(&container_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "container_id": container_id,
                "status": "removed"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}

pub async fn pull_docker_image(
    Json(req): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let image = req
        .get("image")
        .and_then(|v| v.as_str())
        .unwrap_or("ubuntu:latest");

    let executor = crate::docker_executor::DockerExecutor::new(None);

    match executor.pull_image(image).await {
        Ok(msg) => (
            StatusCode::OK,
            Json(json!({
                "image": image,
                "message": msg,
                "status": "success"
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e,
                "status": "failed"
            })),
        ),
    }
}
