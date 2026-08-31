#[allow(unused_imports)]
use super::*;

/// Server bind address (host:port).
#[derive(Clone, Debug)]
pub struct ServerAddress {
    /// IP address the server listens on.
    pub host: String,
    /// TCP port the server listens on.
    pub port: u16,
}

impl Default for ServerAddress {
    /// Returns the default server address (`0.0.0.0:7842`).
    fn default() -> Self {
        let host: String = MCP_DEFAULT_HOST.to_owned();
        let port: u16 = MCP_DEFAULT_PORT;
        Self { host, port }
    }
}
