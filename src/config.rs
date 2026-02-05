/// Timeout for LSP requests in seconds.
pub const LSP_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Delay after opening a document to allow rust-analyzer to process it.
/// Increased from 200ms to 1000ms to support large files with complex types.
pub const DOCUMENT_OPEN_DELAY_MILLIS: u64 = 1000;

/// Timeout for tool calls that may need to wait for indexing to complete.
/// Detects CI environment and uses longer timeout if needed.
pub fn get_indexing_timeout_secs() -> u64 {
    if std::env::var("CI").is_ok() {
        30 // Longer timeout in CI
    } else {
        15 // Shorter timeout for local development
    }
}

/// Interval between retry attempts when waiting for indexing.
pub const RETRY_INTERVAL_MILLIS: u64 = 500;
