/// Timeout for LSP requests in seconds.
pub const LSP_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Delay after opening a document to allow rust-analyzer to process it.
/// Increased from 200ms to 1000ms to support large files with complex types.
pub const DOCUMENT_OPEN_DELAY_MILLIS: u64 = 1000;
