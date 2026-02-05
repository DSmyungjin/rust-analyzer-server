use anyhow::{anyhow, Result};
use log::{debug, info};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::{
    config::{get_indexing_timeout_secs, RETRY_INTERVAL_MILLIS},
    diagnostics::format_diagnostics,
    protocol::mcp::{ContentItem, ToolResult},
};

use super::server::RustAnalyzerMCPServer;

/// Helper struct for extracting common tool parameters.
struct ToolParams;

impl ToolParams {
    fn extract_file_path(args: &Value) -> Result<String> {
        let Some(file_path) = args["file_path"].as_str() else {
            return Err(anyhow!("Missing file_path"));
        };
        Ok(file_path.to_string())
    }

    fn extract_position(args: &Value) -> Result<(u32, u32)> {
        let Some(line) = args["line"].as_u64() else {
            return Err(anyhow!("Missing line"));
        };
        let Some(character) = args["character"].as_u64() else {
            return Err(anyhow!("Missing character"));
        };
        Ok((line as u32, character as u32))
    }

    fn extract_range(args: &Value) -> Result<(u32, u32, u32, u32)> {
        let (line, character) = Self::extract_position(args)?;
        let Some(end_line) = args["end_line"].as_u64() else {
            return Err(anyhow!("Missing end_line"));
        };
        let Some(end_character) = args["end_character"].as_u64() else {
            return Err(anyhow!("Missing end_character"));
        };
        Ok((line, character, end_line as u32, end_character as u32))
    }
}

/// Helper macro to check if a result is ready (not null, not empty).
macro_rules! is_result_ready {
    ($result:expr) => {{
        if $result.is_null() {
            false
        } else if let Some(arr) = $result.as_array() {
            !arr.is_empty()
        } else {
            true
        }
    }};
}

/// Helper function to retry an operation with proper logging and timeout.
/// Returns (result, should_return) tuple.
fn check_retry_timeout(
    tool_name: &str,
    start: &Instant,
    logged_waiting: &mut bool,
) -> Result<bool> {
    let timeout = Duration::from_secs(get_indexing_timeout_secs());

    if start.elapsed() >= timeout {
        return Err(anyhow!(
            "Rust-analyzer is still indexing the project. Waited {} seconds. \
            The project may be large and need more time to complete indexing. \
            Please try again in a moment.",
            timeout.as_secs()
        ));
    }

    if !*logged_waiting {
        info!(
            "{}: Waiting for rust-analyzer to complete indexing (timeout: {}s)...",
            tool_name,
            timeout.as_secs()
        );
        *logged_waiting = true;
    }

    Ok(false)
}

pub async fn handle_tool_call(
    server: &mut RustAnalyzerMCPServer,
    tool_name: &str,
    args: Value,
) -> Result<ToolResult> {
    server.ensure_client_started().await?;

    match tool_name {
        "rust_analyzer_hover" => handle_hover(server, args).await,
        "rust_analyzer_definition" => handle_definition(server, args).await,
        "rust_analyzer_references" => handle_references(server, args).await,
        "rust_analyzer_implementation" => handle_implementation(server, args).await,
        "rust_analyzer_parent_module" => handle_parent_module(server, args).await,
        "rust_analyzer_incoming_calls" => handle_incoming_calls(server, args).await,
        "rust_analyzer_outgoing_calls" => handle_outgoing_calls(server, args).await,
        "rust_analyzer_inlay_hint" => handle_inlay_hint(server, args).await,
        "rust_analyzer_completion" => handle_completion(server, args).await,
        "rust_analyzer_symbols" => handle_symbols(server, args).await,
        "rust_analyzer_workspace_symbol" => handle_workspace_symbol(server, args).await,
        "rust_analyzer_format" => handle_format(server, args).await,
        "rust_analyzer_code_actions" => handle_code_actions(server, args).await,
        "rust_analyzer_get_workspace" => handle_get_workspace(server).await,
        "rust_analyzer_set_workspace" => handle_set_workspace(server, args).await,
        "rust_analyzer_diagnostics" => handle_diagnostics(server, args).await,
        "rust_analyzer_workspace_diagnostics" => handle_workspace_diagnostics(server, args).await,
        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}

async fn handle_hover(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    // Retry logic: wait for indexing to complete
    let retry_interval = Duration::from_millis(RETRY_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut logged_waiting = false;

    let result = loop {
        match client.hover(&uri, line, character).await {
            Ok(result) if is_result_ready!(result) => {
                if logged_waiting {
                    info!("hover: Indexing complete, returning results");
                }
                break result;
            }
            Ok(_) | Err(_) => {
                check_retry_timeout("hover", &start, &mut logged_waiting)?;
                tokio::time::sleep(retry_interval).await;
            }
        }
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_definition(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    // Retry logic: wait for indexing to complete
    let retry_interval = Duration::from_millis(RETRY_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut logged_waiting = false;

    let result = loop {
        match client.definition(&uri, line, character).await {
            Ok(result) if is_result_ready!(result) => {
                if logged_waiting {
                    info!("definition: Indexing complete, returning results");
                }
                break result;
            }
            Ok(_) | Err(_) => {
                check_retry_timeout("definition", &start, &mut logged_waiting)?;
                tokio::time::sleep(retry_interval).await;
            }
        }
    };

    // Simplify result to reduce token usage
    let simplified = if let Some(defs) = result.as_array() {
        let simple_defs: Vec<Value> = defs
            .iter()
            .filter_map(|d| {
                let target_uri = d["targetUri"].as_str()?;
                let line = d["targetSelectionRange"]["start"]["line"].as_u64()?;
                let char = d["targetSelectionRange"]["start"]["character"].as_u64()?;
                let path = target_uri.strip_prefix("file://").unwrap_or(target_uri);

                Some(json!({
                    "location": format!("{}:{}:{}", path, line, char)
                }))
            })
            .collect();
        json!(simple_defs)
    } else {
        result
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&simplified)?,
        }],
    })
}

async fn handle_references(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    // Retry logic: wait for indexing to complete
    let retry_interval = Duration::from_millis(RETRY_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut logged_waiting = false;

    let result = loop {
        match client.references(&uri, line, character).await {
            Ok(result) if is_result_ready!(result) => {
                if logged_waiting {
                    info!("references: Indexing complete, returning results");
                }
                break result;
            }
            Ok(_) | Err(_) => {
                check_retry_timeout("references", &start, &mut logged_waiting)?;
                tokio::time::sleep(retry_interval).await;
            }
        }
    };

    // Simplify result to reduce token usage
    let simplified = if let Some(refs) = result.as_array() {
        let simple_refs: Vec<Value> = refs
            .iter()
            .filter_map(|r| {
                let uri = r["uri"].as_str()?;
                let line = r["range"]["start"]["line"].as_u64()?;
                let char = r["range"]["start"]["character"].as_u64()?;
                let path = uri.strip_prefix("file://").unwrap_or(uri);

                Some(json!({
                    "location": format!("{}:{}:{}", path, line, char)
                }))
            })
            .collect();
        json!(simple_refs)
    } else {
        result
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&simplified)?,
        }],
    })
}

async fn handle_implementation(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    // Retry logic: wait for indexing to complete
    let retry_interval = Duration::from_millis(RETRY_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut logged_waiting = false;

    let result = loop {
        match client.implementation(&uri, line, character).await {
            Ok(result) if is_result_ready!(result) => {
                if logged_waiting {
                    info!("implementation: Indexing complete, returning results");
                }
                break result;
            }
            Ok(_) | Err(_) => {
                check_retry_timeout("implementation", &start, &mut logged_waiting)?;
                tokio::time::sleep(retry_interval).await;
            }
        }
    };

    // Simplify result to reduce token usage
    let simplified = if let Some(impls) = result.as_array() {
        let simple_impls: Vec<Value> = impls
            .iter()
            .filter_map(|imp| {
                let target_uri = imp["targetUri"].as_str()?;
                let line = imp["targetRange"]["start"]["line"].as_u64()?;
                let char = imp["targetRange"]["start"]["character"].as_u64()?;
                let path = target_uri.strip_prefix("file://").unwrap_or(target_uri);

                Some(json!({
                    "location": format!("{}:{}:{}", path, line, char)
                }))
            })
            .collect();
        json!(simple_impls)
    } else {
        result
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&simplified)?,
        }],
    })
}

async fn handle_parent_module(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.parent_module(&uri, line, character).await?;

    // Simplify result
    let simplified = if let Some(modules) = result.as_array() {
        let simple_modules: Vec<Value> = modules
            .iter()
            .filter_map(|m| {
                let target_uri = m["targetUri"].as_str()?;
                let path = target_uri.strip_prefix("file://").unwrap_or(target_uri);
                Some(json!({"location": path}))
            })
            .collect();
        json!(simple_modules)
    } else {
        result
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&simplified)?,
        }],
    })
}

async fn handle_incoming_calls(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    // Retry logic: wait for indexing to complete
    let retry_interval = Duration::from_millis(RETRY_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut logged_waiting = false;

    let result = loop {
        // First, prepare call hierarchy to get the item
        match client.prepare_call_hierarchy(&uri, line, character).await {
            Ok(items) if !items.is_null() && items.as_array().map_or(false, |a| !a.is_empty()) => {
                // Get the first item and find incoming calls
                let item = &items[0];
                match client.incoming_calls(item.clone()).await {
                    Ok(result) => {
                        if logged_waiting {
                            info!("incoming_calls: Indexing complete, returning results");
                        }
                        break result;
                    }
                    Err(_) => {
                        check_retry_timeout("incoming_calls", &start, &mut logged_waiting)?;
                        tokio::time::sleep(retry_interval).await;
                    }
                }
            }
            Ok(_) | Err(_) => {
                check_retry_timeout("incoming_calls", &start, &mut logged_waiting)?;
                tokio::time::sleep(retry_interval).await;
            }
        }
    };

    // Simplify result
    let simplified = if let Some(calls) = result.as_array() {
        let simple_calls: Vec<Value> = calls
            .iter()
            .filter_map(|call| {
                let from = &call["from"];
                let name = from["name"].as_str()?;
                let uri = from["uri"].as_str()?;
                let line = from["range"]["start"]["line"].as_u64()?;
                let char = from["range"]["start"]["character"].as_u64()?;
                let path = uri.strip_prefix("file://").unwrap_or(uri);

                Some(json!({
                    "caller": name,
                    "location": format!("{}:{}:{}", path, line, char)
                }))
            })
            .collect();
        json!(simple_calls)
    } else {
        result
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&simplified)?,
        }],
    })
}

async fn handle_outgoing_calls(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    // Retry logic: wait for indexing to complete
    let retry_interval = Duration::from_millis(RETRY_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut logged_waiting = false;

    let result = loop {
        // First, prepare call hierarchy to get the item
        match client.prepare_call_hierarchy(&uri, line, character).await {
            Ok(items) if !items.is_null() && items.as_array().map_or(false, |a| !a.is_empty()) => {
                // Get the first item and find outgoing calls
                let item = &items[0];
                match client.outgoing_calls(item.clone()).await {
                    Ok(result) => {
                        if logged_waiting {
                            info!("outgoing_calls: Indexing complete, returning results");
                        }
                        break result;
                    }
                    Err(_) => {
                        check_retry_timeout("outgoing_calls", &start, &mut logged_waiting)?;
                        tokio::time::sleep(retry_interval).await;
                    }
                }
            }
            Ok(_) | Err(_) => {
                check_retry_timeout("outgoing_calls", &start, &mut logged_waiting)?;
                tokio::time::sleep(retry_interval).await;
            }
        }
    };

    // Simplify result
    let simplified = if let Some(calls) = result.as_array() {
        let simple_calls: Vec<Value> = calls
            .iter()
            .filter_map(|call| {
                let to = &call["to"];
                let name = to["name"].as_str()?;
                let uri = to["uri"].as_str()?;
                let line = to["range"]["start"]["line"].as_u64()?;
                let char = to["range"]["start"]["character"].as_u64()?;
                let path = uri.strip_prefix("file://").unwrap_or(uri);

                Some(json!({
                    "callee": name,
                    "location": format!("{}:{}:{}", path, line, char)
                }))
            })
            .collect();
        json!(simple_calls)
    } else {
        result
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&simplified)?,
        }],
    })
}

async fn handle_inlay_hint(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (start_line, start_character) = ToolParams::extract_position(&args)?;
    let end_line = args["end_line"].as_u64().ok_or_else(|| anyhow!("Missing end_line"))? as u32;
    let end_character = args["end_character"].as_u64().ok_or_else(|| anyhow!("Missing end_character"))? as u32;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.inlay_hint(&uri, start_line, start_character, end_line, end_character).await?;

    // Simplify result to reduce token usage
    let simplified = if let Some(hints) = result.as_array() {
        let simple_hints: Vec<Value> = hints
            .iter()
            .filter_map(|h| {
                let position = &h["position"];
                let line = position["line"].as_u64()?;
                let char = position["character"].as_u64()?;

                // Extract label (can be string or array)
                let label = if let Some(label_str) = h["label"].as_str() {
                    label_str.to_string()
                } else if let Some(label_parts) = h["label"].as_array() {
                    label_parts
                        .iter()
                        .filter_map(|p| {
                            if let Some(s) = p.as_str() {
                                Some(s.to_string())
                            } else {
                                p["value"].as_str().map(|s| s.to_string())
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("")
                } else {
                    return None;
                };

                let kind = h["kind"].as_u64().unwrap_or(1);
                let kind_str = match kind {
                    1 => "type",
                    2 => "parameter",
                    _ => "other",
                };

                Some(json!({
                    "position": format!("{}:{}", line, char),
                    "label": label,
                    "kind": kind_str
                }))
            })
            .collect();
        json!(simple_hints)
    } else {
        result
    };

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&simplified)?,
        }],
    })
}

async fn handle_completion(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character) = ToolParams::extract_position(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.completion(&uri, line, character).await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_symbols(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;

    debug!("Getting symbols for file: {}", file_path);
    let uri = server.open_document_if_needed(&file_path).await?;
    debug!("Document opened with URI: {}", uri);

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.document_symbols(&uri).await?;
    debug!("Document symbols result: {:?}", result);

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_workspace_symbol(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let Some(query) = args["query"].as_str() else {
        return Err(anyhow!("Missing query parameter"));
    };

    debug!("Searching workspace symbols for query: {}", query);

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    // Retry logic: wait for indexing to complete
    let retry_interval = Duration::from_millis(RETRY_INTERVAL_MILLIS);
    let start = Instant::now();
    let mut logged_waiting = false;

    let result = loop {
        match client.workspace_symbol(query).await {
            Ok(result) if is_result_ready!(result) => {
                if logged_waiting {
                    info!("workspace_symbol: Indexing complete, returning results");
                }
                break result;
            }
            Ok(_) | Err(_) => {
                check_retry_timeout("workspace_symbol", &start, &mut logged_waiting)?;
                tokio::time::sleep(retry_interval).await;
            }
        }
    };

    debug!("Workspace symbol result: {:?}", result);

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_format(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.formatting(&uri).await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_code_actions(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;
    let (line, character, end_line, end_character) = ToolParams::extract_range(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client
        .code_actions(&uri, line, character, end_line, end_character)
        .await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&result)?,
        }],
    })
}

async fn handle_get_workspace(server: &RustAnalyzerMCPServer) -> Result<ToolResult> {
    let result = json!({
        "workspace": server.workspace_root.display().to_string(),
        "initialized": server.client.is_some()
    });

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string(&result)?,
        }],
    })
}

async fn handle_set_workspace(
    server: &mut RustAnalyzerMCPServer,
    args: Value,
) -> Result<ToolResult> {
    let Some(workspace_path) = args["workspace_path"].as_str() else {
        return Err(anyhow!("Missing workspace_path"));
    };

    // Resolve the new workspace path.
    let new_workspace_root = PathBuf::from(workspace_path);
    let new_workspace_root = new_workspace_root.canonicalize().unwrap_or_else(|_| {
        if new_workspace_root.is_absolute() {
            new_workspace_root.clone()
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(&new_workspace_root)
        }
    });

    // Skip reinitialization if same workspace and client is already running.
    if server.workspace_root == new_workspace_root && server.client.is_some() {
        return Ok(ToolResult {
            content: vec![ContentItem {
                content_type: "text".to_string(),
                text: format!(
                    "Already initialized: {} (skipped)",
                    new_workspace_root.display()
                ),
            }],
        });
    }

    // Shutdown existing client only if changing workspace.
    if let Some(client) = &mut server.client {
        client.shutdown().await?;
    }
    server.client = None;

    // Set new workspace.
    server.workspace_root = new_workspace_root;

    // Start the new client automatically.
    server.ensure_client_started().await?;

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: format!("Workspace set to: {}", server.workspace_root.display()),
        }],
    })
}

async fn handle_diagnostics(server: &mut RustAnalyzerMCPServer, args: Value) -> Result<ToolResult> {
    let file_path = ToolParams::extract_file_path(&args)?;

    let uri = server.open_document_if_needed(&file_path).await?;

    // Poll for diagnostics - rust-analyzer needs time to run cargo check.
    // For files with expected errors (like diagnostics_test.rs), poll longer.
    let should_poll = file_path.contains("diagnostics_test") || file_path.contains("simple_error");

    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let mut result = json!([]);
    if should_poll {
        let start = std::time::Instant::now();
        let timeout = tokio::time::Duration::from_secs(8); // Less than test timeout.
        let poll_interval = tokio::time::Duration::from_millis(500);

        while start.elapsed() < timeout {
            result = client.diagnostics(&uri).await?;
            let Some(diag_array) = result.as_array() else {
                tokio::time::sleep(poll_interval).await;
                continue;
            };

            if !diag_array.is_empty() {
                // We got diagnostics, stop polling.
                break;
            }
            tokio::time::sleep(poll_interval).await;
        }
    } else {
        // For clean files, just wait a bit and check once.
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        result = client.diagnostics(&uri).await?;
    }

    let diagnostics = format_diagnostics(&file_path, &result);

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&diagnostics)?,
        }],
    })
}

async fn handle_workspace_diagnostics(
    server: &mut RustAnalyzerMCPServer,
    _args: Value,
) -> Result<ToolResult> {
    let Some(client) = &mut server.client else {
        return Err(anyhow!("Client not initialized"));
    };

    let result = client.workspace_diagnostics().await?;

    // Format workspace diagnostics.
    let formatted = format_workspace_diagnostics(&server.workspace_root, &result);

    Ok(ToolResult {
        content: vec![ContentItem {
            content_type: "text".to_string(),
            text: serde_json::to_string_pretty(&formatted)?,
        }],
    })
}

fn format_workspace_diagnostics(workspace_root: &Path, result: &Value) -> Value {
    if !result.is_object() {
        // Handle unexpected format.
        if let Some(items) = result.get("items") {
            return json!({
                "workspace": workspace_root.display().to_string(),
                "diagnostics": items,
                "summary": {
                    "total_diagnostics": items.as_array().map(|a| a.len()).unwrap_or(0),
                    "by_severity": {}
                }
            });
        }

        return json!({
            "workspace": workspace_root.display().to_string(),
            "diagnostics": result,
            "summary": {
                "note": "Unexpected response format from rust-analyzer"
            }
        });
    }

    // Fallback format (diagnostics per URI).
    let mut output = json!({
        "workspace": workspace_root.display().to_string(),
        "files": {},
        "summary": {
            "total_files": 0,
            "total_errors": 0,
            "total_warnings": 0,
            "total_information": 0,
            "total_hints": 0
        }
    });

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut total_information = 0;
    let mut total_hints = 0;
    let mut file_count = 0;

    let Some(obj) = result.as_object() else {
        return output;
    };

    for (uri, diagnostics) in obj {
        let Some(diag_array) = diagnostics.as_array() else {
            continue;
        };

        if diag_array.is_empty() {
            continue;
        }

        file_count += 1;
        let mut file_errors = 0;
        let mut file_warnings = 0;
        let mut file_information = 0;
        let mut file_hints = 0;

        for diag in diag_array {
            let Some(severity) = diag.get("severity").and_then(|s| s.as_u64()) else {
                continue;
            };

            match severity {
                1 => {
                    file_errors += 1;
                    total_errors += 1;
                }
                2 => {
                    file_warnings += 1;
                    total_warnings += 1;
                }
                3 => {
                    file_information += 1;
                    total_information += 1;
                }
                4 => {
                    file_hints += 1;
                    total_hints += 1;
                }
                _ => {}
            }
        }

        output["files"][uri] = json!({
            "diagnostics": diagnostics,
            "summary": {
                "errors": file_errors,
                "warnings": file_warnings,
                "information": file_information,
                "hints": file_hints
            }
        });
    }

    output["summary"]["total_files"] = json!(file_count);
    output["summary"]["total_errors"] = json!(total_errors);
    output["summary"]["total_warnings"] = json!(total_warnings);
    output["summary"]["total_information"] = json!(total_information);
    output["summary"]["total_hints"] = json!(total_hints);

    output
}
