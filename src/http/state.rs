use std::sync::Arc;
use tokio::sync::{watch, Mutex};

use crate::RustAnalyzerMCPServer;

#[derive(Clone)]
pub struct AppState {
    pub server: Arc<Mutex<RustAnalyzerMCPServer>>,
    pub shutdown_tx: watch::Sender<bool>,
}
