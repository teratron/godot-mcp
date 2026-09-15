use crate::error::{McpError, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeCommand {
    #[serde(rename = "commandId")]
    pub command_id: String,
    #[serde(rename = "type")]
    pub command_type: String,
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeResponse {
    #[serde(rename = "commandId")]
    pub command_id: String,
    pub status: String,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct BridgeClient {
    port: u16,
}

impl BridgeClient {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Check if the Godot editor bridge WebSocket server is reachable
    pub async fn is_connected(&self) -> bool {
        let url = format!("ws://127.0.0.1:{}", self.port);
        match timeout(Duration::from_millis(500), connect_async(&url)).await {
            Ok(Ok((mut ws, _))) => {
                let _ = ws.close(None).await;
                true
            }
            _ => false,
        }
    }

    /// Send a command to Godot Editor via WebSocket bridge and wait for response
    pub async fn send_command(&self, cmd_type: &str, params: Value) -> Result<Value> {
        let url = format!("ws://127.0.0.1:{}", self.port);
        let (mut ws_stream, _) = match timeout(Duration::from_secs(2), connect_async(&url)).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                return Err(McpError::Bridge(format!(
                    "Failed to connect to Godot Editor bridge at {url}: {e}. Ensure Godot is running and the Godot MCP Bridge plugin is enabled."
                )));
            }
            Err(_) => {
                return Err(McpError::Bridge(format!(
                    "Connection to Godot Editor bridge at {url} timed out. Ensure Godot Editor is open and plugin is active on port {}.",
                    self.port
                )));
            }
        };

        let cmd_id = Uuid::new_v4().to_string();
        let cmd = BridgeCommand {
            command_id: cmd_id.clone(),
            command_type: cmd_type.to_string(),
            params,
        };

        let msg_text = serde_json::to_string(&cmd)?;
        ws_stream
            .send(Message::Text(msg_text.into()))
            .await
            .map_err(|e| McpError::Bridge(format!("Failed to send command to Godot: {e}")))?;

        // Await response with a 15-second timeout
        let response_future = async {
            while let Some(msg_result) = ws_stream.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        let resp: BridgeResponse = serde_json::from_str(text.as_str()).map_err(|e| {
                            McpError::Protocol(format!("Malformed bridge response: {e}"))
                        })?;

                        if resp.command_id == cmd_id {
                            if resp.status == "success" {
                                return Ok(resp.result.unwrap_or(Value::Null));
                            } else {
                                return Err(McpError::ToolExecution(
                                    resp.error.unwrap_or_else(|| "Unknown engine error".into()),
                                ));
                            }
                        }
                    }
                    Ok(Message::Binary(_)) => continue,
                    Ok(Message::Ping(data)) => {
                        let _ = ws_stream.send(Message::Pong(data)).await;
                    }
                    Ok(Message::Pong(_)) => continue,
                    Ok(Message::Close(_)) => {
                        return Err(McpError::Bridge(
                            "Connection closed by Godot Editor bridge".into(),
                        ));
                    }
                    Ok(Message::Frame(_)) => continue,
                    Err(e) => {
                        return Err(McpError::Bridge(format!("WebSocket receive error: {e}")));
                    }
                }
            }
            Err(McpError::Bridge(
                "WebSocket connection closed without response".into(),
            ))
        };

        timeout(Duration::from_secs(15), response_future)
            .await
            .map_err(|_| {
                McpError::Bridge(format!(
                    "Timed out waiting for response from Godot Editor for command '{cmd_type}'"
                ))
            })?
    }
}
