// The IPC Unix socket server is no longer needed.
// The main binary now runs as a standalone HTTP server.
// This module is kept for backward compatibility but does nothing.

pub fn socket_path(_project_type: &str) -> std::path::PathBuf {
    std::path::PathBuf::new()
}
