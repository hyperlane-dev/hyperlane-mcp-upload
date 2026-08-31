use super::*;

/// Build an ltpp.vip endpoint URL from the config plus a relative path.
///
/// # Arguments
///
/// - `config: &UploadConfig` - Upload configuration holding host and base path.
/// - `suffix: &str` - Path appended to `config.base_path` (e.g. `/register`).
///
/// # Returns
///
/// - `String`: Fully-qualified `https://<host><base_path><suffix>` URL.
pub fn build_endpoint(config: &UploadConfig, suffix: &str) -> String {
    let trimmed: &str = suffix.trim_start_matches('/');
    let base: &str = config.base_path.trim_end_matches('/');
    let host: &str = config.host.as_str();
    let mut url: String = String::from("https://");
    url.push_str(host);
    url.push_str(base);
    url.push('/');
    url.push_str(trimmed);
    url
}

/// Upload a single local file to ltpp.vip using the three-step
/// register/save/merge protocol executed via `curl`.
///
/// # Arguments
///
/// - `config: &UploadConfig` - Upload configuration.
/// - `file_path: &str` - Absolute or relative path of the file to upload.
///
/// # Returns
///
/// - `McpResult<UploadResult>`: Final public URL plus file metadata, or an error.
///
/// # Errors
///
/// Returns `McpError::FileNotFound` when the path does not exist,
/// `McpError::FileRead` when the file cannot be read,
/// `McpError::Network` when `curl` itself fails to launch,
/// `McpError::UploadStatus` when ltpp.vip returns a non-2xx response.
pub fn upload_file(config: &UploadConfig, file_path: &str) -> McpResult<UploadResult> {
    let resolved: std::path::PathBuf = resolve_path(file_path)?;
    let bytes: Vec<u8> = std::fs::read(&resolved).map_err(|err: std::io::Error| {
        McpError::FileRead(format!("{}: {err}", resolved.display()))
    })?;
    let size: u64 = bytes.len() as u64;
    let file_name: String = file_name_of(&resolved);
    let file_id: String = format!(
        "upload_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d: std::time::Duration| d.as_millis())
            .unwrap_or(0),
        std::process::id()
    );

    let chunk_size: usize = config.chunk_size;
    let total_chunks: usize = if size == 0 {
        1
    } else {
        (size as usize).div_ceil(chunk_size)
    };
    let hash: String = md5_hex(&bytes)?;

    register_chunked(
        config,
        &file_id,
        &file_name,
        size,
        chunk_size,
        total_chunks,
        &hash,
    )?;
    save_chunks(config, &file_id, &file_name, &bytes)?;
    let url_path: String = merge_chunks(config, &file_id, &file_name, total_chunks, &hash)?;

    let url: String = format!("https://{}{}", config.host, url_path);
    let result: UploadResult = UploadResult {
        url,
        file_name,
        size,
    };
    Ok(result)
}

/// Resolve `file_path` against the current working directory and verify the
/// file exists.
///
/// # Arguments
///
/// - `file_path: &str` - User-provided path (absolute or relative).
///
/// # Returns
///
/// - `McpResult<std::path::PathBuf>`: Canonical path on success.
///
/// # Errors
///
/// Returns `McpError::FileNotFound` when no file exists at the resolved path.
pub fn resolve_path(file_path: &str) -> McpResult<std::path::PathBuf> {
    let candidate: std::path::PathBuf = std::path::PathBuf::from(file_path);
    let resolved: std::path::PathBuf = if candidate.is_absolute() {
        candidate
    } else {
        let cwd: std::path::PathBuf = std::env::current_dir().map_err(|err: std::io::Error| {
            McpError::Io(format!("cannot read current directory: {err}"))
        })?;
        cwd.join(candidate)
    };
    if !resolved.exists() {
        let path_str: String = resolved.display().to_string();
        return Err(McpError::FileNotFound(path_str));
    }
    Ok(resolved)
}

/// Extract the base file name from a path, falling back to `upload.bin`.
///
/// # Arguments
///
/// - `path: &std::path::Path` - Path to extract the name from.
///
/// # Returns
///
/// - `String`: Base name of the file, never empty.
pub fn file_name_of(path: &std::path::Path) -> String {
    let raw: std::ffi::OsString = path
        .file_name()
        .map(|n: &std::ffi::OsStr| n.to_os_string())
        .unwrap_or_else(|| std::ffi::OsString::from("upload.bin"));
    raw.to_string_lossy().into_owned()
}

/// Compute the lowercase MD5 hex digest of a byte slice by shelling out to
/// the system `md5sum` command.
///
/// # Arguments
///
/// - `bytes: &[u8]` - Input bytes.
///
/// # Returns
///
/// - `McpResult<String>`: 32-character lowercase hex digest.
///
/// # Errors
///
/// Returns `McpError::Network` when `md5sum` is missing or fails.
pub fn md5_hex(bytes: &[u8]) -> McpResult<String> {
    use std::io::Write;
    let mut child: std::process::Child = std::process::Command::new("md5sum")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|err: std::io::Error| McpError::Network(format!("md5sum spawn: {err}")))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(bytes)
            .map_err(|err: std::io::Error| McpError::Network(format!("md5sum stdin: {err}")))?;
    }
    let output: std::process::Output = child
        .wait_with_output()
        .map_err(|err: std::io::Error| McpError::Network(format!("md5sum wait: {err}")))?;
    if !output.status.success() {
        let stderr_text: String = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(McpError::Network(format!("md5sum failed: {stderr_text}")));
    }
    let stdout_text: String = String::from_utf8_lossy(&output.stdout).into_owned();
    let digest: String = stdout_text
        .split_whitespace()
        .next()
        .map(|s: &str| s.to_owned())
        .unwrap_or_default();
    Ok(digest)
}

/// Step 1 of the upload protocol: register the file with ltpp.vip.
///
/// # Arguments
///
/// - `config: &UploadConfig` - Upload configuration.
/// - `file_id: &str` - Client-generated unique upload id.
/// - `file_name: &str` - Name of the file being uploaded.
/// - `size: u64` - Total file size in bytes.
/// - `chunk_size: usize` - Size of each chunk.
/// - `total_chunks: usize` - Total chunk count.
/// - `hash: &str` - MD5 hex digest of the file body.
///
/// # Returns
///
/// - `McpResult<()>`: Success when the server accepts the registration.
///
/// # Errors
///
/// Returns `McpError::UploadStatus` if the server rejects the registration,
/// `McpError::Network` if `curl` itself fails.
fn register_chunked(
    config: &UploadConfig,
    file_id: &str,
    file_name: &str,
    size: u64,
    chunk_size: usize,
    total_chunks: usize,
    hash: &str,
) -> McpResult<()> {
    let url: String = build_endpoint(config, "register");
    let body: String = format!(
        r#"{{"fileName":"{file_name}","fileSize":{size},"fileHash":"{hash}","chunkSize":{chunk_size},"totalChunks":{total_chunks}}}"#
    );
    run_curl_void(
        &url,
        &[
            ("X-File-Id", file_id),
            ("X-File-Name", file_name),
            ("X-Total-Chunks", &total_chunks.to_string()),
            ("Content-Type", "application/json"),
        ],
        body.as_bytes(),
    )?;
    Ok(())
}

/// Step 2 of the upload protocol: send the file body in chunks.
///
/// # Arguments
///
/// - `config: &UploadConfig` - Upload configuration.
/// - `file_id: &str` - Upload id from step 1.
/// - `file_name: &str` - Original file name.
/// - `bytes: &[u8]` - File bytes.
///
/// # Returns
///
/// - `McpResult<()>`: Success when all chunks are accepted.
///
/// # Errors
///
/// Returns `McpError::UploadStatus` if any chunk upload is rejected.
fn save_chunks(
    config: &UploadConfig,
    file_id: &str,
    file_name: &str,
    bytes: &[u8],
) -> McpResult<()> {
    let url: String = build_endpoint(config, "save");
    let chunk_size: usize = config.chunk_size;
    let total: usize = bytes.len();
    if total == 0 {
        let index_str: String = "0".to_owned();
        let temp_path: std::path::PathBuf = write_temp_chunk(file_id, b"")?;
        run_curl_octet(
            &url,
            &[
                ("X-File-Id", file_id),
                ("X-File-Name", file_name),
                ("X-Chunk-Index", index_str.as_str()),
                ("X-Chunk-Size", "0"),
            ],
            temp_path
                .to_str()
                .ok_or_else(|| McpError::Io("temp chunk path is not valid utf-8".to_owned()))?,
        )?;
        let _ = std::fs::remove_file(&temp_path);
        return Ok(());
    }
    let mut offset: usize = 0;
    let mut index: usize = 0;
    while offset < total {
        let end: usize = if offset + chunk_size > total {
            total
        } else {
            offset + chunk_size
        };
        let slice: &[u8] = &bytes[offset..end];
        let temp_path: std::path::PathBuf = write_temp_chunk(file_id, slice)?;
        let index_str: String = index.to_string();
        let chunk_size_str: String = slice.len().to_string();
        run_curl_octet(
            &url,
            &[
                ("X-File-Id", file_id),
                ("X-File-Name", file_name),
                ("X-Chunk-Index", index_str.as_str()),
                ("X-Chunk-Size", chunk_size_str.as_str()),
            ],
            temp_path
                .to_str()
                .ok_or_else(|| McpError::Io("temp chunk path is not valid utf-8".to_owned()))?,
        )?;
        let _ = std::fs::remove_file(&temp_path);
        offset = end;
        index = index.saturating_add(1);
    }
    Ok(())
}

/// Step 3 of the upload protocol: ask ltpp.vip to merge the chunks.
///
/// # Arguments
///
/// - `config: &UploadConfig` - Upload configuration.
/// - `file_id: &str` - Upload id.
/// - `file_name: &str` - Original file name.
/// - `total_chunks: usize` - Total chunk count.
/// - `hash: &str` - MD5 hex digest of the file body.
///
/// # Returns
///
/// - `McpResult<String>`: Server-side URL path (starting with `/upload/...`).
///
/// # Errors
///
/// Returns `McpError::UploadStatus` if merge fails or the response lacks a URL.
fn merge_chunks(
    config: &UploadConfig,
    file_id: &str,
    file_name: &str,
    total_chunks: usize,
    hash: &str,
) -> McpResult<String> {
    let url: String = build_endpoint(config, "merge");
    let body: String =
        format!(r#"{{"fileName":"{file_name}","fileHash":"{hash}","totalChunks":{total_chunks}}}"#);
    let resp: String = run_curl_capture(
        &url,
        &[
            ("X-File-Id", file_id),
            ("X-File-Name", file_name),
            ("Content-Type", "application/json"),
        ],
        body.as_bytes(),
    )?;
    let url_path: String = parse_url_from_merge(&resp)?;
    Ok(url_path)
}

/// Extract the `url` field from a ltpp.vip merge response.
///
/// # Arguments
///
/// - `body: &str` - Raw JSON body returned by the merge endpoint.
///
/// # Returns
///
/// - `McpResult<String>`: URL path stored under the `url` key.
///
/// # Errors
///
/// Returns `McpError::UploadStatus` if the JSON has no usable URL field.
fn parse_url_from_merge(body: &str) -> McpResult<String> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|err: serde_json::Error| McpError::Json(format!("merge response: {err}")))?;
    let url_path: String = value
        .get("url")
        .and_then(|v: &serde_json::Value| v.as_str())
        .map(|s: &str| s.to_owned())
        .ok_or_else(|| McpError::UploadStatus {
            status: 200,
            message: format!("merge response missing url field: {body}"),
        })?;
    Ok(url_path)
}

/// Write a temporary file with the given bytes; used to ship chunk bodies to
/// `curl` via `@path` syntax.
///
/// # Arguments
///
/// - `file_id: &str` - File id used to make the temp file name unique.
/// - `bytes: &[u8]` - Chunk body to write.
///
/// # Returns
///
/// - `McpResult<std::path::PathBuf>`: Path of the temp file written.
///
/// # Errors
///
/// Returns `McpError::Io` on filesystem failure.
fn write_temp_chunk(file_id: &str, bytes: &[u8]) -> McpResult<std::path::PathBuf> {
    let safe_id: String = file_id
        .chars()
        .map(|c: char| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let pid: u32 = std::process::id();
    let file_name: String = format!("hyperlane_mcp_upload_{pid}_{safe_id}");
    let dir: std::path::PathBuf = std::env::temp_dir();
    let path: std::path::PathBuf = dir.join(file_name);
    std::fs::write(&path, bytes)
        .map_err(|err: std::io::Error| McpError::Io(format!("write temp chunk: {err}")))?;
    Ok(path)
}

/// Invoke `curl` and assert a 2xx response, returning the response body.
///
/// # Arguments
///
/// - `url: &str` - Fully-qualified target URL.
/// - `headers: &[(&str, &str)]` - Custom headers to attach.
/// - `body: &[u8]` - Optional request body; empty means GET.
///
/// # Returns
///
/// - `McpResult<String>`: Response body string.
///
/// # Errors
///
/// Returns `McpError::Network` when curl cannot be launched,
/// `McpError::UploadStatus` when the HTTP status is non-2xx.
fn run_curl_capture(url: &str, headers: &[(&str, &str)], body: &[u8]) -> McpResult<String> {
    let mut command: std::process::Command = std::process::Command::new("curl");
    command
        .arg("--silent")
        .arg("--show-error")
        .arg("--location")
        .arg("--fail-with-body")
        .arg("--proto")
        .arg("=https")
        .arg(url);
    for (name, value) in headers {
        command.arg("--header").arg(format!("{name}: {value}"));
    }
    if !body.is_empty() {
        command.arg("--data-binary").arg("@-");
    }
    let output: std::process::Output = if body.is_empty() {
        command
            .output()
            .map_err(|err: std::io::Error| McpError::Network(format!("curl: {err}")))?
    } else {
        use std::io::Write;
        let mut child: std::process::Child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|err: std::io::Error| McpError::Network(format!("curl spawn: {err}")))?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(body)
                .map_err(|err: std::io::Error| McpError::Network(format!("curl stdin: {err}")))?;
        }
        child
            .wait_with_output()
            .map_err(|err: std::io::Error| McpError::Network(format!("curl wait: {err}")))?
    };
    if !output.status.success() {
        let code: i32 = output.status.code().unwrap_or(-1);
        let stderr_text: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let status: u16 = u16::try_from(code).unwrap_or(599);
        return Err(McpError::UploadStatus {
            status,
            message: stderr_text,
        });
    }
    let body_text: String = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(body_text)
}

/// Invoke `curl` to POST a body, discarding the response body but asserting
/// success.
///
/// # Arguments
///
/// - `url: &str` - Fully-qualified target URL.
/// - `headers: &[(&str, &str)]` - Custom headers to attach.
/// - `body: &[u8]` - Request body bytes; piped via stdin.
///
/// # Returns
///
/// - `McpResult<()>`: Success when the request returns 2xx.
///
/// # Errors
///
/// Returns `McpError::Network` or `McpError::UploadStatus` on failure.
fn run_curl_void(url: &str, headers: &[(&str, &str)], body: &[u8]) -> McpResult<()> {
    let mut command: std::process::Command = std::process::Command::new("curl");
    command
        .arg("--silent")
        .arg("--show-error")
        .arg("--location")
        .arg("--fail")
        .arg("--proto")
        .arg("=https")
        .arg(url);
    for (name, value) in headers {
        command.arg("--header").arg(format!("{name}: {value}"));
    }
    command.arg("--data-binary").arg("@-");
    use std::io::Write;
    let mut child: std::process::Child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|err: std::io::Error| McpError::Network(format!("curl spawn: {err}")))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(body)
            .map_err(|err: std::io::Error| McpError::Network(format!("curl stdin: {err}")))?;
    }
    let output: std::process::Output = child
        .wait_with_output()
        .map_err(|err: std::io::Error| McpError::Network(format!("curl wait: {err}")))?;
    if !output.status.success() {
        let stderr_text: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let code: i32 = output.status.code().unwrap_or(-1);
        let status: u16 = u16::try_from(code).unwrap_or(599);
        return Err(McpError::UploadStatus {
            status,
            message: stderr_text,
        });
    }
    Ok(())
}

/// Invoke `curl` to POST the contents of a file with octet-stream semantics.
///
/// # Arguments
///
/// - `url: &str` - Fully-qualified target URL.
/// - `headers: &[(&str, &str)]` - Custom headers to attach.
/// - `file_path: &str` - Path whose contents form the request body.
///
/// # Returns
///
/// - `McpResult<()>`: Success when the request returns 2xx.
///
/// # Errors
///
/// Returns `McpError::Network` or `McpError::UploadStatus` on failure.
fn run_curl_octet(url: &str, headers: &[(&str, &str)], file_path: &str) -> McpResult<()> {
    let mut command: std::process::Command = std::process::Command::new("curl");
    command
        .arg("--silent")
        .arg("--show-error")
        .arg("--location")
        .arg("--fail")
        .arg("--proto")
        .arg("=https")
        .arg("--header")
        .arg("Content-Type: application/octet-stream")
        .arg(url);
    for (name, value) in headers {
        command.arg("--header").arg(format!("{name}: {value}"));
    }
    command.arg("--data-binary").arg(format!("@{file_path}"));
    let output: std::process::Output = command
        .output()
        .map_err(|err: std::io::Error| McpError::Network(format!("curl: {err}")))?;
    if !output.status.success() {
        let code: i32 = output.status.code().unwrap_or(-1);
        let stderr_text: String = String::from_utf8_lossy(&output.stderr).into_owned();
        let status: u16 = u16::try_from(code).unwrap_or(599);
        return Err(McpError::UploadStatus {
            status,
            message: stderr_text,
        });
    }
    Ok(())
}
