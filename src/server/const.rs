#[allow(unused_imports)]
use super::*;

/// Attribute key used to stash the upload config in the request context.
pub const UPLOAD_CONFIG_ATTR: &str = "mcp_upload_config";

/// Common response header for JSON content.
pub const HEADER_CONTENT_TYPE: &str = "Content-Type";

/// JSON content type value used for MCP responses.
pub const CONTENT_TYPE_JSON: &str = "application/json";
