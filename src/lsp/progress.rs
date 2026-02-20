use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEntry {
    pub token: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<u32>,
}

#[derive(Debug)]
pub struct ProgressState {
    active: HashMap<String, ProgressEntry>,
}

impl ProgressState {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
        }
    }

    pub fn begin(
        &mut self,
        token: String,
        title: String,
        message: Option<String>,
        percentage: Option<u32>,
    ) {
        self.active.insert(
            token.clone(),
            ProgressEntry {
                token,
                title,
                message,
                percentage,
            },
        );
    }

    pub fn report(&mut self, token: &str, message: Option<String>, percentage: Option<u32>) {
        if let Some(entry) = self.active.get_mut(token) {
            entry.message = message;
            entry.percentage = percentage;
        }
    }

    pub fn end(&mut self, token: &str) {
        self.active.remove(token);
    }

    pub fn is_indexing(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn active_tasks(&self) -> Vec<ProgressEntry> {
        self.active.values().cloned().collect()
    }
}

pub type SharedProgress = Arc<Mutex<ProgressState>>;

pub fn new_shared_progress() -> SharedProgress {
    Arc::new(Mutex::new(ProgressState::new()))
}
