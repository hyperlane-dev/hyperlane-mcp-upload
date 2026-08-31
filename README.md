# hyperlane-mcp-upload

MCP server that uploads local files to [ltpp.vip](https://ltpp.vip) and returns the public download URL.

The server speaks JSON-RPC 2.0 over HTTP at `/mcp` and exposes a single tool, `upload_file`, that takes a `file_path` argument pointing at a file on the local filesystem.

## Protocol

The implementation follows the [Model Context Protocol](https://modelcontextprotocol.io/) JSON-RPC conventions.

- `initialize` — handshake returning server identity and capabilities.
- `tools/list` — returns the registered tool descriptors.
- `tools/call` — invokes a tool by name.

### `tools/call` example

```bash
curl -X POST http://127.0.0.1:7842/mcp \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "upload_file",
      "arguments": { "file_path": "/path/to/file.png" }
    }
  }'
```

The response contains a `content` array whose single text element is a JSON object with `url`, `file_name`, and `size`.

## Endpoints

| Path | Method | Purpose |
|------|--------|---------|
| `/mcp` | POST | JSON-RPC 2.0 endpoint |
| `/health` | GET | Static health check returning `{"status":"ok"}` |
| `/*` | ANY | 404 fallback |

## Upload Pipeline

Files are streamed to ltpp.vip through the same three-step REST protocol used by the `hyperlane-upload` Node CLI:

1. `POST /api/upload/register` — register the upload with file size, chunk size, total chunks, and MD5 hash.
2. `POST /api/upload/save` — send one or more `application/octet-stream` chunks.
3. `POST /api/upload/merge` — ask ltpp.vip to merge the chunks into a single URL.

The MD5 hash and HTTP transport are delegated to the system `md5sum` and `curl` commands so no extra Rust dependencies are required.

## Configuration

| Constant | Default | Description |
|----------|---------|-------------|
| `MCP_DEFAULT_HOST` | `0.0.0.0` | Bind address |
| `MCP_DEFAULT_PORT` | `7842` | Bind port |
| `LTPP_HOST` | `ltpp.vip` | Upload service host |
| `LTPP_BASE_PATH` | `/api/upload` | Upload service base path |
| `DEFAULT_CHUNK_SIZE` | `5 MiB` | Chunk size sent to ltpp.vip |

## Build

```bash
cargo build --release
hyperlane fmt
cargo clippy --all-targets
```

## Run

```bash
./target/release/hyperlane-mcp-upload
```

The server listens on `0.0.0.0:7842` by default.

## License

MIT