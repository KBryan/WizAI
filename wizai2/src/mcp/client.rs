//! MCP Client for connecting to MCP servers

use super::protocol::*;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// MCP Client for managing server connections
#[derive(Debug)]
pub struct MCPClient {
    /// Active server connections
    servers: Mutex<HashMap<String, MCPServerConnection>>,
    /// Request ID counter
    request_id: Mutex<u64>,
}

/// MCP Server Connection
pub struct MCPServerConnection {
    /// Server name
    pub name: String,
    /// Server process
    process: Child,
    /// Available tools
    tools: Vec<MCPTool>,
    /// Server capabilities
    capabilities: ServerCapabilities,
}

impl std::fmt::Debug for MCPServerConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MCPServerConnection")
            .field("name", &self.name)
            .field("tools", &self.tools)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl MCPClient {
    /// Create new MCP client
    pub fn new() -> Self {
        Self {
            servers: Mutex::new(HashMap::new()),
            request_id: Mutex::new(0),
        }
    }

    /// Connect to an MCP server via stdio
    pub async fn connect_stdio(
        &self,
        name: &str,
        command: &str,
        args: &[String],
        env: Option<HashMap<String, String>>,
    ) -> Result<()> {
        info!("Connecting to MCP server: {}", name);

        // Build command
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // Spawn process
        let mut child = cmd.spawn()?;

        // Get stdin/stdout handles
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to get stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to get stdout"))?;

        // Initialize connection
        let init_request = InitializeRequest {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            capabilities: ClientCapabilities {
                experimental: None,
            },
            client_info: ImplementationInfo {
                name: "Spree".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 0,
            method: "initialize".to_string(),
            params: init_request,
        };

        // Send initialization request
        let mut stdin_writer = tokio::io::BufWriter::new(stdin);
        let request_json = serde_json::to_string(&request)?;
        stdin_writer.write_all(request_json.as_bytes()).await?;
        stdin_writer.write_all(b"\n").await?;
        stdin_writer.flush().await?;

        // Read response
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let response_line = lines
            .next_line()
            .await?
            .ok_or_else(|| anyhow!("No response from server"))?;

        let response: JsonRpcResponse<InitializeResponse> =
            serde_json::from_str(&response_line)?;

        let init_response = response
            .result
            .ok_or_else(|| anyhow!("Initialization failed: {:?}", response.error))?;

        info!(
            "Connected to MCP server: {} (protocol version: {})",
            init_response.server_info.name, init_response.protocol_version
        );

        // Send initialized notification
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });
        stdin_writer
            .write_all(serde_json::to_string(&notification)?.as_bytes())
            .await?;
        stdin_writer.write_all(b"\n").await?;
        stdin_writer.flush().await?;

        // List available tools
        let tools = self.list_tools(&mut stdin_writer, &mut lines).await?;

        info!("Discovered {} tools from {}", tools.len(), name);
        for tool in &tools {
            debug!("  - {}", tool.name);
        }

        // Store connection
        let connection = MCPServerConnection {
            name: name.to_string(),
            process: child,
            tools,
            capabilities: init_response.capabilities,
        };

        self.servers.lock().await.insert(name.to_string(), connection);

        Ok(())
    }

    /// List available tools from server
    async fn list_tools(
        &self,
        stdin: &mut tokio::io::BufWriter<tokio::process::ChildStdin>,
        stdout: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
    ) -> Result<Vec<MCPTool>> {
        let request_id = self.next_request_id().await;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        };

        // Send request
        let request_json = serde_json::to_string(&request)?;
        stdin.write_all(request_json.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        // Read response
        let response_line = stdout
            .next_line()
            .await?
            .ok_or_else(|| anyhow!("No response"))?;

        let response: JsonRpcResponse<ListToolsResponse> = serde_json::from_str(&response_line)?;

        match response.result {
            Some(result) => Ok(result.tools),
            None => Err(anyhow!("Failed to list tools: {:?}", response.error)),
        }
    }

    /// Call a tool on an MCP server
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let servers = self.servers.lock().await;
        let server = servers
            .get(server_name)
            .ok_or_else(|| anyhow!("Server '{}' not found", server_name))?;

        // For now, return a placeholder
        // In full implementation, we'd maintain the connection and communicate
        info!("Calling tool {} on server {}", tool_name, server_name);

        Ok(CallToolResult {
            content: vec![MCPContent::Text {
                text: format!("Called {} on {}", tool_name, server_name),
            }],
            is_error: false,
        })
    }

    /// Get available tools from a server
    pub async fn get_server_tools(&self, server_name: &str) -> Result<Vec<MCPTool>> {
        let servers = self.servers.lock().await;
        let server = servers
            .get(server_name)
            .ok_or_else(|| anyhow!("Server '{}' not found", server_name))?;

        Ok(server.tools.clone())
    }

    /// Get all connected servers
    pub async fn list_servers(&self) -> Vec<String> {
        self.servers.lock().await.keys().cloned().collect()
    }

    /// Get next request ID
    async fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.lock().await;
        *id += 1;
        *id
    }
}

impl Default for MCPClient {
    fn default() -> Self {
        Self::new()
    }
}
