#[allow(unused_imports)]
use super::*;

/// Default host the MCP server binds to.
pub const MCP_DEFAULT_HOST: &str = "0.0.0.0";

/// Default port the MCP server listens on.
pub const MCP_DEFAULT_PORT: u16 = 7842;

/// Default chunk size in bytes used when uploading files to ltpp.vip.
pub const DEFAULT_CHUNK_SIZE: usize = 5 * 1024 * 1024;

/// Default host of the ltpp.vip upload service.
pub const LTPP_HOST: &str = "ltpp.vip";

/// Default URL path of the ltpp.vip upload service.
pub const LTPP_BASE_PATH: &str = "/api/upload";
