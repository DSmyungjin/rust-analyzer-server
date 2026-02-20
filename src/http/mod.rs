pub(crate) mod routes;
mod state;

pub use state::AppState;

use std::sync::Arc;
use tokio::sync::{watch, Mutex};

use axum::{
    routing::{get, post},
    Router,
};
use log::info;

use crate::RustAnalyzerMCPServer;

pub async fn serve(bind: &str, port: u16, server: RustAnalyzerMCPServer) -> anyhow::Result<()> {
    let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

    let state = AppState {
        server: Arc::new(Mutex::new(server)),
        shutdown_tx,
    };

    let router = Router::new()
        .route("/api/v1/health", get(routes::health))
        .route("/api/v1/status", get(routes::status))
        .route("/api/v1/tools", get(routes::list_tools))
        .route("/api/v1/workspace", get(routes::get_workspace))
        .route("/api/v1/workspace", post(routes::set_workspace))
        .route("/api/v1/shutdown", post(routes::shutdown))
        .route("/api/v1/:tool_name", post(routes::call_tool))
        .with_state(state);

    let addr = format!("{}:{}", bind, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("rust-analyzer HTTP server listening on http://{}", addr);
    info!("rust-analyzer HTTP server listening on http://{}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            // Shut down on Ctrl-C or /api/v1/shutdown
            let ctrl_c = tokio::signal::ctrl_c();
            let shutdown_signal = async {
                while shutdown_rx.changed().await.is_ok() {
                    if *shutdown_rx.borrow() {
                        return;
                    }
                }
            };
            tokio::select! {
                _ = ctrl_c => { info!("Received Ctrl-C, shutting down"); }
                _ = shutdown_signal => { info!("Received shutdown request"); }
            }
        })
        .await?;

    Ok(())
}
