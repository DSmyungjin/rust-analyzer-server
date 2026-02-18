use anyhow::Result;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::process::Child;

/// Client that connects to the HTTP MCP server
pub struct IpcClient {
    http_client: reqwest::Client,
    base_url: String,
    port: u16,
    workspace_path: PathBuf,
    /// Keeps the server process alive; dropped when client is dropped.
    _process: Option<Child>,
}

impl IpcClient {
    /// Connect to or start an HTTP MCP server
    pub async fn get_or_create(project_type: &str) -> Result<Self> {
        // Map project types to workspace paths
        let workspace_path = match project_type {
            "test-project" | "test-project-singleton" | "test-project-concurrent" => {
                let manifest_dir =
                    std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
                Path::new(&manifest_dir).join("test-project")
            }
            "test-project-diagnostics" => {
                let manifest_dir =
                    std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
                Path::new(&manifest_dir).join("test-project-diagnostics")
            }
            _ => return Err(anyhow::anyhow!("Unknown project type: {}", project_type)),
        };

        // Deterministic port based on project type
        let port = deterministic_port(project_type);
        let base_url = format!("http://127.0.0.1:{}", port);

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?;

        // Try to connect to existing server (with retries since another test may be starting it)
        for attempt in 0..30 {
            if let Ok(resp) = http_client
                .get(format!("{}/api/v1/health", base_url))
                .send()
                .await
            {
                if resp.status().is_success() {
                    if attempt > 0 {
                        eprintln!("Connected to HTTP server for {} on port {} (attempt {})", project_type, port, attempt + 1);
                    } else {
                        eprintln!("Connected to existing HTTP server for {} on port {}", project_type, port);
                    }
                    return Ok(Self {
                        http_client,
                        base_url,
                        port,
                        workspace_path,
                        _process: None,
                    });
                }
            }

            // Only try to start server on first attempt
            if attempt == 0 {
                // Check if the port is already in use (another test may be starting the server)
                let port_in_use = std::net::TcpStream::connect_timeout(
                    &format!("127.0.0.1:{}", port).parse().unwrap(),
                    Duration::from_millis(50),
                )
                .is_ok();

                if port_in_use {
                    // Port is bound but health check failed — server is still starting up
                    eprintln!("Port {} is in use, waiting for server to be ready...", port);
                } else {
                    // Port is free — start the server
                    eprintln!("Starting new HTTP server for {} on port {}", project_type, port);
                    if let Err(e) = start_server(&workspace_path, port) {
                        eprintln!("Failed to start server: {}", e);
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Err(anyhow::anyhow!(
            "Failed to connect to HTTP server after 15 seconds (port {})",
            port
        ))
    }

    /// Send a request to the server (backward compatible with MCP-style method names)
    pub async fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value> {
        match method {
            "tools/list" => {
                let resp = self
                    .http_client
                    .get(format!("{}/api/v1/tools", self.base_url))
                    .send()
                    .await?;
                let body: Value = resp.json().await?;
                if body["ok"].as_bool() == Some(true) {
                    Ok(body["result"].clone())
                } else {
                    Err(anyhow::anyhow!(
                        "Server error: {}",
                        body["error"].as_str().unwrap_or("unknown")
                    ))
                }
            }
            "tools/call" => {
                let params = params.unwrap_or(json!({}));
                let name = params["name"].as_str().unwrap_or("").to_string();
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
                self.call_tool(&name, arguments).await
            }
            "initialize" => {
                // No-op for HTTP server — it's already initialized
                Ok(json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": {
                        "name": "rust-analyzer-server",
                        "version": "0.3.0"
                    },
                    "capabilities": {
                        "tools": {}
                    }
                }))
            }
            _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
        }
    }

    /// Call a tool on the server.
    /// Returns a backward-compatible MCP ToolResult shape:
    /// `{"content": [{"type": "text", "text": "..."}]}`
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        let resp = self
            .http_client
            .post(format!("{}/api/v1/{}", self.base_url, name))
            .json(&arguments)
            .send()
            .await?;

        let status = resp.status();
        let body: Value = resp.json().await?;

        if body["ok"].as_bool() == Some(true) {
            // Wrap in MCP-compatible ToolResult format for backward compatibility
            let result = &body["result"];
            let text = if result.is_string() {
                result.as_str().unwrap().to_string()
            } else {
                serde_json::to_string_pretty(result)?
            };
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            }))
        } else {
            let error_msg = body["error"]
                .as_str()
                .unwrap_or("unknown error")
                .to_string();
            if status.is_server_error() || status.is_client_error() {
                Err(anyhow::anyhow!("{}", error_msg))
            } else {
                Err(anyhow::anyhow!("Server error: {}", error_msg))
            }
        }
    }

    /// Get the workspace path
    pub fn workspace_path(&self) -> &Path {
        &self.workspace_path
    }

    /// Get the server port
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        // Don't kill the process — it's shared across tests.
        // The server will shut down on its own when no longer needed.
    }
}

/// Start the server binary as a background process
fn start_server(workspace_path: &Path, port: u16) -> Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = Path::new(&manifest_dir);

    let release_binary = project_root.join("target/release/rust-analyzer-server");
    let debug_binary = project_root.join("target/debug/rust-analyzer-server");

    // Prefer the most recently built binary (debug is usually more up-to-date during development)
    let binary = match (debug_binary.exists(), release_binary.exists()) {
        (true, true) => {
            let debug_modified = std::fs::metadata(&debug_binary)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let release_modified = std::fs::metadata(&release_binary)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            if release_modified > debug_modified {
                release_binary
            } else {
                debug_binary
            }
        }
        (true, false) => debug_binary,
        (false, true) => release_binary,
        (false, false) => {
            return Err(anyhow::anyhow!(
                "rust-analyzer-server binary not found. Run `cargo build` first."
            ));
        }
    };

    // Use std::process::Command (not tokio) so the process is detached from the async runtime
    eprintln!("Spawning binary: {:?} --workspace {:?} --port {}", binary, workspace_path, port);
    std::process::Command::new(&binary)
        .arg("--workspace")
        .arg(workspace_path.to_str().unwrap())
        .arg("--port")
        .arg(port.to_string())
        .arg("--bind")
        .arg("127.0.0.1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

/// Deterministic port based on project type name hash
fn deterministic_port(project_type: &str) -> u16 {
    let hash: u32 = project_type
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    13000 + (hash % 1000) as u16
}
