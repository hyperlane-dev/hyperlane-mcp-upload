use super::*;

/// Tool name registered with the MCP server.
pub const TOOL_UPLOAD_FILE: &str = "upload_file";

/// JSON Schema describing the `upload_file` tool's `file_path` argument.
pub fn upload_file_input_schema() -> serde_json::Value {
    let mut properties: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    properties.insert(
        "file_path".to_owned(),
        serde_json::json!({
            "type": "string",
            "description": "Absolute or relative path of the file to upload to ltpp.vip.",
        }),
    );
    let schema: serde_json::Value = serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": ["file_path"],
    });
    schema
}

/// Build the descriptor advertised by `tools/list` for the `upload_file` tool.
///
/// # Returns
///
/// - `ToolDescriptor`: Tool descriptor serialised into the MCP list response.
pub fn upload_file_descriptor() -> ToolDescriptor {
    let name: String = TOOL_UPLOAD_FILE.to_owned();
    let description: String =
        "Upload a local file to ltpp.vip and return the public URL.".to_owned();
    let input_schema: serde_json::Value = upload_file_input_schema();
    ToolDescriptor {
        name,
        description,
        input_schema,
    }
}

/// Build the `tools/list` payload returned to clients.
///
/// # Returns
///
/// - `serde_json::Value`: JSON object with a `tools` array.
pub fn tools_list_payload() -> serde_json::Value {
    let descriptor: ToolDescriptor = upload_file_descriptor();
    let payload: serde_json::Value = serde_json::json!({
        "tools": [descriptor],
    });
    payload
}

/// Dispatch a parsed JSON-RPC request to the matching MCP method.
///
/// # Arguments
///
/// - `request: &McpRequest` - Parsed JSON-RPC envelope.
/// - `config: &UploadConfig` - Upload configuration used by `tools/call`.
///
/// # Returns
///
/// - `serde_json::Value`: Either a `result` or `error` field per JSON-RPC 2.0.
pub fn dispatch(request: &McpRequest, config: &UploadConfig) -> serde_json::Value {
    match request.method.as_str() {
        "tools/list" => serde_json::json!({
            "result": tools_list_payload(),
        }),
        "tools/call" => handle_tools_call(&request.params, config),
        "initialize" => serde_json::json!({
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "hyperlane-mcp-upload",
                    "version": env!("CARGO_PKG_VERSION"),
                },
                "capabilities": {
                    "tools": {},
                },
            },
        }),
        _ => error_value(
            JsonRpcError::MethodNotFound,
            &format!("unknown method: {}", request.method),
        ),
    }
}

/// Handle a `tools/call` invocation by routing to the named tool.
///
/// # Arguments
///
/// - `params: &serde_json::Value` - Tool call parameters object.
/// - `config: &UploadConfig` - Upload configuration.
///
/// # Returns
///
/// - `serde_json::Value`: `result` or `error` field per JSON-RPC 2.0.
fn handle_tools_call(params: &serde_json::Value, config: &UploadConfig) -> serde_json::Value {
    let tool_name: String = match params
        .get("name")
        .and_then(|v: &serde_json::Value| v.as_str())
    {
        Some(name) => name.to_owned(),
        None => return error_value(JsonRpcError::InvalidParams, "missing tool name"),
    };
    let arguments: serde_json::Value = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    if tool_name == TOOL_UPLOAD_FILE {
        let raw_path: Option<&str> = arguments
            .get("file_path")
            .and_then(|v: &serde_json::Value| v.as_str());
        let file_path: String = match raw_path {
            Some(path) => path.to_owned(),
            None => {
                return error_value(
                    JsonRpcError::InvalidParams,
                    "upload_file requires string arguments.file_path",
                );
            }
        };
        return invoke_upload_file(&file_path, config);
    }
    error_value(
        JsonRpcError::MethodNotFound,
        &format!("unknown tool: {tool_name}"),
    )
}

/// Invoke the upload tool against a file path, returning an MCP-shaped result.
///
/// # Arguments
///
/// - `file_path: &str` - Local file path provided by the client.
/// - `config: &UploadConfig` - Upload configuration.
///
/// # Returns
///
/// - `serde_json::Value`: `result` or `error` field per JSON-RPC 2.0.
fn invoke_upload_file(file_path: &str, config: &UploadConfig) -> serde_json::Value {
    match upload_file(config, file_path) {
        Ok(result) => {
            let payload: serde_json::Value =
                serde_json::to_value(&result).unwrap_or_else(|err: serde_json::Error| {
                    serde_json::json!({
                        "error": format!("serialise result: {err}"),
                    })
                });
            let text: String = serde_json::to_string(&payload)
                .unwrap_or_else(|err: serde_json::Error| format!("{{\"error\":\"{err}\"}}"));
            let content: ToolCallContent = ToolCallContent {
                content: vec![ToolCallResult {
                    r#type: "text".to_owned(),
                    text,
                }],
            };
            serde_json::json!({
                "result": content,
            })
        }
        Err(err) => error_value(JsonRpcError::InternalError, &format!("{err}")),
    }
}

/// Build a JSON-RPC error value from a code + message.
///
/// # Arguments
///
/// - `code: JsonRpcError` - One of the standard codes.
/// - `message: &str` - Error message.
///
/// # Returns
///
/// - `serde_json::Value`: Object with `code` and `message` fields.
fn error_value(code: JsonRpcError, message: &str) -> serde_json::Value {
    serde_json::json!({
        "error": {
            "code": code.code(),
            "message": format!("{}: {}", code.message(), message),
        },
    })
}

/// Parse a raw JSON-RPC body, returning either a parsed request or a JSON-RPC
/// parse-error envelope.
///
/// # Arguments
///
/// - `body: &str` - Raw request body.
///
/// # Returns
///
/// - `Result<McpRequest, serde_json::Value>`: Parsed request or a JSON-RPC
///   parse-error value (with `id: null`).
pub fn parse_request(body: &str) -> Result<McpRequest, serde_json::Value> {
    serde_json::from_str::<McpRequest>(body)
        .map_err(|err: serde_json::Error| error_value(JsonRpcError::ParseError, &format!("{err}")))
}
