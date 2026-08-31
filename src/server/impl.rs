use super::*;

/// Handler that serves the JSON-RPC endpoint at `/mcp`.
impl ServerHook for McpHandler {
    async fn new(_stream: &mut Stream, _ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, stream: &mut Stream, ctx: &mut Context) -> Status {
        let config: UploadConfig = UploadConfig::default();
        let body: String = String::from_utf8_lossy(&ctx.get_request().body).into_owned();
        let response: String = handle_mcp_body(&body, &config);
        let data: Vec<u8> = ctx
            .get_mut_response()
            .set_status_code(200)
            .set_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON)
            .set_body(response)
            .build();
        let _ = stream.try_send(data).await;
        Status::Continue
    }
}

/// Handler that responds to `/health` with a static OK payload.
impl ServerHook for HealthHandler {
    async fn new(_stream: &mut Stream, _ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, stream: &mut Stream, ctx: &mut Context) -> Status {
        let body: String = "{\"status\":\"ok\"}".to_owned();
        let data: Vec<u8> = ctx
            .get_mut_response()
            .set_status_code(200)
            .set_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON)
            .set_body(body)
            .build();
        let _ = stream.try_send(data).await;
        Status::Continue
    }
}

/// Handler that returns a plain-text 404 for unmatched routes.
impl ServerHook for NotFoundHandler {
    async fn new(_stream: &mut Stream, _ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, stream: &mut Stream, ctx: &mut Context) -> Status {
        let body: String = "404 not found".to_owned();
        let data: Vec<u8> = ctx
            .get_mut_response()
            .set_status_code(404)
            .set_header(HEADER_CONTENT_TYPE, "text/plain; charset=utf-8")
            .set_body(body)
            .build();
        let _ = stream.try_send(data).await;
        Status::Continue
    }
}

/// Hook that recovers from background-task panics and logs them.
impl ServerHook for PanicHandler {
    async fn new(_stream: &mut Stream, _ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, _stream: &mut Stream, _ctx: &mut Context) -> Status {
        eprintln!("hyperlane-mcp-upload: task panic captured");
        Status::Continue
    }
}

/// Hook that converts request errors into a JSON error envelope.
impl ServerHook for RequestErrorHandler {
    async fn new(_stream: &mut Stream, _ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, stream: &mut Stream, ctx: &mut Context) -> Status {
        let error: RequestError = ctx.try_get_request_error_data().unwrap_or_default();
        let status_code: ResponseStatusCode = error.get_http_status_code();
        let message: String = error.to_string();
        let body: String = format!(
            r#"{{"error":"{}","status":{}}}"#,
            message.replace('"', "'"),
            status_code as u16,
        );
        let data: Vec<u8> = ctx
            .get_mut_response()
            .set_status_code(status_code)
            .set_header(HEADER_CONTENT_TYPE, CONTENT_TYPE_JSON)
            .set_body(body)
            .build();
        let _ = stream.try_send(data).await;
        Status::Continue
    }
}

/// Middleware that records the inbound request method + path to stderr.
impl ServerHook for RequestLogMiddleware {
    async fn new(_stream: &mut Stream, _ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, _stream: &mut Stream, ctx: &mut Context) -> Status {
        let method: String = format!("{:?}", ctx.get_request().method);
        let path: String = ctx.get_request().path.clone();
        eprintln!("hyperlane-mcp-upload: -> {method} {path}");
        Status::Continue
    }
}

/// Middleware that records the outbound response status code to stderr.
impl ServerHook for ResponseLogMiddleware {
    async fn new(_stream: &mut Stream, _ctx: &mut Context) -> Self {
        Self
    }

    async fn handle(self, _stream: &mut Stream, ctx: &mut Context) -> Status {
        let status_code: ResponseStatusCode = ctx.get_response().get_status_code();
        eprintln!("hyperlane-mcp-upload: <- {}", status_code as u16);
        Status::Continue
    }
}
