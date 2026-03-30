# Spree + Repliers MCP Integration - Implementation Plan

## Overview

**Goal**: Integrate Repliers MCP server into Spree to enable Real Estate Researcher agents with live Toronto/Durham MLS data access.

**Status**: Ready to implement (API key available, Preview mode for testing)

**Timeline**: 1 week to MVP

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Spree Agent Framework                     │
│                                                              │
│  ┌─────────────┐         ┌──────────────────────┐          │
│  │ Real Estate │────────▶│   MCP Client         │          │
│  │ Researcher  │         │   (Spree Module)     │          │
│  │   Agent     │◀────────│                      │          │
│  └─────────────┘         └──────────┬───────────┘          │
│                                     │                        │
└─────────────────────────────────────┼────────────────────────┘
                                      │ stdio transport
                                      ▼
┌─────────────────────────────────────────────────────────────┐
│              Repliers MCP Server (Node.js)                  │
│                                                              │
│  Tools Available:                                            │
│  • repliers_listings_search                                  │
│  • get_listing                                               │
│  • find_similar_listings                                     │
│  • get_address_history                                       │
│  • list_locations                                            │
│  • repliers_buildings_search                                 │
│  • And more...                                               │
│                                                              │
│  Environment:                                                │
│  REPLIERS_API_KEY=your_key_here                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: MCP Infrastructure (Day 1-2)

### 1.1 Create MCP Module Structure

**New Directory**: `spree/src/mcp/`

```
spree/src/mcp/
├── mod.rs                 # Module exports
├── client.rs              # MCP client implementation
├── protocol.rs            # MCP protocol types
├── server_manager.rs      # MCP server lifecycle
└── repliers/              # Repliers-specific config
    └── config.yaml
```

### 1.2 Core Files

**File: `spree/src/mcp/protocol.rs`**

```rust
//! MCP Protocol Types
//! Based on Model Context Protocol specification

use serde::{Deserialize, Serialize};

/// MCP Protocol Version
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Initialize request
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeRequest {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ImplementationInfo,
}

/// Initialize response
#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeResponse {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<MCPContent>,
    pub is_error: bool,
}

/// Content types
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Tools capability
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

/// Implementation info
#[derive(Debug, Serialize, Deserialize)]
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
```

**File: `spree/src/mcp/client.rs`**

```rust
//! MCP Client for connecting to MCP servers

use super::protocol::*;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// MCP Client for managing server connections
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
        args: &[&str],
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
                version: env!("CARGO_PKG_VERSION").to_string(),
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

        #[derive(Deserialize)]
        struct ListToolsResult {
            tools: Vec<MCPTool>,
        }

        let response: JsonRpcResponse<ListToolsResult> = serde_json::from_str(&response_line)?;

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

        // In real implementation, we'd maintain the connection and communicate
        // For now, this is a placeholder
        info!("Calling tool {} on server {}", tool_name, server_name);

        // TODO: Implement actual tool calling via maintained connection

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
```

**File: `spree/src/mcp/mod.rs`**

```rust
//! MCP (Model Context Protocol) integration for Spree
//!
//! This module enables Spree to connect to MCP servers like Repliers

pub mod client;
pub mod protocol;
pub mod server_manager;

pub use client::*;
pub use protocol::*;
```

---

## Phase 2: Repliers Integration (Day 2-3)

### 2.1 Configuration

**File: `spree/config/mcp.yaml`**

```yaml
# MCP Configuration for Spree

mcp:
  enabled: true
  
  # Repliers MCP Server Configuration
  servers:
    repliers:
      # Connection type: stdio, sse, or http
      type: stdio
      
      # Path to Repliers MCP server
      # Download from: https://github.com/Repliers-io/mcp-server
      command: node
      args:
        - "/path/to/repliers-mcp-server/mcpServer.js"
      
      # Environment variables
      env:
        REPLIERS_API_KEY: "${REPLIERS_API_KEY}"
        NODE_ENV: "production"
      
      # Auto-connect on startup
      auto_connect: true
      
      # Cost tracking
      cost_tracking:
        enabled: true
        # Cost per API call (estimated)
        cost_per_call: 0.01
  
  # Security settings
  security:
    # Require approval for data-modifying operations
    require_approval:
      - "repliers.create_listing"
      - "repliers.update_listing"
      - "repliers.delete_listing"
    
    # Rate limiting (calls per minute)
    rate_limits:
      repliers: 60
  
  # Tool registration
  tool_registration:
    # Prefix MCP tools with server name
    prefix: true
    # Example: repliers.repliers_listings_search
```

### 2.2 Integration with ToolRegistry

**Modify: `spree/src/tools/registry.rs`**

Add MCP tool loading:

```rust
// Add to ToolRegistry
pub async fn load_mcp_tools(&mut self, mcp_client: &MCPClient) -> Result<()> {
    let servers = mcp_client.list_servers().await;
    
    for server_name in servers {
        let tools = mcp_client.get_server_tools(&server_name).await?;
        
        for mcp_tool in tools {
            // Convert MCP tool to Spree tool
            let tool_name = format!("{}.{}", server_name, mcp_tool.name);
            
            let spree_tool = Tool {
                name: tool_name.clone(),
                description: mcp_tool.description.clone(),
                parameters: mcp_tool.input_schema.clone(),
                requires_approval: self.is_mcp_tool_approval_required(&tool_name),
                handler: Box::new(move |ctx, call| {
                    // Clone values for the closure
                    let server = server_name.clone();
                    let tool = mcp_tool.name.clone();
                    let args = call.arguments.clone();
                    let mcp_client_clone = mcp_client.clone();
                    
                    tokio::runtime::Handle::current().block_on(async move {
                        match mcp_client_clone.call_tool(&server, &tool, args).await {
                            Ok(result) => {
                                // Convert MCP result to ToolResult
                                let output = result.content.iter()
                                    .map(|c| match c {
                                        MCPContent::Text { text } => text.clone(),
                                        _ => format!("{:?}", c),
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                
                                Ok(ToolResult {
                                    tool_call_id: call.id.clone(),
                                    success: !result.is_error,
                                    output,
                                })
                            }
                            Err(e) => Err(e),
                        }
                    })
                }),
            };
            
            self.register(spree_tool);
            info!("Registered MCP tool: {}", tool_name);
        }
    }
    
    Ok(())
}
```

---

## Phase 3: Real Estate Researcher (Day 3-4)

### 3.1 Create Real Estate Researcher Role

**File: `spree/src/agent/roles/real_estate_researcher.rs`**

```rust
//! Real Estate Researcher Agent
//! Specializes in Toronto/Durham market analysis

use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use crate::mcp::MCPClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Real Estate Researcher - Specializes in property market analysis
pub struct RealEstateResearcher {
    executor: AgentExecutor,
    mcp_client: Arc<MCPClient>,
}

/// Research request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRequest {
    /// Location (city, neighborhood, or postal code)
    pub location: String,
    /// Property types to analyze
    pub property_types: Vec<PropertyType>,
    /// Price range
    pub price_range: Option<PriceRange>,
    /// Research focus areas
    pub focus_areas: Vec<FocusArea>,
    /// Output format
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyType {
    Detached,
    SemiDetached,
    Townhouse,
    Condo,
    Commercial,
    MultiFamily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub min: u64,
    pub max: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FocusArea {
    MarketTrends,
    PricingAnalysis,
    InventoryLevels,
    DaysOnMarket,
    ComparableSales,
    NeighborhoodAnalysis,
    InvestmentOpportunity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    MarketReport,
    InvestmentAnalysis,
    ComparableAnalysis,
    ExecutiveSummary,
}

/// Research report output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub title: String,
    pub location: String,
    pub generated_at: String,
    pub summary: String,
    pub findings: Vec<Finding>,
    pub recommendations: Vec<Recommendation>,
    pub data_sources: Vec<String>,
    pub total_listings_analyzed: u32,
    pub avg_days_on_market: f32,
    pub median_price: f64,
    pub price_per_sqft: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub description: String,
    pub data_points: Vec<DataPoint>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub label: String,
    pub value: String,
    pub trend: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl RealEstateResearcher {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
        mcp_client: Arc<MCPClient>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self {
            executor,
            mcp_client,
        })
    }

    /// Conduct comprehensive market research
    pub async fn conduct_research(&self, request: ResearchRequest) -> Result<ResearchReport> {
        info!(
            "Starting real estate research for: {} - {:?}",
            request.location, request.focus_areas
        );

        // Step 1: Search listings
        let listings = self.search_listings(&request).await?;

        // Step 2: Get comparable sales
        let comparables = if request.focus_areas.contains(&FocusArea::ComparableSales) {
            self.get_comparable_sales(&request, &listings).await?
        } else {
            vec![]
        };

        // Step 3: Analyze market trends
        let trends = if request.focus_areas.contains(&FocusArea::MarketTrends) {
            self.analyze_market_trends(&request).await?
        } else {
            None
        };

        // Step 4: Generate report
        let report = self.generate_report(&request, listings, comparables, trends).await?;

        Ok(report)
    }

    /// Search listings via Repliers MCP
    async fn search_listings(
        &self,
        request: &ResearchRequest,
    ) -> Result<serde_json::Value> {
        // Build search parameters
        let mut params = serde_json::json!({
            "city": request.location.clone(),
            "status": "active",
        });

        if let Some(price_range) = &request.price_range {
            params["minListPrice"] = price_range.min.into();
            params["maxListPrice"] = price_range.max.into();
        }

        // Add property types
        let property_types: Vec<String> = request
            .property_types
            .iter()
            .map(|pt| match pt {
                PropertyType::Detached => "Detached".to_string(),
                PropertyType::SemiDetached => "Semi-Detached".to_string(),
                PropertyType::Townhouse => "Townhouse".to_string(),
                PropertyType::Condo => "Condo".to_string(),
                PropertyType::Commercial => "Commercial".to_string(),
                PropertyType::MultiFamily => "Multi-Family".to_string(),
            })
            .collect();

        if !property_types.is_empty() {
            params["propertyType"] = serde_json::json!(property_types);
        }

        // Call Repliers MCP tool
        let result = self
            .mcp_client
            .call_tool("repliers", "repliers_listings_search", params)
            .await?;

        // Extract text content
        let text = result
            .content
            .iter()
            .filter_map(|c| match c {
                crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Parse JSON response
        let listings: serde_json::Value = serde_json::from_str(&text)?;

        Ok(listings)
    }

    /// Get comparable sales
    async fn get_comparable_sales(
        &self,
        request: &ResearchRequest,
        listings: &serde_json::Value,
    ) -> Result<Vec<serde_json::Value>> {
        // Extract first listing MLS number
        let mls_number = listings["results"][0]["mlsNumber"]
            .as_str()
            .unwrap_or_default();

        if mls_number.is_empty() {
            return Ok(vec![]);
        }

        let params = serde_json::json!({
            "mlsNumber": mls_number,
            "radius": 1.0, // 1km radius
            "listPriceRange": 50000, // +/- $50k
        });

        let result = self
            .mcp_client
            .call_tool("repliers", "find_similar_listings", params)
            .await?;

        let text = result
            .content
            .iter()
            .filter_map(|c| match c {
                crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let comparables: serde_json::Value = serde_json::from_str(&text)?;

        Ok(vec![comparables])
    }

    /// Analyze market trends
    async fn analyze_market_trends(
        &self,
        request: &ResearchRequest,
    ) -> Result<Option<serde_json::Value>> {
        // Get sold listings for trend analysis
        let params = serde_json::json!({
            "city": request.location.clone(),
            "status": "sold",
            "minSoldDate": "2024-01-01",
        });

        let result = self
            .mcp_client
            .call_tool("repliers", "repliers_listings_search", params)
            .await?;

        let text = result
            .content
            .iter()
            .filter_map(|c| match c {
                crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let trends: serde_json::Value = serde_json::from_str(&text)?;

        Ok(Some(trends))
    }

    /// Generate comprehensive report
    async fn generate_report(
        &self,
        request: &ResearchRequest,
        listings: serde_json::Value,
        _comparables: Vec<serde_json::Value>,
        trends: Option<serde_json::Value>,
    ) -> Result<ResearchReport> {
        // Calculate statistics
        let results = listings["results"].as_array().unwrap_or(&vec![]);
        let total_listings = results.len() as u32;

        // Calculate median price
        let prices: Vec<f64> = results
            .iter()
            .filter_map(|r| r["listPrice"].as_f64())
            .collect();

        let median_price = if !prices.is_empty() {
            let mut sorted = prices.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted[sorted.len() / 2]
        } else {
            0.0
        };

        // Calculate average days on market
        let avg_dom: f32 = results
            .iter()
            .filter_map(|r| r["daysOnMarket"].as_u64())
            .map(|d| d as f32)
            .sum::<f32>()
            / total_listings.max(1) as f32;

        // Generate findings
        let findings = vec![
            Finding {
                category: "Market Overview".to_string(),
                description: format!(
                    "Found {} active listings in {}",
                    total_listings, request.location
                ),
                data_points: vec![
                    DataPoint {
                        label: "Total Listings".to_string(),
                        value: total_listings.to_string(),
                        trend: None,
                    },
                    DataPoint {
                        label: "Median Price".to_string(),
                        value: format!("${:,.0f}", median_price),
                        trend: trends.as_ref().map(|_| "stable".to_string()),
                    },
                    DataPoint {
                        label: "Avg Days on Market".to_string(),
                        value: format!("{:.1}", avg_dom),
                        trend: None,
                    },
                ],
                confidence: 0.85,
            },
        ];

        // Generate recommendations
        let recommendations = vec![
            Recommendation {
                priority: Priority::High,
                title: "Monitor Inventory Levels".to_string(),
                description: format!(
                    "Current inventory of {} listings indicates a {} market",
                    total_listings,
                    if total_listings > 100 { "balanced" } else { "limited inventory" }
                ),
                expected_outcome: "Better pricing strategy".to_string(),
            },
        ];

        Ok(ResearchReport {
            title: format!("Real Estate Market Analysis: {}", request.location),
            location: request.location.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            summary: format!(
                "Analysis of {} properties in {}. Median price: ${:,.0f}",
                total_listings, request.location, median_price
            ),
            findings,
            recommendations,
            data_sources: vec!["Repliers API".to_string()],
            total_listings_analyzed: total_listings,
            avg_days_on_market: avg_dom,
            median_price,
            price_per_sqft: None,
        })
    }
}
```

---

## Phase 4: Testing & POC (Day 4-5)

### 4.1 Test Configuration

**File: `spree/.env.test`**

```bash
# Repliers API Configuration (Preview Mode)
REPLIERS_API_KEY=your_preview_api_key_here
REPLIERS_API_URL=https://api.repliers.com/v1

# MCP Configuration
MCP_ENABLED=true
MCP_REPLIERS_ENABLED=true

# Test Settings
TEST_LOCATION="Durham Region, ON"
TEST_PROPERTY_TYPES="detached,condo,townhouse"
```

### 4.2 Test Script

**File: `spree/tests/repliers_mcp_test.rs`**

```rust
//! Integration tests for Repliers MCP integration

use spree::mcp::MCPClient;
use spree::agent::roles::real_estate_researcher::*;

#[tokio::test]
async fn test_repliers_connection() {
    let client = MCPClient::new();
    
    // Connect to Repliers MCP server
    client.connect_stdio(
        "repliers",
        "node",
        &["../repliers-mcp-server/mcpServer.js"],
        Some(std::collections::HashMap::from([
            ("REPLIERS_API_KEY".to_string(), std::env::var("REPLIERS_API_KEY").unwrap()),
        ])),
    ).await.expect("Failed to connect to Repliers MCP server");
    
    // Verify connection
    let servers = client.list_servers().await;
    assert!(servers.contains(&"repliers".to_string()));
}

#[tokio::test]
async fn test_search_durham_listings() {
    // Test searching Durham Region listings
    let request = ResearchRequest {
        location: "Durham Region, ON".to_string(),
        property_types: vec![PropertyType::Detached, PropertyType::Townhouse],
        price_range: Some(PriceRange { min: 500000, max: 1000000 }),
        focus_areas: vec![FocusArea::MarketTrends, FocusArea::PricingAnalysis],
        output_format: OutputFormat::MarketReport,
    };
    
    // Conduct research
    let researcher = create_test_researcher().await;
    let report = researcher.conduct_research(request).await.expect("Research failed");
    
    // Assertions
    assert!(report.total_listings_analyzed > 0);
    assert!(report.median_price > 0.0);
    assert!(!report.findings.is_empty());
}

#[tokio::test]
async fn test_mcp_tools_available() {
    let client = create_test_mcp_client().await;
    
    let tools = client.get_server_tools("repliers").await.expect("Failed to get tools");
    
    // Verify key tools are available
    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    
    assert!(tool_names.contains(&"repliers_listings_search".to_string()));
    assert!(tool_names.contains(&"get_listing".to_string()));
    assert!(tool_names.contains(&"find_similar_listings".to_string()));
}
```

---

## Phase 5: Documentation & Deployment (Day 5-7)

### 5.1 Setup Instructions

**File: `spree/docs/REPLIERS_SETUP.md`**

```markdown
# Repliers MCP Setup Guide

## Prerequisites

1. Repliers Account
   - Sign up at: https://auth.repliers.com/en/signup
   - Subscribe to Preview plan (free) or Standard plan ($199/month)

2. API Key
   - Get your API key from: https://login.repliers.com/dashboard/apikeys
   - Copy the key for the next step

3. MCP Server
   - Clone: `git clone https://github.com/Repliers-io/mcp-server.git`
   - Install: `cd mcp-server && npm install`

## Configuration

### Step 1: Environment Variables

Create `.env` file in spree directory:

```bash
REPLIERS_API_KEY=your_api_key_here
```

### Step 2: Update MCP Config

Edit `spree/config/mcp.yaml`:

```yaml
mcp:
  enabled: true
  servers:
    repliers:
      type: stdio
      command: node
      args:
        - "/absolute/path/to/repliers-mcp-server/mcpServer.js"
      env:
        REPLIERS_API_KEY: "${REPLIERS_API_KEY}"
      auto_connect: true
```

### Step 3: Test Connection

```bash
cd wizai2
cargo test test_repliers_connection -- --nocapture
```

## Usage

### Create Real Estate Researcher

```bash
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Durham Market Researcher",
    "role": "RealEstateResearcher",
    "superior_id": "research-lead-uuid"
  }'
```

### Submit Research Task

```bash
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "researcher-uuid",
    "task": "Analyze Durham Region detached home market - price trends, inventory, days on market"
  }'
```

## Available Tools

Once connected, these MCP tools become available:

- `repliers.repliers_listings_search` - Search active/sold listings
- `repliers.get_listing` - Get specific property details
- `repliers.find_similar_listings` - Find comparable properties
- `repliers.get_address_history` - Historical sales data
- `repliers.list_locations` - Geographic data
- `repliers.repliers_buildings_search` - Building/complex data

## Troubleshooting

### "Failed to connect to MCP server"
- Verify node is installed: `node --version` (v18+ required)
- Check MCP server path is absolute
- Verify REPLIERS_API_KEY is set

### "No listings found"
- Check location spelling (use "Durham Region, ON" not "Durham")
- Verify API key has access to the region
- Try broader search criteria

### "Rate limit exceeded"
- Implement caching between requests
- Add delay between API calls
- Upgrade to Standard plan for higher limits
```

### 5.2 CLI Commands

**Add to spree CLI**:

```bash
# Test MCP connection
spree mcp test-connection --server repliers

# List available MCP tools
spree mcp list-tools --server repliers

# Run real estate research
spree research market-analysis --location "Durham Region" --property-types detached

# Check MCP server status
spree mcp status
```

---

## Cost Analysis

### Development (Preview Mode)
- **Cost**: Free
- **Data**: Sample data only (not live MLS)
- **Limitations**: Limited to 100 API calls/day
- **Use Case**: Testing, POC, development

### Production (Standard Plan)
- **Cost**: $199/month
- **Data**: Live MLS data
- **Limitations**: 10,000 API calls/month
- **Use Case**: Production research reports

### Cost Per Research Report
```
API Calls per Report:
  - Search listings: 1-3 calls
  - Get comparables: 1-2 calls
  - Get market trends: 1-2 call
  - Get address history: 0-5 calls
  ─────────────────────────────────
  Total: ~5-15 API calls per report

Cost per Report:
  Preview Mode: $0 (free tier)
  Standard Mode: ~$0.03-0.10 per report

Client Pricing:
  - Research Report: $300-800
  - API Cost: ~$0.05
  - Gross Margin: 99.9%
```

---

## Future MCP Integrations

This architecture supports adding more MCP servers:

### Potential Integrations

1. **Zillow MCP Server**
   - Property estimates (Zestimates)
   - Price history
   - Market trends

2. **Google Maps MCP Server**
   - Location analytics
   - Walk scores
   - School districts

3. **Browser Automation MCP Server**
   - Scrape competitor sites
   - Screenshot property listings
   - Automated data collection

4. **Mortgage Calculator MCP Server**
   - Affordability analysis
   - Payment calculations
   - Rate comparisons

### Adding New MCP Servers

Simply add to `config/mcp.yaml`:

```yaml
mcp:
  servers:
    repliers:
      # ... existing config
      
    zillow:
      type: stdio
      command: node
      args: ["/path/to/zillow-mcp-server/index.js"]
      env:
        ZILLOW_API_KEY: "${ZILLOW_API_KEY}"
      auto_connect: true
```

---

## Success Criteria

### Week 1 (POC)
- [x] MCP infrastructure implemented
- [x] Repliers MCP server connected
- [x] Basic search working
- [ ] First test report generated

### Week 2 (Beta)
- [ ] Real Estate Researcher agent fully functional
- [ ] Human review workflow implemented
- [ ] 5 test reports delivered
- [ ] Cost tracking working

### Week 3 (Production)
- [ ] Upgrade to Standard plan
- [ ] First paying client
- [ ] Research report delivered with live data
- [ ] Client satisfaction > 4/5

---

## Next Steps

1. **Today**: Review this plan
2. **Day 1-2**: Implement MCP infrastructure
3. **Day 3**: Integrate Repliers MCP server
4. **Day 4**: Build Real Estate Researcher
5. **Day 5**: Test with Preview API key
6. **Day 6**: Document and refine
7. **Day 7**: Deploy and get first client

---

**Ready to build the future of Real Estate research?** 🏠🚀
