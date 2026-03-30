//! MCP Protocol Types
//! Based on Model Context Protocol specification

use serde::{Deserialize, Serialize};

/// MCP Protocol Version
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Initialize request
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeRequest {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ImplementationInfo,
}

/// Initialize response
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeResponse {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ImplementationInfo,
}

/// Tool definition from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool call request
#[derive(Debug, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool call result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CallToolResult {
    pub content: Vec<MCPContent>,
    pub is_error: bool,
}

/// Content types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum MCPContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: MCPResource },
}

/// Resource reference
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MCPResource {
    pub uri: String,
    pub mime_type: Option<String>,
}

/// Client capabilities
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
}

/// Server capabilities
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "tools")]
    pub tools: Option<ToolsCapability>,
}

/// Tools capability
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

/// Implementation info
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImplementationInfo {
    pub name: String,
    pub version: String,
}

/// JSON-RPC request wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

/// JSON-RPC response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// List tools request
#[derive(Debug, Serialize, Deserialize)]
pub struct ListToolsRequest {}

/// List tools response
#[derive(Debug, Serialize, Deserialize)]
pub struct ListToolsResponse {
    pub tools: Vec<MCPTool>,
}
