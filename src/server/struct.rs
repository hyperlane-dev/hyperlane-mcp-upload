#[allow(unused_imports)]
use super::*;

/// Service handler for the MCP JSON-RPC endpoint mounted at `/mcp`.
pub struct McpHandler;

/// Service handler for a basic health check mounted at `/health`.
pub struct HealthHandler;

/// Service handler returning the 404 page for unmatched routes.
pub struct NotFoundHandler;

/// Service handler that logs task panics and keeps the server alive.
pub struct PanicHandler;

/// Service handler that turns request errors into JSON responses.
pub struct RequestErrorHandler;

/// Service handler that records the request method + path on every inbound
/// request.
pub struct RequestLogMiddleware;

/// Service handler that records the response status on every outbound
/// response.
pub struct ResponseLogMiddleware;
