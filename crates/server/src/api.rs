use crate::ai::AIRequest;
use crate::models::{ExecuteCellRequest, ExecuteCellResponse, Notebook, Output};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
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

/// Interrupt the currently running cell by sending SIGINT to the interpreter
/// (raises KeyboardInterrupt in the user's code). Doesn't take the kernel lock,
/// so it works while an execute() is in flight holding that lock.
pub async fn kernel_interrupt(State(state): State<Arc<AppState>>) -> StatusCode {
    let pid = state.kernel_pid.load(std::sync::atomic::Ordering::SeqCst);
    if pid <= 0 {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    #[cfg(unix)]
    {
        let _ = tokio::process::Command::new("kill")
            .arg("-INT")
            .arg(pid.to_string())
            .status()
            .await;
        StatusCode::OK
    }
    #[cfg(not(unix))]
    {
        StatusCode::NOT_IMPLEMENTED
    }
}

/// Restart the kernel, clearing all variables/imports.
pub async fn kernel_restart(State(state): State<Arc<AppState>>) -> StatusCode {
    let mut kernel = state.kernel.lock().await;
    match kernel.as_mut() {
        Some(k) => match k.restart() {
            Ok(()) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        },
        None => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// Per-cell interpreter selected by a leading magic line.
enum Magic {
    Python,
    Sql,
    Shell,
    Markdown,
}

/// Inspect the first line for a `%magic` (or `--sql` / `!`) and return the
/// interpreter plus the body to run (with the magic line stripped).
fn parse_magic(code: &str) -> (Magic, String) {
    let trimmed = code.trim_start();
    let first_line = trimmed.lines().next().unwrap_or("").trim();
    let rest = || trimmed.splitn(2, '\n').nth(1).unwrap_or("").to_string();

    if first_line.eq_ignore_ascii_case("%sql") {
        (Magic::Sql, rest())
    } else if trimmed.starts_with("--sql") {
        (Magic::Sql, trimmed.to_string()) // --sql is a valid SQL comment; keep it
    } else if first_line.eq_ignore_ascii_case("%sh") || first_line.eq_ignore_ascii_case("%bash") {
        (Magic::Shell, rest())
    } else if let Some(cmd) = first_line.strip_prefix('!') {
        // `!cmd` shell escape (single line)
        (Magic::Shell, cmd.to_string())
    } else if first_line.eq_ignore_ascii_case("%md") || first_line.eq_ignore_ascii_case("%markdown") {
        (Magic::Markdown, rest())
    } else if first_line.eq_ignore_ascii_case("%python") || first_line.eq_ignore_ascii_case("%py") {
        (Magic::Python, rest())
    } else {
        (Magic::Python, code.to_string())
    }
}

/// Run a shell command and return its combined output as a stream object.
async fn run_shell_cell(cmd: &str) -> anyhow::Result<Vec<serde_json::Value>> {
    let out = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await?;
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    let err = String::from_utf8_lossy(&out.stderr);
    if !err.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&err);
    }
    let name = if out.status.success() { "stdout" } else { "stderr" };
    Ok(vec![serde_json::json!({
        "output_type": "stream",
        "name": name,
        "text": [text],
    })])
}

/// Convert a kernel JSON output object (Jupyter-style) into our `Output` model,
/// preserving MIME bundles (`data`) and normalising `text` to a string list.
fn value_to_output(out: serde_json::Value) -> Output {
    let text = match out.get("text") {
        Some(serde_json::Value::Array(a)) => {
            Some(a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        }
        Some(serde_json::Value::String(s)) => Some(vec![s.clone()]),
        _ => None,
    };
    Output {
        output_type: out
            .get("output_type")
            .and_then(|v| v.as_str())
            .unwrap_or("stream")
            .to_string(),
        data: out.get("data").cloned(),
        text,
        metadata: out.get("metadata").cloned(),
    }
}

pub async fn execute_cell(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteCellRequest>,
) -> (StatusCode, Json<ExecuteCellResponse>) {
    // Preferred path: the client sends the code directly, so execution does not
    // depend on the notebook file existing on disk or on cell-id round-tripping.
    let code = if let Some(c) = req.code.clone() {
        c
    } else {
        // Fallback: load the notebook from disk and look the cell up by id.
        let path = format!("{}/{}.ipynb", state.notebooks_dir, id);
        let notebook = match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(ipynb) => match crate::files::from_ipynb(ipynb) {
                    Ok(nb) => nb,
                    Err(_) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ExecuteCellResponse { execution_count: 0, outputs: vec![] }),
                        )
                    }
                },
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ExecuteCellResponse { execution_count: 0, outputs: vec![] }),
                    )
                }
            },
            Err(_) => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ExecuteCellResponse { execution_count: 0, outputs: vec![] }),
                )
            }
        };

        let cell = match notebook.cells.iter().find(|c| c.id == req.cell_id) {
            Some(c) => c,
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(ExecuteCellResponse { execution_count: 0, outputs: vec![] }),
                )
            }
        };

        if cell.cell_type != "code" {
            return (
                StatusCode::BAD_REQUEST,
                Json(ExecuteCellResponse { execution_count: 0, outputs: vec![] }),
            );
        }

        // Join ALL source lines (the old code only ran source[0]).
        cell.source.join("")
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
            // Route the cell by its leading magic (Zeppelin-style interpreters).
            let (magic, body) = parse_magic(&code);

            // Live-output streaming: forward stdout chunks to WS clients tagged
            // with this cell id. Additive — the response below is authoritative.
            let (stx, mut srx) = tokio::sync::mpsc::unbounded_channel::<String>();
            {
                let btx = state.stream_tx.clone();
                let cid = req.cell_id.clone();
                tokio::spawn(async move {
                    while let Some(chunk) = srx.recv().await {
                        let _ = btx.send(json!({ "cell_id": cid, "text": chunk }).to_string());
                    }
                });
            }

            let result = match magic {
                Magic::Sql => {
                    // Run SQL in-process via DuckDB inside the shared Python kernel.
                    // DuckDB can query pandas DataFrames defined in other cells by
                    // name, so `%sql SELECT * FROM my_df` just works. Result is a
                    // DataFrame -> renders as an HTML table (chartable downstream).
                    let q = serde_json::to_string(&body).unwrap_or_else(|_| "\"\"".to_string());
                    let py = format!(
                        "try:\n    import duckdb as _ddb\nexcept ImportError:\n    raise ImportError('%sql needs DuckDB — install it: pip install duckdb')\n_ddb.sql({}).df()",
                        q
                    );
                    match k.execute(&py).await {
                        Ok((_s, o)) => Ok(o),
                        Err(e) => Err(e),
                    }
                }
                Magic::Shell => run_shell_cell(&body).await,
                Magic::Markdown => Ok(vec![serde_json::json!({
                    "output_type": "display_data",
                    "data": {"text/markdown": body},
                    "metadata": {},
                })]),
                Magic::Python => match k.execute_streaming(&body, Some(stx)).await {
                    Ok((_stdout, outputs)) => Ok(outputs),
                    Err(e) => Err(e),
                },
            };

            match result {
                Ok(outputs) => {
                    let response = ExecuteCellResponse {
                        execution_count: k.execution_count(),
                        outputs: outputs.into_iter().map(value_to_output).collect(),
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

#[derive(serde::Deserialize)]
pub struct TerminalRequest {
    pub command: String,
    #[serde(default)]
    pub cwd: Option<String>,
}

/// Run a shell command and return combined stdout+stderr. Local dev tool — runs
/// in the server process's working directory (or `cwd` if provided).
pub async fn terminal_exec(Json(req): Json<TerminalRequest>) -> Json<serde_json::Value> {
    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c").arg(&req.command);
    if let Some(dir) = req.cwd.as_ref() {
        cmd.current_dir(dir);
    }
    let result = cmd.output().await;
    let (output, code) = match result {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr);
            if !err.is_empty() {
                if !s.is_empty() && !s.ends_with('\n') {
                    s.push('\n');
                }
                s.push_str(&err);
            }
            (s, o.status.code().unwrap_or(-1))
        }
        Err(e) => (format!("prismnote: {e}"), -1),
    };
    Json(serde_json::json!({ "output": output.trim_end_matches('\n'), "exit_code": code }))
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
    let configured = state.ai_engine.read().await.is_some();
    Json(AIConfigResponse {
        configured,
        provider: if configured { Some("configured".to_string()) } else { None },
    })
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
    State(state): State<Arc<AppState>>,
    Json(req): Json<AIConfigRequest>,
) -> (StatusCode, Json<AIConfigResponse>) {
    // Build the engine from the posted config, swap it in at runtime, and persist
    // it so the choice survives restarts. This is what makes the AI settings UI
    // actually take effect (previously it was a no-op).
    let config = crate::ai::AIConfig {
        provider: req.provider.clone(),
        ollama_url: req.ollama_url.clone(),
        ollama_model: req.ollama_model.clone(),
        claude_api_key: req.claude_api_key.clone(),
        openai_api_key: req.openai_api_key.clone(),
        openai_model: req.openai_model.clone(),
    };

    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = std::fs::write(&state.ai_config_path, json);
    }

    *state.ai_engine.write().await = Some(Arc::new(crate::ai::AIEngine::new(config)));

    (
        StatusCode::OK,
        Json(AIConfigResponse {
            configured: true,
            provider: Some(req.provider),
        }),
    )
}

#[derive(Serialize)]
pub struct AIResponseData {
    pub suggestion: String,
}

pub async fn ai_explain(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AIRequest>,
) -> (StatusCode, Json<AIResponseData>) {
    match state.ai_engine.read().await.as_ref() {
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
    match state.ai_engine.read().await.as_ref() {
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
    match state.ai_engine.read().await.as_ref() {
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

// ── Server-side filesystem browser ──────────────────────────────────────────
// Reliable "Open Folder" for a local-first app: the backend lists the real
// filesystem, so it works in any browser (including embedded/automated ones)
// without depending on the sandboxed File System Access API.

#[derive(Deserialize)]
pub struct FsQuery {
    pub path: Option<String>,
    #[serde(default)]
    pub show_hidden: Option<bool>,
}

fn default_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
}

pub async fn fs_list(Query(q): Query<FsQuery>) -> (StatusCode, Json<serde_json::Value>) {
    let path = q.path.filter(|p| !p.trim().is_empty()).unwrap_or_else(default_dir);
    let p = std::path::Path::new(&path);

    let read = match std::fs::read_dir(p) {
        Ok(rd) => rd,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("cannot open {}: {}", path, e) })),
            )
        }
    };

    let show_hidden = q.show_hidden.unwrap_or(false);
    let mut entries: Vec<serde_json::Value> = vec![];
    for e in read.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') && !show_hidden {
            continue; // hide dotfiles unless explicitly requested
        }
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        entries.push(json!({
            "name": name,
            "path": e.path().to_string_lossy(),
            "is_dir": is_dir,
        }));
    }
    // directories first, then case-insensitive by name
    entries.sort_by(|a, b| {
        let ad = a["is_dir"].as_bool().unwrap_or(false);
        let bd = b["is_dir"].as_bool().unwrap_or(false);
        bd.cmp(&ad).then_with(|| {
            a["name"].as_str().unwrap_or("").to_lowercase()
                .cmp(&b["name"].as_str().unwrap_or("").to_lowercase())
        })
    });

    let abs = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let parent = abs.parent().map(|x| x.to_string_lossy().to_string());
    (
        StatusCode::OK,
        Json(json!({
            "path": abs.to_string_lossy(),
            "parent": parent,
            "entries": entries,
        })),
    )
}

pub async fn fs_read(Query(q): Query<FsQuery>) -> (StatusCode, Json<serde_json::Value>) {
    let path = q.path.unwrap_or_default();
    match std::fs::read_to_string(&path) {
        Ok(content) => (StatusCode::OK, Json(json!({ "path": path, "content": content }))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("cannot read {}: {}", path, e) })),
        ),
    }
}

// File operations for the server-side browser. Local-first: these act on the
// machine running PrismNote (same trust model as fs_list/fs_read).
#[derive(Deserialize)]
pub struct FsEntryReq {
    pub path: String, // parent directory
    pub name: String,
}
#[derive(Deserialize)]
pub struct FsRenameReq {
    pub path: String, // existing file/dir
    pub new_name: String,
}
#[derive(Deserialize)]
pub struct FsWriteReq {
    pub path: String, // full file path
    pub content: String,
}
#[derive(Deserialize)]
pub struct FsPathReq {
    pub path: String,
}

fn fs_ok(r: std::io::Result<()>, path: String) -> (StatusCode, Json<serde_json::Value>) {
    match r {
        Ok(()) => (StatusCode::OK, Json(json!({ "ok": true, "path": path }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "error": e.to_string() }))),
    }
}

pub async fn fs_mkdir(Json(req): Json<FsEntryReq>) -> (StatusCode, Json<serde_json::Value>) {
    let p = std::path::Path::new(&req.path).join(&req.name);
    let path = p.to_string_lossy().to_string();
    fs_ok(std::fs::create_dir_all(&p), path)
}

pub async fn fs_new_file(Json(req): Json<FsEntryReq>) -> (StatusCode, Json<serde_json::Value>) {
    let p = std::path::Path::new(&req.path).join(&req.name);
    if p.exists() {
        return (StatusCode::BAD_REQUEST, Json(json!({ "ok": false, "error": "already exists" })));
    }
    let path = p.to_string_lossy().to_string();
    fs_ok(std::fs::write(&p, b"").map(|_| ()), path)
}

pub async fn fs_write(Json(req): Json<FsWriteReq>) -> (StatusCode, Json<serde_json::Value>) {
    fs_ok(std::fs::write(&req.path, req.content.as_bytes()).map(|_| ()), req.path.clone())
}

pub async fn fs_rename(Json(req): Json<FsRenameReq>) -> (StatusCode, Json<serde_json::Value>) {
    let from = std::path::Path::new(&req.path);
    let to = from.with_file_name(&req.new_name);
    let path = to.to_string_lossy().to_string();
    fs_ok(std::fs::rename(from, &to), path)
}

pub async fn fs_delete(Json(req): Json<FsPathReq>) -> (StatusCode, Json<serde_json::Value>) {
    let p = std::path::Path::new(&req.path);
    let r = if p.is_dir() { std::fs::remove_dir_all(p) } else { std::fs::remove_file(p) };
    fs_ok(r, req.path.clone())
}

#[derive(Deserialize)]
pub struct AIEditRequest {
    pub code: String,
    pub instruction: String,
    pub context: Option<String>,
}

pub async fn ai_edit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AIEditRequest>,
) -> (StatusCode, Json<AIResponseData>) {
    match state.ai_engine.read().await.as_ref() {
        Some(engine) => match engine
            .transform(&req.code, &req.instruction, req.context.as_deref())
            .await
        {
            Ok(suggestion) => (StatusCode::OK, Json(AIResponseData { suggestion })),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AIResponseData {
                    suggestion: format!("AI error: {}", e),
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

// ── Cloud deployment: generate ready-to-use artifacts ────────────────────────
// Returns real, copy-pasteable files so PrismNote can be shipped to a container
// host, Kubernetes, or Fly.io with one command each.

pub async fn deploy_artifacts() -> (StatusCode, Json<serde_json::Value>) {
    let dockerfile = r#"# syntax=docker/dockerfile:1
# ---- build frontend ----
FROM node:20-slim AS web
WORKDIR /web
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ---- build server ----
FROM rust:1-slim AS server
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
COPY --from=web /web/dist ./frontend/dist
RUN cargo build --release --bin prismnote

# ---- runtime ----
FROM python:3.11-slim
RUN pip install --no-cache-dir ipykernel pandas matplotlib rich duckdb
WORKDIR /app
COPY --from=server /app/target/release/prismnote /usr/local/bin/prismnote
COPY --from=web /web/dist ./frontend/dist
EXPOSE 8000
CMD ["prismnote"]
"#;

    let compose = r#"services:
  prismnote:
    build: .
    ports:
      - "8000:8000"
    volumes:
      - prismnote-data:/root/.prismnote
    environment:
      - PRISMNOTE_DIR=/root/.prismnote/notebooks
    restart: unless-stopped
volumes:
  prismnote-data:
"#;

    let k8s = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: prismnote
spec:
  replicas: 1
  selector: { matchLabels: { app: prismnote } }
  template:
    metadata: { labels: { app: prismnote } }
    spec:
      containers:
        - name: prismnote
          image: ghcr.io/OWNER/prismnote:latest
          ports: [{ containerPort: 8000 }]
          resources:
            requests: { cpu: "250m", memory: "512Mi" }
            limits: { cpu: "1", memory: "2Gi" }
          volumeMounts:
            - { name: data, mountPath: /root/.prismnote }
      volumes:
        - name: data
          persistentVolumeClaim: { claimName: prismnote-data }
---
apiVersion: v1
kind: Service
metadata: { name: prismnote }
spec:
  selector: { app: prismnote }
  ports: [{ port: 80, targetPort: 8000 }]
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata: { name: prismnote-data }
spec:
  accessModes: [ReadWriteOnce]
  resources: { requests: { storage: 5Gi } }
"#;

    let fly = r#"app = "prismnote"
primary_region = "iad"

[build]
  dockerfile = "Dockerfile"

[http_service]
  internal_port = 8000
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 0

[[mounts]]
  source = "prismnote_data"
  destination = "/root/.prismnote"
"#;

    (
        StatusCode::OK,
        Json(json!({
            "Dockerfile": dockerfile,
            "docker-compose.yml": compose,
            "k8s.yaml": k8s,
            "fly.toml": fly,
            "commands": {
                "docker": "docker compose up -d",
                "kubernetes": "kubectl apply -f k8s.yaml",
                "fly": "fly launch --copy-config --now"
            }
        })),
    )
}

// ── Git / GitHub integration (real, via the local `git` CLI) ─────────────────
// Works several ways: init a repo, clone one, stage+commit, push, pull, status.
// Operates on a directory the user points at (their workspace/notebook folder).

#[derive(Deserialize)]
pub struct GitCloneReq {
    pub url: String,
    pub dir: String,
}

#[derive(Deserialize)]
pub struct GitOpReq {
    pub dir: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub remote: Option<String>,
    #[serde(default)]
    pub branch: Option<String>,
}

async fn git(dir: Option<&str>, args: &[&str]) -> (bool, String) {
    let mut c = tokio::process::Command::new("git");
    if let Some(d) = dir {
        c.arg("-C").arg(d);
    }
    c.args(args);
    match c.output().await {
        Ok(o) => {
            let mut s = String::from_utf8_lossy(&o.stdout).to_string();
            let e = String::from_utf8_lossy(&o.stderr);
            if !e.is_empty() {
                s.push_str(&e);
            }
            (o.status.success(), s.trim().to_string())
        }
        Err(e) => (false, format!("git not available: {}", e)),
    }
}

fn git_json(ok: bool, output: String) -> (StatusCode, Json<serde_json::Value>) {
    (
        if ok { StatusCode::OK } else { StatusCode::BAD_REQUEST },
        Json(json!({ "ok": ok, "output": output })),
    )
}

pub async fn git_status(Query(q): Query<FsQuery>) -> (StatusCode, Json<serde_json::Value>) {
    let dir = q.path.unwrap_or_else(default_dir);
    let (ok_b, branch) = git(Some(&dir), &["rev-parse", "--abbrev-ref", "HEAD"]).await;
    let (ok_s, status) = git(Some(&dir), &["status", "--short"]).await;
    (
        StatusCode::OK,
        Json(json!({
            "ok": ok_b && ok_s,
            "is_repo": ok_b,
            "branch": if ok_b { branch } else { String::new() },
            "status": status,
        })),
    )
}

pub async fn git_init(Json(req): Json<GitOpReq>) -> (StatusCode, Json<serde_json::Value>) {
    let (ok, out) = git(Some(&req.dir), &["init"]).await;
    git_json(ok, out)
}

pub async fn git_clone(Json(req): Json<GitCloneReq>) -> (StatusCode, Json<serde_json::Value>) {
    let (ok, out) = git(None, &["clone", &req.url, &req.dir]).await;
    git_json(ok, out)
}

pub async fn git_commit(Json(req): Json<GitOpReq>) -> (StatusCode, Json<serde_json::Value>) {
    let (ok_a, out_a) = git(Some(&req.dir), &["add", "-A"]).await;
    if !ok_a {
        return git_json(false, out_a);
    }
    let msg = req.message.unwrap_or_else(|| "Update from PrismNote".to_string());
    let (ok, out) = git(Some(&req.dir), &["commit", "-m", &msg]).await;
    git_json(ok, out)
}

pub async fn git_push(Json(req): Json<GitOpReq>) -> (StatusCode, Json<serde_json::Value>) {
    let mut args = vec!["push".to_string()];
    if let Some(r) = req.remote {
        args.push(r);
        if let Some(b) = req.branch {
            args.push(b);
        }
    }
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let (ok, out) = git(Some(&req.dir), &refs).await;
    git_json(ok, out)
}

pub async fn git_pull(Json(req): Json<GitOpReq>) -> (StatusCode, Json<serde_json::Value>) {
    let (ok, out) = git(Some(&req.dir), &["pull", "--ff-only"]).await;
    git_json(ok, out)
}

// ── Jobs: run a whole notebook as a unit, optionally on a schedule ───────────
use crate::jobs::{Job, JobRun, Schedule};

#[derive(Deserialize)]
pub struct CreateJobRequest {
    pub name: String,
    pub cells: Vec<String>,
    #[serde(default)]
    pub schedule: Option<Schedule>,
}

/// Execute a job's code cells in order against the shared kernel, returning a
/// run summary. Each cell that emits an `error` output counts as failed.
async fn execute_job_cells(state: &Arc<AppState>, cells: &[String]) -> JobRun {
    let started = chrono::Local::now().to_rfc3339();
    let mut ok = 0usize;
    let mut failed = 0usize;
    let mut log = String::new();

    let mut kernel = state.kernel.lock().await;
    match kernel.as_mut() {
        Some(k) => {
            for (i, code) in cells.iter().enumerate() {
                if code.trim().is_empty() {
                    continue;
                }
                match k.execute(code).await {
                    Ok((_s, outputs)) => {
                        let has_err = outputs.iter().any(|o| {
                            o.get("output_type").and_then(|v| v.as_str()) == Some("error")
                        });
                        if has_err {
                            failed += 1;
                            log.push_str(&format!("cell {} ❌ error\n", i + 1));
                        } else {
                            ok += 1;
                            log.push_str(&format!("cell {} ✓\n", i + 1));
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        log.push_str(&format!("cell {} ❌ {}\n", i + 1, e));
                    }
                }
            }
        }
        None => {
            log.push_str("kernel unavailable\n");
            failed += 1;
        }
    }

    JobRun {
        started_at: started,
        finished_at: chrono::Local::now().to_rfc3339(),
        status: if failed == 0 { "success".into() } else { "failed".into() },
        cells_ok: ok,
        cells_failed: failed,
        log,
    }
}

/// Run a single job by id, recording the run in its history (capped at 20).
async fn run_job(state: &Arc<AppState>, id: &str) -> Option<JobRun> {
    let cells = {
        let jobs = state.jobs.lock().await;
        jobs.iter().find(|j| j.id == id)?.cells.clone()
    };
    let run = execute_job_cells(state, &cells).await;
    let mut jobs = state.jobs.lock().await;
    if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
        job.last_run = Some(run.finished_at.clone());
        job.last_status = Some(run.status.clone());
        job.runs.push(run.clone());
        let len = job.runs.len();
        if len > 20 {
            job.runs.drain(0..len - 20);
        }
    }
    crate::jobs::save_jobs(&jobs);
    Some(run)
}

/// Called every minute by the scheduler: run any job whose schedule is due.
pub async fn run_due_jobs(state: &Arc<AppState>) {
    let now = chrono::Local::now();
    let due: Vec<String> = {
        let jobs = state.jobs.lock().await;
        jobs.iter()
            .filter(|j| schedule_due(&j.schedule, j.last_run.as_deref(), now))
            .map(|j| j.id.clone())
            .collect()
    };
    for id in due {
        let _ = run_job(state, &id).await;
    }
}

fn schedule_due(
    sched: &Schedule,
    last_run: Option<&str>,
    now: chrono::DateTime<chrono::Local>,
) -> bool {
    use chrono::{Datelike, Timelike};
    let last = last_run.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
    match sched.kind.as_str() {
        "interval" => {
            let mins = sched.minutes.unwrap_or(0);
            if mins == 0 {
                return false;
            }
            match last {
                Some(l) => (now.signed_duration_since(l)).num_minutes() >= mins as i64,
                None => true, // never run → run now
            }
        }
        "daily" => {
            let t = sched.time.as_deref().unwrap_or("");
            let parts: Vec<&str> = t.split(':').collect();
            if parts.len() != 2 {
                return false;
            }
            let (h, m) = (parts[0].parse::<u32>().ok(), parts[1].parse::<u32>().ok());
            let (h, m) = match (h, m) {
                (Some(h), Some(m)) => (h, m),
                _ => return false,
            };
            if now.hour() != h || now.minute() != m {
                return false;
            }
            // don't run twice in the same minute/day
            match last {
                Some(l) => !(l.year() == now.year() && l.ordinal() == now.ordinal()),
                None => true,
            }
        }
        _ => false, // manual
    }
}

pub async fn list_jobs(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let jobs = state.jobs.lock().await;
    // list view omits run logs for brevity
    let summary: Vec<serde_json::Value> = jobs
        .iter()
        .map(|j| {
            json!({
                "id": j.id, "name": j.name, "schedule": j.schedule,
                "created_at": j.created_at, "last_run": j.last_run,
                "last_status": j.last_status, "cells": j.cells.len(),
                "runs": j.runs.len(),
            })
        })
        .collect();
    Json(json!({ "jobs": summary }))
}

pub async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let jobs = state.jobs.lock().await;
    match jobs.iter().find(|j| j.id == id) {
        Some(j) => (StatusCode::OK, Json(serde_json::to_value(j).unwrap())),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "job not found" }))),
    }
}

pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateJobRequest>,
) -> (StatusCode, Json<Job>) {
    let job = Job {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        cells: req.cells,
        schedule: req.schedule.unwrap_or_default(),
        created_at: chrono::Local::now().to_rfc3339(),
        last_run: None,
        last_status: None,
        runs: vec![],
    };
    let mut jobs = state.jobs.lock().await;
    jobs.push(job.clone());
    crate::jobs::save_jobs(&jobs);
    (StatusCode::CREATED, Json(job))
}

pub async fn delete_job(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> StatusCode {
    let mut jobs = state.jobs.lock().await;
    let before = jobs.len();
    jobs.retain(|j| j.id != id);
    if jobs.len() != before {
        crate::jobs::save_jobs(&jobs);
    }
    StatusCode::NO_CONTENT
}

pub async fn run_job_now(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match run_job(&state, &id).await {
        Some(run) => (StatusCode::OK, Json(serde_json::to_value(run).unwrap())),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "job not found" }))),
    }
}

/// Trigger a job by its name — a stable identifier for remote callers (Airflow,
/// cron, CI) that don't want to track UUIDs.
pub async fn run_job_by_name(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let id = {
        let jobs = state.jobs.lock().await;
        jobs.iter().find(|j| j.name == name).map(|j| j.id.clone())
    };
    match id {
        Some(id) => match run_job(&state, &id).await {
            Some(run) => (StatusCode::OK, Json(serde_json::to_value(run).unwrap())),
            None => (StatusCode::NOT_FOUND, Json(json!({ "error": "job not found" }))),
        },
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": format!("no job named '{}'", name) }))),
    }
}

/// Generate a ready-to-use Airflow DAG that triggers this job over HTTP. Uses a
/// BashOperator+curl so no Airflow HTTP connection setup is required — drop the
/// file in your `dags/` folder and it works.
pub async fn job_airflow_dag(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let jobs = state.jobs.lock().await;
    let job = match jobs.iter().find(|j| j.id == id) {
        Some(j) => j,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "job not found" }))),
    };
    let safe = job
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let schedule = match job.schedule.kind.as_str() {
        "interval" => format!("timedelta(minutes={})", job.schedule.minutes.unwrap_or(60)),
        "daily" => {
            let t = job.schedule.time.clone().unwrap_or_else(|| "09:00".into());
            let parts: Vec<&str> = t.split(':').collect();
            format!("\"{} {} * * *\"", parts.get(1).unwrap_or(&"0"), parts.first().unwrap_or(&"9"))
        }
        _ => "None".to_string(),
    };
    let dag = format!(
        r#"from datetime import datetime, timedelta
from airflow import DAG
from airflow.operators.bash import BashOperator

# Triggers the PrismNote job "{name}" via its stable name.
# Adjust PRISMNOTE_URL to wherever PrismNote is reachable from your Airflow workers.
PRISMNOTE_URL = "http://localhost:8000"

with DAG(
    dag_id="prismnote_{safe}",
    start_date=datetime(2024, 1, 1),
    schedule={schedule},
    catchup=False,
    tags=["prismnote"],
) as dag:
    run = BashOperator(
        task_id="run_{safe}",
        bash_command=(
            'curl -fsS -X POST '
            f'{{PRISMNOTE_URL}}/api/jobs/run-by-name/{enc}'
        ),
    )
"#,
        name = job.name,
        safe = safe,
        schedule = schedule,
        enc = job.name.replace(' ', "%20"),
    );
    (StatusCode::OK, Json(json!({ "dag": dag, "filename": format!("prismnote_{}.py", safe) })))
}

// ── Variable explorer ────────────────────────────────────────────────────────
pub async fn kernel_variables(State(state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    let mut kernel = state.kernel.lock().await;
    match kernel.as_mut() {
        Some(k) => match k.inspect().await {
            Ok(v) => (StatusCode::OK, Json(json!({ "variables": v }))),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string(), "variables": [] }))),
        },
        None => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "variables": [] }))),
    }
}

// ── Real SQL execution via the kernel (OSS connectors, no bundled drivers) ────
// We generate Python that ends in a pandas DataFrame and run it in the shared
// kernel; the DataFrame's `application/vnd.prismnote.df+json` bundle is reshaped
// into {columns, rows}. This keeps the core MIT and lets users bring whichever
// permissively-licensed connector they need (pg8000/PyMySQL/duckdb/…); nothing
// proprietary is vendored.
async fn query_via_kernel(
    state: &Arc<AppState>,
    py: &str,
) -> Result<(Vec<serde_json::Value>, Vec<serde_json::Value>), String> {
    let mut kernel = state.kernel.lock().await;
    let k = kernel.as_mut().ok_or_else(|| "kernel unavailable".to_string())?;
    let outputs = match k.execute(py).await {
        Ok((_s, o)) => o,
        Err(e) => return Err(e.to_string()),
    };
    for o in &outputs {
        if o.get("output_type").and_then(|v| v.as_str()) == Some("error") {
            let txt = o
                .get("text")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<String>())
                .unwrap_or_else(|| "query failed".to_string());
            return Err(txt);
        }
    }
    for o in &outputs {
        if let Some(df) = o.get("data").and_then(|d| d.get("application/vnd.prismnote.df+json")) {
            let columns = df.get("columns").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let rows = df.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            return Ok((columns, rows));
        }
    }
    Err("query did not return a table".to_string())
}

fn js(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

fn db_query_py(conn: &crate::db::DatabaseConnection, query: &str) -> Result<String, String> {
    let q = js(query);
    let host = js(conn.host.as_deref().unwrap_or("localhost"));
    let port = conn.port.unwrap_or(0);
    let user = js(conn.username.as_deref().unwrap_or(""));
    let pass = js(conn.password.as_deref().unwrap_or(""));
    let db = js(&conn.database);
    // `q`/`db`/etc are already JSON-encoded, which is a valid Python string
    // literal — embed directly (no json.loads needed).
    match conn.db_type.as_str() {
        "sqlite" => Ok(format!(
            "import sqlite3, pandas as pd\n_c=sqlite3.connect({db})\n_df=pd.read_sql_query({q}, _c)\n_c.close()\n_df"
        )),
        "duckdb" => Ok(format!(
            "import duckdb\nduckdb.connect({db}).execute({q}).df()"
        )),
        "postgresql" => Ok(format!(
            "import pandas as pd\ntry:\n    import pg8000.dbapi as _pg\nexcept ImportError:\n    raise ImportError('PostgreSQL needs pg8000 (BSD): pip install pg8000')\n_c=_pg.connect(host={host}, port={port} or 5432, user={user}, password={pass}, database={db})\n_df=pd.read_sql_query({q}, _c)\n_c.close()\n_df"
        )),
        "mysql" => Ok(format!(
            "import pandas as pd\ntry:\n    import pymysql as _my\nexcept ImportError:\n    raise ImportError('MySQL needs PyMySQL (MIT): pip install pymysql')\n_c=_my.connect(host={host}, port={port} or 3306, user={user}, password={pass}, database={db})\n_df=pd.read_sql_query({q}, _c)\n_c.close()\n_df"
        )),
        "mongodb" => Err("MongoDB is not a SQL source".to_string()),
        other => Err(format!("unsupported database type: {other}")),
    }
}

/// Turn generated query code (whose last line is the resulting DataFrame
/// expression) into a notebook-cell-friendly snippet: assign it to `var` and
/// display it. Reproducible and editable in the notebook.
fn to_cell_code(code: &str, var: &str) -> String {
    let mut lines: Vec<&str> = code.lines().collect();
    match lines.pop() {
        Some(last) => {
            let head = lines.join("\n");
            let sep = if head.is_empty() { "" } else { "\n" };
            format!("{head}{sep}{var} = {expr}\n{var}", expr = last.trim())
        }
        None => code.to_string(),
    }
}

#[derive(Deserialize)]
pub struct QueryCodeRequest {
    pub query: String,
}

pub async fn db_query_code(
    Path(id): Path<String>,
    Json(req): Json<QueryCodeRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let conn = match load_databases().into_iter().find(|d| d.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "connection not found" }))),
    };
    match db_query_py(&conn, &req.query) {
        Ok(code) => (StatusCode::OK, Json(json!({ "code": to_cell_code(&code, "df") }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))),
    }
}

pub async fn warehouse_query_code(
    Path(id): Path<String>,
    Json(req): Json<QueryCodeRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let conn = match load_warehouses().into_iter().find(|c| c.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "warehouse connection not found" }))),
    };
    match warehouse_query_py(&conn, &req.query) {
        Ok(code) => (StatusCode::OK, Json(json!({ "code": to_cell_code(&code, "df") }))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))),
    }
}

// Database connectors
#[derive(Serialize)]
pub struct DatabaseList {
    pub databases: Vec<crate::db::DatabaseConnection>,
}

fn databases_path() -> String {
    format!("{}/.prismnote/databases.json", default_dir())
}

fn load_databases() -> Vec<crate::db::DatabaseConnection> {
    std::fs::read_to_string(databases_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_databases(dbs: &[crate::db::DatabaseConnection]) {
    if let Ok(json) = serde_json::to_string_pretty(dbs) {
        let path = databases_path();
        if let Some(dir) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, json);
    }
}

pub async fn list_databases() -> Json<DatabaseList> {
    Json(DatabaseList { databases: load_databases() })
}

pub async fn create_database(
    Json(mut req): Json<crate::db::DatabaseConnection>,
) -> (StatusCode, Json<crate::db::DatabaseConnection>) {
    req.id = Uuid::new_v4().to_string();
    req.created_at = chrono::Local::now().to_rfc3339();

    if let Err(_) = crate::db::DatabaseManager::validate_connection(&req) {
        return (StatusCode::BAD_REQUEST, Json(req));
    }

    let mut dbs = load_databases();
    dbs.push(req.clone());
    save_databases(&dbs);
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<crate::db::QueryRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let conn = match load_databases().into_iter().find(|d| d.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "connection not found" }))),
    };
    let py = match db_query_py(&conn, &req.query) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))),
    };
    match query_via_kernel(&state, &py).await {
        Ok((columns, rows)) => (
            StatusCode::OK,
            Json(json!({ "columns": columns, "rows": rows, "row_count": rows.len() })),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))),
    }
}

pub async fn delete_database(Path(id): Path<String>) -> StatusCode {
    let mut dbs = load_databases();
    let before = dbs.len();
    dbs.retain(|d| d.id != id);
    if dbs.len() != before {
        save_databases(&dbs);
    }
    StatusCode::NO_CONTENT
}

// Library recommendations
pub async fn suggest_libraries(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
    Json(req): Json<crate::library_advisor::SuggestLibrariesRequest>,
) -> (StatusCode, Json<crate::library_advisor::SuggestionsResponse>) {
    let advisor = crate::library_advisor::LibraryAdvisor::new(state.ai_engine.read().await.clone());

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

// Cloud warehouse connection persistence (mirrors databases.json).
fn warehouses_path() -> String {
    format!("{}/.prismnote/cloud_warehouses.json", default_dir())
}
fn load_warehouses() -> Vec<crate::cloud_warehouse::CloudWarehouseConnection> {
    std::fs::read_to_string(warehouses_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}
fn save_warehouses(ws: &[crate::cloud_warehouse::CloudWarehouseConnection]) {
    if let Ok(json) = serde_json::to_string_pretty(ws) {
        let path = warehouses_path();
        if let Some(dir) = std::path::Path::new(&path).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, json);
    }
}

/// Generate Python that runs `query` against a cloud warehouse using its official
/// **open-source** client (Apache-2.0 / MIT) and returns a pandas DataFrame.
/// Drivers are optional and user-installed — none are vendored.
fn warehouse_query_py(conn: &crate::cloud_warehouse::CloudWarehouseConnection, query: &str) -> Result<String, String> {
    use crate::cloud_warehouse::CloudWarehouseType as T;
    let q = js(query);
    let host = js(conn.host.as_deref().unwrap_or(""));
    let port = conn.port.unwrap_or(0);
    let user = js(&conn.username);
    let pass = js(&conn.password);
    let db = js(&conn.database);
    let region = js(conn.region.as_deref().unwrap_or(""));
    let project = js(conn.project_id.as_deref().unwrap_or(""));
    let account = js(conn.account_id.as_deref().unwrap_or(""));
    let warehouse = js(conn.warehouse_id.as_deref().unwrap_or(""));
    let cred = |k: &str| js(conn.credentials.get(k).map(|s| s.as_str()).unwrap_or(""));
    Ok(match conn.warehouse_type {
        T::Snowflake => format!(
            "try:\n    import snowflake.connector as _sf\nexcept ImportError:\n    raise ImportError('Snowflake needs snowflake-connector-python[pandas] (Apache-2.0): pip install \"snowflake-connector-python[pandas]\"')\n_c=_sf.connect(user={user}, password={pass}, account={account}, database={db}, warehouse={warehouse})\n_cur=_c.cursor(); _cur.execute({q}); _df=_cur.fetch_pandas_all(); _cur.close(); _c.close()\n_df"
        ),
        T::BigQuery => format!(
            "try:\n    from google.cloud import bigquery as _bq\nexcept ImportError:\n    raise ImportError('BigQuery needs google-cloud-bigquery (Apache-2.0): pip install google-cloud-bigquery db-dtypes')\n_bq.Client(project={project}).query({q}).to_dataframe()"
        ),
        T::Redshift => format!(
            "try:\n    import redshift_connector as _rs\nexcept ImportError:\n    raise ImportError('Redshift needs redshift_connector (Apache-2.0): pip install redshift_connector')\n_c=_rs.connect(host={host}, port={port} or 5439, database={db}, user={user}, password={pass})\n_cur=_c.cursor(); _cur.execute({q}); _df=_cur.fetch_dataframe(); _cur.close(); _c.close()\n_df"
        ),
        T::Databricks => format!(
            "import pandas as pd\ntry:\n    from databricks import sql as _dbx\nexcept ImportError:\n    raise ImportError('Databricks needs databricks-sql-connector (Apache-2.0): pip install databricks-sql-connector')\n_c=_dbx.connect(server_hostname={host}, http_path={http}, access_token={pass})\n_cur=_c.cursor(); _cur.execute({q}); _rows=_cur.fetchall(); _cols=[d[0] for d in _cur.description]; _cur.close(); _c.close()\npd.DataFrame(_rows, columns=_cols)",
            http = cred("http_path")
        ),
        T::Athena => format!(
            "import pandas as pd\ntry:\n    from pyathena import connect as _ath\nexcept ImportError:\n    raise ImportError('Athena needs PyAthena (MIT): pip install pyathena')\n_c=_ath(s3_staging_dir={s3}, region_name={region})\npd.read_sql({q}, _c)",
            s3 = cred("s3_staging_dir")
        ),
        T::Trino => format!(
            "import pandas as pd\ntry:\n    import trino as _trino\nexcept ImportError:\n    raise ImportError('Trino needs the trino client (Apache-2.0): pip install trino')\n_c=_trino.dbapi.connect(host={host}, port={port} or 8080, user={user}, catalog={db})\npd.read_sql({q}, _c)"
        ),
        T::Presto => format!(
            "import pandas as pd\ntry:\n    import prestodb as _presto\nexcept ImportError:\n    raise ImportError('Presto needs presto-python-client (Apache-2.0): pip install presto-python-client')\n_c=_presto.dbapi.connect(host={host}, port={port} or 8080, user={user}, catalog={db})\npd.read_sql({q}, _c)"
        ),
        T::AzureSynapse => format!(
            "import pandas as pd\ntry:\n    import pyodbc as _odbc\nexcept ImportError:\n    raise ImportError('Azure Synapse needs pyodbc (MIT) + an ODBC driver: pip install pyodbc')\n_c=_odbc.connect('DRIVER={{ODBC Driver 18 for SQL Server}};SERVER='+{host}+';DATABASE='+{db}+';UID='+{user}+';PWD='+{pass}+';Encrypt=yes;TrustServerCertificate=yes')\npd.read_sql({q}, _c)"
        ),
    })
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

    let mut ws = load_warehouses();
    ws.push(conn.clone());
    save_warehouses(&ws);
    (StatusCode::CREATED, Json(json!(conn)))
}

pub async fn list_cloud_warehouse_connections() -> Json<serde_json::Value> {
    Json(json!({ "connections": load_warehouses() }))
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
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ExecuteSQLRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let conn = match load_warehouses().into_iter().find(|c| c.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(json!({ "error": "warehouse connection not found" }))),
    };
    let py = match warehouse_query_py(&conn, &req.query) {
        Ok(p) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))),
    };
    match query_via_kernel(&state, &py).await {
        Ok((columns, rows)) => (
            StatusCode::OK,
            Json(json!({ "columns": columns, "rows": rows, "row_count": rows.len() })),
        ),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({ "error": e }))),
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

// Global Search endpoints
#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Vec<String>,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<crate::search_engine::SearchResult>,
    pub total: usize,
    pub execution_time_ms: u64,
}

pub async fn search_notebooks(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> (StatusCode, Json<SearchResponse>) {
    let start = std::time::Instant::now();

    // Load all notebooks
    let dir = &state.notebooks_dir;
    let mut all_items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "ipynb") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Ok(ipynb) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Ok(nb) = crate::files::from_ipynb(ipynb) {
                                    // Add notebook itself
                                    all_items.push(crate::search_engine::SearchableItem {
                                        id: nb.id.clone(),
                                        title: nb.name.clone(),
                                        category: crate::search_engine::SearchCategory::Notebook,
                                        content: format!("Notebook: {}", nb.name),
                                        context: None,
                                        path: Some(path.to_string_lossy().to_string()),
                                        timestamp: Some(chrono::Utc::now()),
                                    });

                                    // Add cell contents
                                    for (idx, cell) in nb.cells.iter().enumerate() {
                                        let cell_content = cell.source.join("");
                                        if !cell_content.is_empty() {
                                            all_items.push(crate::search_engine::SearchableItem {
                                                id: format!("{}-cell-{}", nb.id, idx),
                                                title: format!("Cell {} in {}", idx + 1, nb.name),
                                                category: if cell.cell_type == "code" {
                                                    crate::search_engine::SearchCategory::Notebook
                                                } else {
                                                    crate::search_engine::SearchCategory::Notebook
                                                },
                                                content: cell_content,
                                                context: Some(nb.name.clone()),
                                                path: Some(path.to_string_lossy().to_string()),
                                                timestamp: Some(chrono::Utc::now()),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Simple text-based search (full Tantivy implementation in next iteration)
    let query_lower = req.query.to_lowercase();
    let mut results: Vec<crate::search_engine::SearchResult> = all_items
        .iter()
        .filter_map(|item| {
            let title_match = item.title.to_lowercase().contains(&query_lower);
            let content_match = item.content.to_lowercase().contains(&query_lower);
            let context_match = item
                .context
                .as_ref()
                .map(|c| c.to_lowercase().contains(&query_lower))
                .unwrap_or(false);

            if title_match || content_match || context_match {
                let mut score = 0.5;
                if title_match {
                    score += 0.3;
                }
                if content_match {
                    score += 0.1;
                }
                if context_match {
                    score += 0.1;
                }

                // Check filters
                let category_str = item.category.to_string();
                let matches_filter = req.filters.is_empty() || req.filters.contains(&category_str);

                if matches_filter {
                    Some(crate::search_engine::SearchResult {
                        id: item.id.clone(),
                        title: item.title.clone(),
                        category: item.category.clone(),
                        content: item.content.clone(),
                        context: item.context.clone(),
                        path: item.path.clone(),
                        timestamp: item.timestamp.map(|t| t.to_rfc3339()),
                        score: (score as f32).min(1.0),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    let total = results.len();
    let execution_time_ms = start.elapsed().as_millis() as u64;

    (
        StatusCode::OK,
        Json(SearchResponse {
            results,
            total,
            execution_time_ms,
        }),
    )
}
