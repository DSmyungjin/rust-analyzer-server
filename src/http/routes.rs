use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::mcp::{handlers::handle_tool_call, tools::get_tools};

use super::state::AppState;

#[derive(Serialize)]
pub(crate) struct ApiResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ApiResponse {
    fn success(result: Value) -> Json<ApiResponse> {
        Json(ApiResponse {
            ok: true,
            result: Some(result),
            error: None,
        })
    }

    fn error(msg: impl Into<String>) -> (StatusCode, Json<ApiResponse>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                ok: false,
                result: None,
                error: Some(msg.into()),
            }),
        )
    }
}

pub async fn health(State(state): State<AppState>) -> Json<ApiResponse> {
    let server = state.server.lock().await;
    ApiResponse::success(json!({
        "status": "ok",
        "workspace": server.workspace_root.display().to_string(),
        "initialized": server.client.is_some(),
    }))
}

pub async fn status(State(state): State<AppState>) -> Json<ApiResponse> {
    let server = state.server.lock().await;
    let has_client = server.client.is_some();
    let is_indexing = server.is_indexing().await;
    let active_tasks = server.active_progress().await;
    let workspace_valid = server.workspace_exists();
    let (trigger, previous_workspace) = server.trigger_info();

    let server_state = if !workspace_valid {
        "error"
    } else if !has_client {
        "stopped"
    } else if is_indexing {
        "indexing"
    } else {
        "ready"
    };

    let mut result = json!({
        "workspace": server.workspace_root.display().to_string(),
        "workspace_valid": workspace_valid,
        "state": server_state,
        "initialized": has_client,
        "indexing": is_indexing,
        "trigger": trigger,
        "progress": active_tasks,
    });

    if let Some(prev) = previous_workspace {
        result["previous_workspace"] = json!(prev);
    }

    ApiResponse::success(result)
}

pub async fn list_tools() -> Json<ApiResponse> {
    let tools = get_tools();
    ApiResponse::success(json!({ "tools": tools }))
}

pub async fn get_workspace(State(state): State<AppState>) -> Json<ApiResponse> {
    let server = state.server.lock().await;
    ApiResponse::success(json!({
        "workspace": server.workspace_root.display().to_string(),
        "initialized": server.client.is_some(),
    }))
}

#[derive(Deserialize)]
pub struct SetWorkspaceRequest {
    pub workspace_path: String,
}

pub async fn set_workspace(
    State(state): State<AppState>,
    Json(body): Json<SetWorkspaceRequest>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    let mut server = state.server.lock().await;
    let args = json!({ "workspace_path": body.workspace_path });
    match handle_tool_call(&mut server, "rust_analyzer_set_workspace", args).await {
        Ok(result) => {
            let text = result
                .content
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_default();
            Ok(ApiResponse::success(json!({ "message": text })))
        }
        Err(e) => Err(ApiResponse::error(e.to_string())),
    }
}

pub async fn shutdown(State(state): State<AppState>) -> Json<ApiResponse> {
    let mut server = state.server.lock().await;
    server.shutdown().await;
    let _ = state.shutdown_tx.send(true);
    ApiResponse::success(json!({ "message": "shutting down" }))
}

pub async fn call_tool(
    State(state): State<AppState>,
    Path(tool_name): Path<String>,
    Json(args): Json<Value>,
) -> Result<Json<ApiResponse>, (StatusCode, Json<ApiResponse>)> {
    let mut server = state.server.lock().await;
    match handle_tool_call(&mut server, &tool_name, args).await {
        Ok(result) => {
            // Parse the text content back to JSON if possible, otherwise return as string
            let value = if let Some(item) = result.content.first() {
                serde_json::from_str(&item.text).unwrap_or_else(|_| json!(item.text))
            } else {
                json!(null)
            };
            Ok(ApiResponse::success(value))
        }
        Err(e) => Err(ApiResponse::error(e.to_string())),
    }
}
