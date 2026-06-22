mod ai;
mod ai_training;
mod api;
mod cell_executor;
mod cloud_warehouse;
mod cloud_storage;
mod data_profiler;
mod db;
mod enterprise_auth;
mod github_integration;
mod output_renderer;
mod k8s_deployment;
mod dbt_integration;
mod airflow_integration;
mod docker_executor;
mod execution_pipeline;
mod file_manager;
mod files;
mod jobs;
mod kernel;
mod library_advisor;
mod models;
mod platform;
mod rbac;
mod realtime_collab;
mod scheduler;
mod search_engine;
mod spark_manager;
mod sql_executor;
mod versioning;
mod ws;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

// The built frontend is embedded into the binary so a single downloaded
// executable is fully self-contained (no ./frontend/dist on disk required).
#[derive(RustEmbed)]
#[folder = "../../frontend/dist"]
struct Assets;

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    match Assets::get(path) {
        Some(content) => (
            [(header::CONTENT_TYPE, content.metadata.mimetype().to_string())],
            content.data.into_owned(),
        )
            .into_response(),
        // SPA fallback: serve index.html for unknown non-asset routes.
        None => match Assets::get("index.html") {
            Some(c) => (
                [(header::CONTENT_TYPE, "text/html".to_string())],
                c.data.into_owned(),
            )
                .into_response(),
            None => (StatusCode::NOT_FOUND, "not found").into_response(),
        },
    }
}
use tracing_subscriber;

pub struct AppState {
    notebooks_dir: String,
    /// Runtime-mutable so AI settings saved from the UI take effect immediately.
    ai_engine: tokio::sync::RwLock<Option<Arc<ai::AIEngine>>>,
    /// Where the AI config is persisted (so it survives restarts).
    ai_config_path: String,
    kernel: tokio::sync::Mutex<Option<kernel::KernelManager>>,
    /// Live interpreter PID (0 = none), used to interrupt a running cell.
    kernel_pid: std::sync::Arc<std::sync::atomic::AtomicI32>,
    /// Saved notebooks that run as a unit (optionally scheduled), Airflow-style.
    jobs: tokio::sync::Mutex<Vec<jobs::Job>>,
    /// Live cell output stream (JSON {cell_id, text}) broadcast to WebSocket
    /// clients. Purely additive — the HTTP response is still authoritative.
    stream_tx: tokio::sync::broadcast::Sender<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let notebooks_dir = std::env::var("PRISMNOTE_DIR")
        .unwrap_or_else(|_| format!("{}/.prismnote/notebooks", dirs::home_dir().unwrap().display()));

    std::fs::create_dir_all(&notebooks_dir)?;

    let prismnote_home = format!("{}/.prismnote", dirs::home_dir().unwrap().display());
    std::fs::create_dir_all(&prismnote_home).ok();
    let ai_config_path = format!("{}/ai_config.json", prismnote_home);

    // AI engine: prefer env vars; otherwise load a previously-saved UI config.
    let ai_engine = if let Ok(provider) = std::env::var("PRISMNOTE_AI_PROVIDER") {
        let config = ai::AIConfig {
            provider,
            ollama_url: std::env::var("PRISMNOTE_OLLAMA_URL").ok(),
            ollama_model: std::env::var("PRISMNOTE_OLLAMA_MODEL").ok(),
            claude_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_model: std::env::var("PRISMNOTE_OPENAI_MODEL").ok(),
        };
        Some(Arc::new(ai::AIEngine::new(config)))
    } else if let Ok(text) = std::fs::read_to_string(&ai_config_path) {
        match serde_json::from_str::<ai::AIConfig>(&text) {
            Ok(config) => Some(Arc::new(ai::AIEngine::new(config))),
            Err(_) => None,
        }
    } else {
        None
    };

    // Initialize Jupyter kernel
    let kernel = match kernel::KernelManager::new() {
        Ok(k) => {
            tracing::info!("Jupyter kernel initialized");
            Some(k)
        }
        Err(e) => {
            tracing::warn!("Failed to initialize kernel: {}", e);
            None
        }
    };

    let kernel_pid = kernel
        .as_ref()
        .map(|k| k.pid_handle())
        .unwrap_or_else(|| std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0)));

    let state = Arc::new(AppState {
        notebooks_dir,
        ai_engine: tokio::sync::RwLock::new(ai_engine),
        ai_config_path,
        kernel: tokio::sync::Mutex::new(kernel),
        kernel_pid,
        jobs: tokio::sync::Mutex::new(jobs::load_jobs()),
        stream_tx: tokio::sync::broadcast::channel(2048).0,
    });

    // Background scheduler: every 60s, run any jobs whose schedule is due.
    {
        let sched_state = state.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                api::run_due_jobs(&sched_state).await;
            }
        });
    }

    let api_routes = Router::new()
        .route("/notebooks", get(api::list_notebooks).post(api::create_notebook))
        .route("/notebooks/:id", get(api::get_notebook).put(api::update_notebook).delete(api::delete_notebook))
        .route("/notebooks/:id/execute", post(api::execute_cell))
        .route("/kernel/interrupt", post(api::kernel_interrupt))
        .route("/kernel/restart", post(api::kernel_restart))
        .route("/kernel/variables", get(api::kernel_variables))
        .route("/jobs", get(api::list_jobs).post(api::create_job))
        .route("/jobs/:id", get(api::get_job).delete(api::delete_job))
        .route("/jobs/:id/run", post(api::run_job_now))
        .route("/jobs/run-by-name/:name", post(api::run_job_by_name))
        .route("/jobs/:id/airflow-dag", get(api::job_airflow_dag))
        .route("/git/status", get(api::git_status))
        .route("/git/init", post(api::git_init))
        .route("/git/clone", post(api::git_clone))
        .route("/git/commit", post(api::git_commit))
        .route("/git/push", post(api::git_push))
        .route("/git/pull", post(api::git_pull))
        .route("/deploy/artifacts", get(api::deploy_artifacts))
        .route("/terminal/exec", post(api::terminal_exec))
        .route("/notebooks/:id/suggest-libraries", post(api::suggest_libraries))
        .route("/notebooks/:id/libraries/ignore", post(api::ignore_library))
        .route("/notebooks/:id/libraries/ignored", get(api::get_ignored_libraries))
        .route("/notebooks/:id/execution-plan", post(api::build_execution_plan))
        .route("/notebooks/:id/execution-stats", get(api::get_execution_statistics))
        .route("/ai/explain", post(api::ai_explain))
        .route("/ai/fix", post(api::ai_fix))
        .route("/ai/complete", post(api::ai_complete))
        .route("/ai/edit", post(api::ai_edit))
        .route("/fs/list", get(api::fs_list))
        .route("/fs/read", get(api::fs_read))
        .route("/ai/config", get(api::get_ai_config).post(api::set_ai_config))
        .route("/databases", get(api::list_databases).post(api::create_database))
        .route("/databases/:id/test", post(api::test_database))
        .route("/databases/:id/query", post(api::execute_database_query))
        .route("/databases/:id", delete(api::delete_database))
        .route("/sql/execute", post(api::execute_sql))
        .route("/sql/optimize", post(api::get_query_optimizations))
        .route("/spark/sessions", post(api::create_spark_session).get(api::list_spark_sessions))
        .route("/spark/sessions/:id", get(api::get_spark_session))
        .route("/cloud-warehouses", post(api::create_cloud_warehouse_connection).get(api::list_cloud_warehouse_connections))
        .route("/cloud-warehouses/:id/test", post(api::test_cloud_warehouse_connection))
        .route("/cloud-warehouses/:id/query", post(api::execute_cloud_warehouse_query))
        .route("/cloud-warehouses/:id/databases", get(api::get_cloud_warehouse_databases))
        .route("/cloud-warehouses/:id/databases/:db/tables", get(api::get_cloud_warehouse_tables))
        .route("/cloud-warehouses/:id/estimate-cost", post(api::estimate_cloud_query_cost))
        .route("/ai/fine-tuning/jobs", post(api::create_fine_tuning_job).get(api::list_fine_tuning_jobs))
        .route("/ai/fine-tuning/jobs/:id", get(api::get_fine_tuning_job))
        .route("/ai/fine-tuning/jobs/:id/start", post(api::start_fine_tuning_job))
        .route("/ai/fine-tuning/jobs/:id/cancel", post(api::cancel_fine_tuning_job))
        .route("/ai/fine-tuning/jobs/:id/checkpoints", get(api::list_model_checkpoints))
        .route("/ai/inference/endpoints", post(api::deploy_inference_endpoint).get(api::list_inference_endpoints))
        .route("/ai/inference/endpoints/:id", delete(api::delete_inference_endpoint))
        .route("/ai/compute/runpod-instances", get(api::get_runpod_instances))
        // Realtime collaboration (v0.4)
        .route("/notebooks/:id/collaborate", post(api::join_collaboration))
        .route("/notebooks/:id/collaborators", get(api::get_active_collaborators))
        .route("/notebooks/:id/comments", post(api::post_comment))
        // File upload/download (v0.4)
        .route("/notebooks/:id/files", post(api::upload_file).get(api::list_files))
        .route("/notebooks/:id/files/:file_id", get(api::download_file).delete(api::delete_file))
        // Cloud storage (v0.4)
        .route("/cloud-storage", post(api::add_cloud_storage).get(api::list_cloud_storage))
        .route("/cloud-storage/:name", delete(api::remove_cloud_storage))
        // GitHub integration (v0.5)
        .route("/github/configure", post(api::configure_github))
        .route("/notebooks/:id/github/sync", post(api::sync_with_github))
        .route("/notebooks/:id/github/push", post(api::push_to_github))
        .route("/notebooks/:id/github/pull", get(api::pull_from_github))
        // Output zoom and fullscreen
        .route("/outputs/:output_id/zoom", put(api::set_output_zoom))
        .route("/outputs/:output_id/fullscreen", get(api::fullscreen_output))
        .route("/outputs/:cell_id/zoom/reset", post(api::reset_output_zoom))
        // Typography and display settings
        .route("/settings/display", get(api::get_display_settings).put(api::update_display_settings))
        .route("/settings/fonts/mac", get(api::get_mac_compatible_fonts))
        // Kubernetes and Docker (v1.0)
        .route("/infra/k8s/manifest", get(api::get_k8s_manifest))
        .route("/infra/k8s/deploy", post(api::deploy_to_k8s))
        .route("/infra/k8s/pods", get(api::get_k8s_pods))
        .route("/infra/docker/compose", get(api::get_docker_compose))
        // dbt integration (v1.0)
        .route("/notebooks/:id/dbt/models", get(api::list_dbt_models))
        .route("/notebooks/:id/dbt/run", post(api::run_dbt_models))
        .route("/notebooks/:id/dbt/test", post(api::run_dbt_tests))
        .route("/dbt/config", get(api::get_dbt_project_yml))
        // Airflow integration (v1.0)
        .route("/airflow/dags", get(api::list_airflow_dags))
        .route("/airflow/dags/:dag_id/trigger", post(api::trigger_airflow_dag))
        .route("/airflow/dags/:dag_id/status", get(api::get_airflow_dag_status))
        .route("/airflow/generate-dag", post(api::generate_airflow_dag))
        // Docker container code execution (v1.0)
        .route("/docker/containers", get(api::list_docker_containers))
        .route("/docker/containers/execute", post(api::execute_code_in_container))
        .route("/docker/containers/:container_id/start", post(api::start_docker_container))
        .route("/docker/containers/:container_id/stop", post(api::stop_docker_container))
        .route("/docker/containers/:container_id", delete(api::remove_docker_container))
        .route("/docker/containers/create", post(api::create_docker_container))
        .route("/docker/containers/:container_id/logs", get(api::get_container_logs))
        .route("/docker/containers/:container_id/stats", get(api::get_container_stats))
        .route("/docker/containers/:container_id/files/:path",
            get(api::read_container_file).post(api::write_container_file))
        .route("/docker/containers/:container_id/files-list/:path",
            get(api::get_container_files))
        .route("/docker/images/pull", post(api::pull_docker_image))
        // Global search (v0.3)
        .route("/search", post(api::search_notebooks))
        .with_state(state.clone());

    let ws_routes = Router::new()
        .route("/notebook/:id", axum::routing::get(ws::notebook_ws))
        .with_state(state);

    let app = Router::new()
        .nest("/api", api_routes)
        .nest("/ws", ws_routes)
        .fallback(static_handler)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    tracing::info!("Server running on http://{}", addr);
    tracing::info!("Serving static files from ./frontend/dist");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
