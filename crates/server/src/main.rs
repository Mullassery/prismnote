mod api;
mod files;
mod kernel;
mod models;
mod ws;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let notebooks_dir = std::env::var("PRISMNOTE_DIR")
        .unwrap_or_else(|_| format!("{}/.prismnote/notebooks", dirs::home_dir().unwrap().display()));

    std::fs::create_dir_all(&notebooks_dir)?;

    let state = Arc::new(AppState { notebooks_dir });

    let api_routes = Router::new()
        .route("/notebooks", get(api::list_notebooks).post(api::create_notebook))
        .route("/notebooks/:id", get(api::get_notebook).delete(api::delete_notebook))
        .route("/notebooks/:id/execute", post(api::execute_cell))
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
