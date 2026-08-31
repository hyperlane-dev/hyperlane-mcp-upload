use super::*;

/// JSON-RPC 2.0 envelope for an inbound MCP request.
#[derive(Clone, Debug, Deserialize)]
pub struct McpRequest {
    /// Protocol version; must equal `"2.0"`.
    pub jsonrpc: String,
    /// Identifier echoed back in the response.
    pub id: serde_json::Value,
    /// Method name (e.g. `tools/list`, `tools/call`).
    pub method: String,
    /// Method-specific parameters.
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 success response.
#[derive(Clone, Debug, Serialize)]
pub struct McpResponse {
    /// Protocol version; always `"2.0"`.
    pub jsonrpc: String,
    /// Identifier echoing the inbound request.
    pub id: serde_json::Value,
    /// Result payload on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 error response.
#[derive(Clone, Debug, Serialize)]
pub struct McpErrorResponse {
    /// Protocol version; always `"2.0"`.
    pub jsonrpc: String,
    /// Identifier echoing the inbound request.
    pub id: serde_json::Value,
    /// Error payload describing the failure.
    pub error: McpErrorBody,
}

/// JSON-RPC 2.0 error object body.
#[derive(Clone, Debug, Serialize)]
pub struct McpErrorBody {
    /// Numeric error code per JSON-RPC 2.0 spec.
    pub code: i32,
    /// Short human-readable error message.
    pub message: String,
}

/// Description of an MCP tool exposed to clients.
#[derive(Clone, Debug, Serialize)]
pub struct ToolDescriptor {
    /// Tool name (clients use this to invoke).
    pub name: String,
    /// Human-readable description of the tool.
    pub description: String,
    /// JSON Schema for the tool's input parameters.
    pub input_schema: serde_json::Value,
}

/// Result content of a single tool call.
#[derive(Clone, Debug, Serialize)]
pub struct ToolCallResult {
    /// Always `"text"` for textual payloads.
    /// `text` payload for the tool result.
    pub r#type: String,
    /// Payload string (JSON-encoded for structured data).
    pub text: String,
}

/// Wrap a JSON value in the MCP `content` array shape.
#[derive(Clone, Debug, Serialize)]
pub struct ToolCallContent {
    /// Always contains exactly one text element.
    pub content: Vec<ToolCallResult>,
}

/// Standard JSON-RPC error codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JsonRpcError {
    /// Invalid JSON was received by the server.
    ParseError,
    /// The JSON sent is not a valid Request object.
    InvalidRequest,
    /// The method does not exist or is not available.
    MethodNotFound,
    /// Invalid method parameter(s).
    InvalidParams,
    /// Internal JSON-RPC error.
    InternalError,
}

impl JsonRpcError {
    /// Numeric code per JSON-RPC 2.0 specification.
    pub fn code(&self) -> i32 {
        match self {
            Self::ParseError => -32700,
            Self::InvalidRequest => -32600,
            Self::MethodNotFound => -32601,
            Self::InvalidParams => -32602,
            Self::InternalError => -32603,
        }
    }

    /// Short message associated with the error code.
    pub fn message(&self) -> &'static str {
        match self {
            Self::ParseError => "Parse error",
            Self::InvalidRequest => "Invalid Request",
            Self::MethodNotFound => "Method not found",
            Self::InvalidParams => "Invalid params",
            Self::InternalError => "Internal error",
        }
    }
}
