/// MCP JSON-RPC 2.0 request handler.
///
/// Supports two transports:
///   - stdio  (default)  — reads newline-delimited JSON from stdin, writes to stdout
///   - http   (MCP_TRANSPORT=http)  — Axum POST /mcp endpoint on port 8500
use crate::client::VelumClient;
use crate::protocol::{Content, Request, Response};
use crate::tools;
use serde_json::{json, Value};
use anyhow::Result;

const MCP_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "velum-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

fn to_value(r: Response) -> Value {
    serde_json::to_value(&r).unwrap_or(Value::Null)
}

/// Handle a single JSON-RPC request object and return a response Value.
pub async fn handle_request(req: Request, client: &VelumClient) -> Value {
    let id = req.id.clone();
    match req.method.as_str() {
        "initialize" => to_value(Response::ok(
            id,
            json!({
                "protocolVersion": MCP_VERSION,
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION
                }
            }),
        )),

        "notifications/initialized" => {
            // Client notification — no response needed
            Value::Null
        }

        "tools/list" => {
            let defs = tools::all_definitions();
            let tool_list: Vec<Value> = defs
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema
                    })
                })
                .collect();
            to_value(Response::ok(id, json!({ "tools": tool_list })))
        }

        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params["name"].as_str().unwrap_or("").to_string();
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default()));

            match tools::dispatch(&tool_name, &args, client).await {
                Ok(result) => {
                    let content: Vec<Value> = result
                        .content
                        .iter()
                        .map(|c| match c {
                            Content::Text { text } => json!({ "type": "text", "text": text }),
                        })
                        .collect();
                    to_value(Response::ok(
                        id,
                        json!({
                            "content": content,
                            "isError": result.is_error
                        }),
                    ))
                }
                Err(e) => to_value(Response::err(id, -32603, e.to_string())),
            }
        }

        "ping" => to_value(Response::ok(id, json!({}))),

        other => to_value(Response::err(
            id,
            -32601,
            format!("Method not found: {other}"),
        )),
    }
}

/// Run the stdio transport loop: read lines from stdin, write responses to stdout.
pub async fn run_stdio(client: VelumClient) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    tracing::info!("velum-mcp stdio transport ready");

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break; // EOF
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response_value = match serde_json::from_str::<Request>(trimmed) {
            Ok(req) => handle_request(req, &client).await,
            Err(e) => to_value(Response::err(
                None,
                -32700,
                format!("Parse error: {e}"),
            )),
        };

        // Skip null responses (e.g. notifications that need no reply)
        if response_value.is_null() {
            continue;
        }

        let mut out = serde_json::to_string(&response_value)?;
        out.push('\n');
        stdout.write_all(out.as_bytes()).await?;
        stdout.flush().await?;
    }
    Ok(())
}

/// Run the HTTP transport: POST /mcp on port 8500.
pub async fn run_http(client: VelumClient, port: u16) -> Result<()> {
    use axum::{
        extract::State,
        http::StatusCode,
        response::IntoResponse,
        routing::post,
        Json, Router,
    };
    use std::sync::Arc;

    type SharedClient = Arc<VelumClient>;

    async fn mcp_handler(
        State(client): State<SharedClient>,
        Json(req): Json<Request>,
    ) -> impl IntoResponse {
        let val = handle_request(req, &client).await;
        if val.is_null() {
            (StatusCode::NO_CONTENT, Json(Value::Null))
        } else {
            (StatusCode::OK, Json(val))
        }
    }

    let shared = Arc::new(client);
    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .with_state(shared);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("velum-mcp HTTP transport listening on http://{addr}/mcp");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
