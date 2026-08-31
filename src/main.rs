//! Entry point binary that wires the MCP upload server onto a hyperlane
//! [`Server`] and binds to the configured host:port.

use hyperlane_mcp_upload::{
    RequestConfig, Server, ServerAddress, ServerConfig, ServerControlHook, UploadConfig,
    format_bind, register,
};

#[tokio::main]
async fn main() {
    let address: ServerAddress = ServerAddress::default();
    let bind: String = format_bind(&address);

    let mut server_config: ServerConfig = ServerConfig::default();
    server_config.set_address(bind.clone());

    let mut server: Server = Server::default();
    server.server_config(server_config);
    server.request_config(RequestConfig::default());

    let _upload_config: UploadConfig = UploadConfig::default();

    register(&mut server);

    let control: ServerControlHook = server.run().await.unwrap_or_default();
    eprintln!("hyperlane-mcp-upload: listening on {bind}");
    control.wait().await;
}
