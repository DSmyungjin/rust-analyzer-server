use anyhow::Result;
use log::info;
use std::path::PathBuf;

use crate::lsp::RustAnalyzerClient;

pub struct RustAnalyzerMCPServer {
    pub(crate) client: Option<RustAnalyzerClient>,
    pub(crate) workspace_root: PathBuf,
}

impl Default for RustAnalyzerMCPServer {
    fn default() -> Self {
        Self::new()
    }
}

impl RustAnalyzerMCPServer {
    pub fn new() -> Self {
        Self {
            client: None,
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    pub fn with_workspace(workspace_root: PathBuf) -> Self {
        // Ensure the workspace root is absolute.
        let workspace_root = workspace_root.canonicalize().unwrap_or_else(|_| {
            // If canonicalize fails, try to make it absolute.
            if workspace_root.is_absolute() {
                workspace_root.clone()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(&workspace_root)
            }
        });

        Self {
            client: None,
            workspace_root,
        }
    }

    pub(crate) async fn ensure_client_started(&mut self) -> Result<()> {
        if self.client.is_none() {
            info!("Starting rust-analyzer for workspace: {}", self.workspace_root.display());
            let mut client = RustAnalyzerClient::new(self.workspace_root.clone());
            client.start().await?;
            self.client = Some(client);
        }
        Ok(())
    }

    pub(crate) async fn open_document_if_needed(&mut self, file_path: &str) -> Result<String> {
        let absolute_path = self.workspace_root.join(file_path);
        // Ensure we have an absolute path for the URI.
        let absolute_path = absolute_path
            .canonicalize()
            .unwrap_or_else(|_| absolute_path.clone());
        let uri = format!("file://{}", absolute_path.display());
        let content = tokio::fs::read_to_string(&absolute_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path, e))?;

        let Some(client) = &mut self.client else {
            return Err(anyhow::anyhow!("Client not initialized"));
        };

        client.open_document(&uri, &content).await?;
        Ok(uri)
    }

    pub async fn shutdown(&mut self) {
        info!("Shutting down rust-analyzer");
        if let Some(client) = &mut self.client {
            let _ = client.shutdown().await;
        }
    }
}
