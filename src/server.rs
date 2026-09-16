use crate::bridge::BridgeClient;
use crate::cli_runner::CliRunner;
use crate::config::Config;
use crate::error::Result;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use crate::tools::ToolManager;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};

pub struct McpServer {
    tool_manager: Arc<ToolManager>,
}

impl McpServer {
    pub fn new(config: &Config) -> Self {
        let godot_bin = config.resolve_godot_path();
        info!("Using Godot binary at: {}", godot_bin.display());

        let bridge = BridgeClient::new(config.debug_port);
        let cli = CliRunner::new(godot_bin);
        let tool_manager = Arc::new(ToolManager::new(bridge, cli));

        Self { tool_manager }
    }

    /// Run standard I/O loop processing JSON-RPC messages
    pub async fn run_stdio(self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin).lines();

        info!("Godot MCP server listening on stdio");

        while let Some(line) = reader.next_line().await? {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(line) {
                Ok(req) => req,
                Err(err) => {
                    error!("Malformed JSON-RPC request: {err}");
                    let resp = JsonRpcResponse::error(Value::Null, -32700, "Parse error");
                    Self::write_response(&mut stdout, &resp).await?;
                    continue;
                }
            };

            let maybe_response = self.handle_request(request).await;
            if let Some(resp) = maybe_response {
                Self::write_response(&mut stdout, &resp).await?;
            }
        }

        info!("Godot MCP server stdio session closed");
        Ok(())
    }

    async fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = req.id.clone().unwrap_or(Value::Null);

        match req.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "godot-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                });
                Some(JsonRpcResponse::success(id, result))
            }
            "notifications/initialized" => {
                // Client initialized notification, no response required
                None
            }
            "ping" => Some(JsonRpcResponse::success(id, json!({}))),
            "tools/list" => {
                let tools = ToolManager::list_tools();
                Some(JsonRpcResponse::success(id, json!({ "tools": tools })))
            }
            "tools/call" => {
                let tool_name = req
                    .params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or_default();
                let arguments = req
                    .params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));

                let tool_res = self
                    .tool_manager
                    .clone()
                    .call_tool(tool_name, arguments)
                    .await;
                Some(JsonRpcResponse::success(
                    id,
                    serde_json::to_value(tool_res).unwrap_or(Value::Null),
                ))
            }
            _ => {
                if req.id.is_some() {
                    Some(JsonRpcResponse::error(
                        id,
                        -32601,
                        format!("Method '{}' not found", req.method),
                    ))
                } else {
                    None
                }
            }
        }
    }

    async fn write_response(
        stdout: &mut tokio::io::Stdout,
        response: &JsonRpcResponse,
    ) -> Result<()> {
        let json_str = serde_json::to_string(response)?;
        stdout.write_all(json_str.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }
}
