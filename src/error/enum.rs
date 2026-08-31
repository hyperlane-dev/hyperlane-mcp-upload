#[allow(unused_imports)]
use super::*;

/// Errors that can occur while serving MCP requests or uploading files.
#[derive(Clone, Debug, thiserror::Error)]
pub enum McpError {
    /// The provided file path does not exist on disk.
    #[error("file not found: {0}")]
    FileNotFound(String),
    /// Failed to read the local file from disk.
    #[error("failed to read file: {0}")]
    FileRead(String),
    /// Failed to serialize or deserialize a JSON payload.
    #[error("json error: {0}")]
    Json(String),
    /// The ltpp.vip upload service returned a non-success status code.
    #[error("upload service returned status {status}: {message}")]
    UploadStatus {
        /// HTTP status code returned by the upload service.
        status: u16,
        /// Error message returned by the upload service.
        message: String,
    },
    /// A network or transport-level error occurred while talking to ltpp.vip.
    #[error("network error: {0}")]
    Network(String),
    /// The MCP request payload did not match the expected schema.
    #[error("invalid mcp request: {0}")]
    InvalidRequest(String),
    /// The requested tool is not registered with this MCP server.
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    /// Internal I/O failure.
    #[error("io error: {0}")]
    Io(String),
}

/// Convenience alias for `Result<T, McpError>`.
pub type McpResult<T> = Result<T, McpError>;
