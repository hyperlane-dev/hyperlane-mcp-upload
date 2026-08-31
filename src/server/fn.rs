use super::*;

/// Register every route, middleware, panic, and error handler on the given
/// hyperlane `Server`. This is the single entry point used by `main`.
///
/// # Arguments
///
/// - `server: &mut hyperlane::Server` - The hyperlane server builder to
///   populate.
pub fn register(server: &mut Server) {
    server.route::<McpHandler>("/mcp");
    server.route::<HealthHandler>("/health");
    server.route::<NotFoundHandler>("/*");
    server.task_panic::<PanicHandler>();
    server.request_error::<RequestErrorHandler>();
    server.request_middleware::<RequestLogMiddleware>();
    server.response_middleware::<ResponseLogMiddleware>();
}

/// Build the JSON-RPC response body for a request envelope.
///
/// # Arguments
///
/// - `body: &str` - Raw request body.
/// - `config: &UploadConfig` - Upload configuration.
///
/// # Returns
///
/// - `String`: JSON-serialised JSON-RPC response.
pub fn handle_mcp_body(body: &str, config: &UploadConfig) -> String {
    match parse_request(body) {
        Ok(request) => {
            let payload: serde_json::Value = dispatch(&request, config);
            let id: serde_json::Value = request.id;
            let mut envelope: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            envelope.insert(
                "jsonrpc".to_owned(),
                serde_json::Value::String("2.0".to_owned()),
            );
            envelope.insert("id".to_owned(), id);
            if let Some(result) = payload.get("result") {
                envelope.insert("result".to_owned(), result.clone());
            }
            if let Some(error) = payload.get("error") {
                envelope.insert("error".to_owned(), error.clone());
            }
            serde_json::to_string(&serde_json::Value::Object(envelope))
                .unwrap_or_else(|_: serde_json::Error| {
                    r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize failure"}}"#
                        .to_owned()
                })
        }
        Err(error) => {
            let mut envelope: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            envelope.insert(
                "jsonrpc".to_owned(),
                serde_json::Value::String("2.0".to_owned()),
            );
            envelope.insert("id".to_owned(), serde_json::Value::Null);
            if let Some(err) = error.get("error") {
                envelope.insert("error".to_owned(), err.clone());
            }
            serde_json::to_string(&serde_json::Value::Object(envelope))
                .unwrap_or_else(|_: serde_json::Error| {
                    r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize failure"}}"#
                        .to_owned()
                })
        }
    }
}

/// Read the shared upload config from the `Context`'s attribute store.
///
/// # Arguments
///
/// - `ctx: &Context` - Per-request context.
///
/// # Returns
///
/// - `UploadConfig`: Upload configuration stored under the `upload_config`
///   key, falling back to defaults when not set.
pub fn load_upload_config(ctx: &Context) -> UploadConfig {
    if let Some(cfg) = ctx.try_get_attribute::<UploadConfig>(UPLOAD_CONFIG_ATTR) {
        return cfg.clone();
    }
    UploadConfig::default()
}

/// Build the host:port bind address string for the server.
///
/// # Arguments
///
/// - `address: &ServerAddress` - Server bind configuration.
///
/// # Returns
///
/// - `String`: Formatted `host:port` bind string.
pub fn format_bind(address: &ServerAddress) -> String {
    Server::format_bind_address(address.host.as_str(), address.port)
}
