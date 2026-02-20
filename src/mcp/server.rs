use anyhow::Result;
use log::info;
use std::path::PathBuf;

use crate::lsp::progress::ProgressEntry;
use crate::lsp::RustAnalyzerClient;

/// Tracks why the server is in its current state.
#[derive(Debug, Clone, PartialEq)]
pub enum InitTrigger {
    /// Fresh server start, no workspace set yet.
    None,
    /// First client start via tool call or CLI.
    InitialStart,
    /// Workspace was changed to a different path.
    WorkspaceChange { previous: PathBuf },
}

pub struct RustAnalyzerMCPServer {
    pub(crate) client: Option<RustAnalyzerClient>,
    pub(crate) workspace_root: PathBuf,
    pub(crate) init_trigger: InitTrigger,
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
            init_trigger: InitTrigger::None,
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
            init_trigger: InitTrigger::None,
        }
    }

    pub(crate) async fn ensure_client_started(&mut self) -> Result<()> {
        if self.client.is_none() {
            // Validate workspace path exists.
            if !self.workspace_root.exists() {
                return Err(anyhow::anyhow!(
                    "Workspace path does not exist: {}",
                    self.workspace_root.display()
                ));
            }
            if self.init_trigger == InitTrigger::None {
                self.init_trigger = InitTrigger::InitialStart;
            }
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

    pub async fn is_indexing(&self) -> bool {
        match &self.client {
            Some(client) => client.progress.lock().await.is_indexing(),
            None => false,
        }
    }

    pub async fn active_progress(&self) -> Vec<ProgressEntry> {
        match &self.client {
            Some(client) => client.progress.lock().await.active_tasks(),
            None => vec![],
        }
    }

    pub fn trigger_info(&self) -> (&str, Option<String>) {
        match &self.init_trigger {
            InitTrigger::None => ("none", None),
            InitTrigger::InitialStart => ("initial_start", None),
            InitTrigger::WorkspaceChange { previous } => {
                ("workspace_change", Some(previous.display().to_string()))
            }
        }
    }

    pub fn workspace_exists(&self) -> bool {
        self.workspace_root.exists()
    }

    pub async fn shutdown(&mut self) {
        info!("Shutting down rust-analyzer");
        if let Some(client) = &mut self.client {
            let _ = client.shutdown().await;
        }
    }
}
