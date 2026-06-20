mod ai;
mod api;
mod files;
mod kernel;
mod models;
mod ws;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post, put},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

pub struct AppState {
    notebooks_dir: String,
    ai_engine: Option<ai::AIEngine>,
    kernel: tokio::sync::Mutex<Option<kernel::KernelManager>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let notebooks_dir = std::env::var("PRISMNOTE_DIR")
        .unwrap_or_else(|_| format!("{}/.prismnote/notebooks", dirs::home_dir().unwrap().display()));

    std::fs::create_dir_all(&notebooks_dir)?;

    // Try to initialize AI engine from environment variables
    let ai_engine = if let Ok(provider) = std::env::var("PRISMNOTE_AI_PROVIDER") {
        let config = ai::AIConfig {
            provider,
            ollama_url: std::env::var("PRISMNOTE_OLLAMA_URL").ok(),
            ollama_model: std::env::var("PRISMNOTE_OLLAMA_MODEL").ok(),
            claude_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_model: std::env::var("PRISMNOTE_OPENAI_MODEL").ok(),
        };
        Some(ai::AIEngine::new(config))
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

    let state = Arc::new(AppState {
        notebooks_dir,
        ai_engine,
        kernel: tokio::sync::Mutex::new(kernel),
    });

    let api_routes = Router::new()
        .route("/notebooks", get(api::list_notebooks).post(api::create_notebook))
        .route("/notebooks/:id", get(api::get_notebook).put(api::update_notebook).delete(api::delete_notebook))
        .route("/notebooks/:id/execute", post(api::execute_cell))
        .route("/ai/explain", post(api::ai_explain))
        .route("/ai/fix", post(api::ai_fix))
        .route("/ai/complete", post(api::ai_complete))
        .route("/ai/config", get(api::get_ai_config).post(api::set_ai_config))
        .with_state(state.clone());

    let ws_routes = Router::new()
        .route("/notebook/:id", axum::routing::get(ws::notebook_ws))
        .with_state(state);

    let app = Router::new()
        .nest("/api", api_routes)
        .nest("/ws", ws_routes)
        .fallback_service(ServeDir::new("./frontend/dist"))
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
