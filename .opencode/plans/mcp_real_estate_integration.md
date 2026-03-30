# MCP Integration Plan: Real Estate Research Organization

## Executive Summary

**Goal**: Integrate MCP (Model Context Protocol) into Spree to create a powerful Real Estate research organization that can dynamically access external data sources, APIs, and tools through standardized MCP servers.

**Impact**: Transform Spree from a closed system with built-in tools to an open platform that connects to live real estate data (MLS, Zillow, county records, mortgage calculators), enabling truly autonomous market research.

---

## 1. What is MCP?

**Model Context Protocol (MCP)** is an open protocol that standardizes how applications provide context to LLMs. Think of it as "USB-C for AI applications" - a universal way to connect AI systems to data sources and tools.

### Key Benefits for Real Estate:
- **Live Data Access**: Connect to MLS databases, Zillow APIs, county records
- **Dynamic Tool Loading**: Add/remove data sources without code changes
- **Standardized Interface**: Same protocol for all external integrations
- **Growing Ecosystem**: Community-built MCP servers for common tasks

---

## 2. MCP Architecture for Spree

### 2.1 System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Spree Agent Framework                     │
│                                                              │
│  ┌─────────────┐    ┌──────────────────┐    ┌─────────────┐ │
│  │   Agent     │───▶│  MCP Client      │───▶│ MCP Servers │ │
│  │  (Research) │    │  (Bridge Layer)   │    │  (External) │ │
│  └─────────────┘    └──────────────────┘    └─────────────┘ │
│                           │                                  │
│                           ▼                                  │
│                    ┌──────────────────┐                     │
│                    │  Tool Registry    │                     │
│                    │  (Unified Tools)  │                     │
│                    └──────────────────┘                     │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    MCP Servers                               │
├─────────────────────────────────────────────────────────────┤
│  MLS Database Server    │  Zillow API Server                │
│  County Records Server  │  Browser Automation Server        │
│  Mortgage Calculator    │  Google Maps Server               │
│  Weather Data Server    │  Census Data Server               │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

**Research Request Flow**:
```
1. Research Lead Agent receives request: "Analyze Durham Region prices"
   ↓
2. Agent calls MCP tool: "mls.query_listings"
   ↓
3. MCP Client routes to MLS Database Server
   ↓
4. Server queries live MLS data
   ↓
5. Results returned to Agent
   ↓
6. Agent calls additional tools (Zillow, county records)
   ↓
7. Agent synthesizes findings into report
```

---

## 3. MCP Server Inventory for Real Estate

### 3.1 Tier 1: Essential (Build First)

#### 1. MLS Database MCP Server
**Purpose**: Query live Multiple Listing Service data

**Capabilities**:
- Search listings by location, price range, property type
- Get listing details (price, days on market, photos)
- Comparable sales (comps) analysis
- Historical price trends

**Tools**:
```json
{
  "name": "mls.search_listings",
  "description": "Search MLS database for properties",
  "parameters": {
    "location": "string (city, zip, or coordinates)",
    "min_price": "number (optional)",
    "max_price": "number (optional)",
    "property_type": "enum: [detached, condo, townhouse, commercial]",
    "min_beds": "number (optional)",
    "min_baths": "number (optional)",
    "days_on_market": "number (optional, max days)"
  }
}
```

**Data Sources**: 
- Local MLS APIs (requires Realtor association membership)
- IDX (Internet Data Exchange) feeds
- Third-party aggregators (Estated, ATTOM)

---

#### 2. Zillow/Redfin API MCP Server
**Purpose**: Access Zillow's property data and estimates

**Capabilities**:
- Property valuations (Zestimates)
- Price history
- Comparable properties
- Market trends by region
- Rental estimates

**Tools**:
```json
{
  "name": "zillow.get_property_details",
  "description": "Get detailed property information from Zillow",
  "parameters": {
    "address": "string",
    "city": "string",
    "state": "string",
    "zip": "string"
  }
}
```

**APIs**: Zillow API, Redfin API

---

#### 3. Browser Automation MCP Server
**Purpose**: Scrape websites that don't have APIs

**Capabilities**:
- Visit competitor websites
- Extract listing data
- Screenshot pages
- Fill out forms
- Navigate complex sites

**Tools**:
```json
{
  "name": "browser.visit_and_extract",
  "description": "Visit a URL and extract structured data",
  "parameters": {
    "url": "string",
    "extraction_schema": "object (what data to extract)",
    "take_screenshot": "boolean"
  }
}
```

**Technology**: Playwright, Puppeteer via MCP protocol

---

### 3.2 Tier 2: High Value (Build Next)

#### 4. County Records MCP Server
**Purpose**: Access public property records

**Capabilities**:
- Property ownership history
- Tax assessments
- Zoning information
- Building permits
- Liens and encumbrances

**Data Sources**: 
- County assessor APIs
- Public records databases
- Data aggregators (CoreLogic, Black Knight)

---

#### 5. Mortgage Calculator MCP Server
**Purpose**: Calculate mortgage payments and affordability

**Capabilities**:
- Monthly payment calculation
- Affordability analysis
- Interest rate trends
- Comparison of loan types
- Amortization schedules

**Tools**:
```json
{
  "name": "mortgage.calculate_payment",
  "description": "Calculate monthly mortgage payment",
  "parameters": {
    "principal": "number",
    "interest_rate": "number (annual %)",
    "loan_term_years": "number",
    "property_tax": "number (annual)",
    "insurance": "number (annual)"
  }
}
```

---

#### 6. Google Maps MCP Server
**Purpose**: Location analysis and mapping

**Capabilities**:
- Walk scores
- Nearby amenities
- School districts
- Commute times
- Neighborhood boundaries

---

### 3.3 Tier 3: Advanced (Future)

#### 7. Weather/Climate MCP Server
**Purpose**: Climate risk assessment

**Capabilities**:
- Flood zone data
- Climate change projections
- Natural disaster risk
- Insurance implications

---

#### 8. Census/Demographic MCP Server
**Purpose**: Demographic analysis

**Capabilities**:
- Population trends
- Income demographics
- Age distribution
- Migration patterns

---

#### 9. School District MCP Server
**Purpose**: Education quality analysis

**Capabilities**:
- School ratings
- Test scores
- District boundaries
- Private school options

---

## 4. Implementation Architecture

### 4.1 New Module: `src/mcp/`

```
spree/src/
├── mcp/
│   ├── mod.rs              # Public exports
│   ├── client.rs           # MCP client implementation
│   ├── server_manager.rs   # Server lifecycle management
│   ├── protocol.rs         # MCP protocol types
│   ├── tool_adapter.rs     # Bridge MCP tools to Spree registry
│   └── servers/            # Built-in MCP servers
│       ├── mls/
│       ├── zillow/
│       ├── browser/
│       └── county_records/
```

### 4.2 Core Components

#### MCP Client (`client.rs`)

```rust
pub struct MCPClient {
    /// Active MCP server connections
    servers: HashMap<String, MCPServerConnection>,
    /// Tool registry for unified access
    tool_registry: Arc<ToolRegistry>,
    /// Venice client for LLM calls
    venice: Arc<VeniceClient>,
}

impl MCPClient {
    /// Connect to an MCP server
    pub async fn connect_server(
        &mut self,
        name: &str,
        transport: MCPTransport,
    ) -> Result<ServerConnection>;
    
    /// Discover available tools from MCP server
    pub async fn discover_tools(&self, server_name: &str) -> Result<Vec<MCPTool>>;
    
    /// Execute MCP tool
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<MCPCallResult>;
    
    /// Register MCP tools as Spree tools
    pub async fn register_mcp_tools(&self, server_name: &str) -> Result<()>;
}
```

#### MCP Protocol Types (`protocol.rs`)

```rust
// MCP Protocol Messages
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPInitializeRequest {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    pub client_info: ImplementationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPCallToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPCallToolResult {
    pub content: Vec<MCPContent>,
    pub is_error: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MCPContent {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { resource: MCPResource },
}
```

### 4.3 Server Transport Options

**Option 1: stdio (Recommended for built-in servers)**
```rust
// Spawn MCP server as subprocess
let child = tokio::process::Command::new("mcp-mls-server")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

// Communicate via stdin/stdout (JSON-RPC)
```

**Option 2: HTTP/SSE**
```rust
// Connect to remote MCP server
let client = reqwest::Client::new();
// Use Server-Sent Events for streaming
```

**Option 3: WebSocket**
```rust
// Real-time bidirectional communication
let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
```

---

## 5. Built-in MCP Servers

### 5.1 MLS Database Server

**File**: `src/mcp/servers/mls/`

```rust
// MLS Server main.rs
#[tokio::main]
async fn main() {
    let server = MLSMCPServer::new(
        std::env::var("MLS_API_KEY").expect("MLS_API_KEY required"),
        std::env::var("MLS_API_URL").expect("MLS_API_URL required"),
    );
    
    // Run as stdio MCP server
    let transport = StdioTransport::new(std::io::stdin(), std::io::stdout());
    server.run(transport).await.unwrap();
}
```

**Supported Tools**:
- `mls.search_listings`
- `mls.get_listing_details`
- `mls.get_comparable_sales`
- `mls.get_price_history`
- `mls.get_market_trends`

---

### 5.2 Zillow API Server

**File**: `src/mcp/servers/zillow/`

```rust
pub struct ZillowMCPServer {
    api_key: String,
    client: reqwest::Client,
}

impl ZillowMCPServer {
    async fn handle_get_property(&self, params: Value) -> Result<MCPCallToolResult> {
        let address = params["address"].as_str().unwrap();
        let property = self.zillow_api.get_property(address).await?;
        
        Ok(MCPCallToolResult {
            content: vec![MCPContent::Text {
                text: serde_json::to_string(&property)?
            }],
            is_error: false,
        })
    }
}
```

**Supported Tools**:
- `zillow.get_property_details`
- `zillow.get_zestimate`
- `zillow.get_comparable_properties`
- `zillow.get_price_history`
- `zillow.get_rental_estimate`

---

### 5.3 Browser Automation Server

**File**: `src/mcp/servers/browser/`

```rust
pub struct BrowserMCPServer {
    playwright: Playwright,
    browser: Browser,
}

impl BrowserMCPServer {
    async fn handle_visit_and_extract(&self, params: Value) -> Result<MCPCallToolResult> {
        let url = params["url"].as_str().unwrap();
        let schema: ExtractionSchema = serde_json::from_value(params["extraction_schema"].clone())?;
        
        let page = self.browser.new_page(url).await?;
        let data = page.extract_data(schema).await?;
        
        // Take screenshot if requested
        let screenshot = if params["take_screenshot"].as_bool().unwrap_or(false) {
            Some(page.screenshot().await?)
        } else {
            None
        };
        
        Ok(MCPCallToolResult {
            content: vec![
                MCPContent::Text { text: serde_json::to_string(&data)? },
                MCPContent::Image { 
                    data: base64::encode(&screenshot.unwrap()),
                    mime_type: "image/png".to_string()
                }
            ],
            is_error: false,
        })
    }
}
```

**Supported Tools**:
- `browser.visit_and_extract`
- `browser.fill_form`
- `browser.click_element`
- `browser.take_screenshot`
- `browser.scroll_and_extract`

---

## 6. Spree Integration Points

### 6.1 Agent Role Integration

**Update Real Estate Researcher role** to use MCP tools:

```rust
// In src/agent/roles/real_estate_researcher.rs
pub async fn conduct_market_analysis(&self, params: MarketAnalysisParams) -> Result<MarketReport> {
    // Get MLS data
    let listings = self.mcp_client.call_tool(
        "mls-server",
        "mls.search_listings",
        json!({
            "location": params.location,
            "property_type": params.property_type,
            "days_on_market": 90
        })
    ).await?;
    
    // Get Zillow estimates
    let zillow_data = self.mcp_client.call_tool(
        "zillow-server",
        "zillow.get_market_trends",
        json!({"zip_code": params.zip_code})
    ).await?;
    
    // Scrape competitor sites
    let competitor_data = self.mcp_client.call_tool(
        "browser-server",
        "browser.visit_and_extract",
        json!({
            "url": params.competitor_url,
            "extraction_schema": self.get_listing_schema()
        })
    ).await?;
    
    // Synthesize report
    self.generate_market_report(listings, zillow_data, competitor_data).await
}
```

### 6.2 Tool Registry Integration

**Modify ToolRegistry** to include MCP tools:

```rust
impl ToolRegistry {
    pub async fn load_mcp_tools(&mut self, mcp_client: &MCPClient) -> Result<()> {
        // Discover tools from all connected MCP servers
        for server_name in mcp_client.list_servers() {
            let tools = mcp_client.discover_tools(&server_name).await?;
            
            for mcp_tool in tools {
                // Wrap MCP tool as Spree tool
                let spree_tool = Tool {
                    name: format!("{}.{}", server_name, mcp_tool.name),
                    description: mcp_tool.description,
                    parameters: mcp_tool.input_schema,
                    requires_approval: self.should_require_approval(&mcp_tool.name),
                    handler: Box::new(move |ctx, call| {
                        // Forward to MCP client
                        tokio::runtime::Handle::current().block_on(async {
                            mcp_client.call_tool(&server_name, &mcp_tool.name, call.arguments.clone()).await
                        })
                    }),
                };
                
                self.register(spree_tool);
            }
        }
        
        Ok(())
    }
}
```

### 6.3 Configuration

**Config file**: `spree/config/mcp.yaml`

```yaml
mcp:
  enabled: true
  
  servers:
    # Built-in servers
    mls-server:
      type: builtin
      command: "./mcp-servers/mls-server"
      env:
        MLS_API_KEY: "${MLS_API_KEY}"
        MLS_API_URL: "https://api.mls-provider.com/v1"
      auto_start: true
      
    zillow-server:
      type: builtin
      command: "./mcp-servers/zillow-server"
      env:
        ZILLOW_API_KEY: "${ZILLOW_API_KEY}"
      auto_start: true
      
    browser-server:
      type: builtin
      command: "./mcp-servers/browser-server"
      auto_start: true
      
    # External servers
    google-maps:
      type: sse
      url: "https://mcp.google.com/maps/sse"
      api_key: "${GOOGLE_MAPS_API_KEY}"
      
    custom-county-records:
      type: http
      url: "http://localhost:8080/mcp"
      
  security:
    # Require approval for data-modifying tools
    approval_required:
      - "mls.*"
      - "browser.fill_form"
      - "browser.click_element"
    
    # Rate limiting per server
    rate_limits:
      mls-server: 100  # calls per minute
      zillow-server: 50
      browser-server: 20
```

---

## 7. Real Estate Research Workflow with MCP

### 7.1 Example: Durham Region Market Analysis

**Step 1: Research Lead creates project**
```
Agent: "Research Lead Durham"
Task: "Conduct comprehensive market analysis for Durham Region"
Budget: $500
```

**Step 2: Agent queries MLS data**
```
Tool Call: mls.search_listings
Parameters:
  location: "Durham Region, ON"
  property_types: ["detached", "condo", "townhouse"]
  days_on_market: 90
  
Result: 1,247 active listings
```

**Step 3: Get Zillow market data**
```
Tool Call: zillow.get_market_trends
Parameters:
  zip_codes: ["L1G", "L1H", "L1J", "L1K"]
  
Result: Price trends, inventory levels, days on market
```

**Step 4: Scrape competitor websites**
```
Tool Call: browser.visit_and_extract
Parameters:
  urls: ["realtor.ca/durham", "zolo.ca/durham"]
  extraction_schema: {listings: [], avg_price: "", market_summary: ""}
  
Result: Competitor listing data and pricing
```

**Step 5: Query county records**
```
Tool Call: county.get_tax_assessments
Parameters:
  municipality: "Durham"
  year: 2024
  
Result: Property assessments, tax trends
```

**Step 6: Calculate affordability**
```
Tool Call: mortgage.calculate_affordability
Parameters:
  median_income: 85000
  location: "Durham Region"
  
Result: What locals can afford
```

**Step 7: Generate report**
Agent synthesizes all data into comprehensive market report with:
- Market trends
- Price analysis
- Inventory levels
- Competitor positioning
- Affordability metrics
- Investment recommendations

**Step 8: Human review**
Human researcher reviews AI findings, adds insights, approves for delivery.

---

## 8. Business Model Integration

### 8.1 Cost Tracking

**Track costs per MCP call**:

```rust
// In mcp/client.rs
pub async fn call_tool_with_cost_tracking(
    &self,
    server_name: &str,
    tool_name: &str,
    arguments: Value,
    agent_id: AgentId,
) -> Result<MCPCallResult> {
    let start = Instant::now();
    
    // Call the tool
    let result = self.call_tool(server_name, tool_name, arguments).await?;
    
    let duration = start.elapsed();
    
    // Calculate cost based on server pricing
    let cost = match server_name {
        "mls-server" => 0.01,  // $0.01 per MLS query
        "zillow-server" => 0.005, // $0.005 per Zillow API call
        "browser-server" => duration.as_secs_f64() * 0.001, // $0.001 per second
        _ => 0.001,
    };
    
    // Track in payment system
    self.payment_system.charge_mcp(agent_id, server_name, tool_name, cost).await?;
    
    Ok(result)
}
```

### 8.2 Pricing

**Client Billing**:
```
Research Report: $800
  - Base research: $400
  - MCP data costs: $150
    - MLS queries (15 @ $0.01): $0.15
    - Zillow API calls (20 @ $0.005): $0.10
    - Browser automation (5 min @ $0.06/min): $0.30
    - County records (5 queries @ $0.02): $0.10
    - Markup (20x cost): $2.85 → $150
  - Human review: $200
  - Platform fee: $50
```

---

## 9. Security Considerations

### 9.1 Data Protection

- **MLS Data**: Licensed data, client confidentiality
- **Personal Info**: Anonymize all owner data
- **Rate Limiting**: Prevent abuse of external APIs
- **Caching**: Cache expensive queries to reduce costs

### 9.2 Approval Workflow

**High-risk operations require approval**:
- Modifying external databases (never for research)
- Form submissions on websites
- Automated offers/purchases (future feature)

### 9.3 Audit Trail

```rust
pub struct MCPAuditLog {
    pub timestamp: DateTime<Utc>,
    pub agent_id: AgentId,
    pub server_name: String,
    pub tool_name: String,
    pub cost: f64,
    pub success: bool,
    pub data_hash: String,  // Hash of returned data for verification
}
```

---

## 10. Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Goal**: Get basic MCP infrastructure working

**Tasks**:
- [ ] Create `src/mcp/` module structure
- [ ] Implement MCP protocol types
- [ ] Build MCP client with stdio transport
- [ ] Create simple test MCP server
- [ ] Integrate with ToolRegistry

**Deliverable**: Spree can connect to and call tools from a test MCP server

---

### Phase 2: First Real Estate Server (Week 3-4)

**Goal**: Zillow API integration

**Tasks**:
- [ ] Build Zillow MCP server
- [ ] Implement tool: `zillow.get_property_details`
- [ ] Implement tool: `zillow.get_market_trends`
- [ ] Add cost tracking
- [ ] Test with Real Estate Researcher agent

**Deliverable**: Agent can query Zillow data autonomously

---

### Phase 3: MLS Integration (Week 5-6)

**Goal**: Live MLS data access

**Tasks**:
- [ ] Obtain MLS API access (or use IDX feed)
- [ ] Build MLS MCP server
- [ ] Implement core MLS tools
- [ ] Add caching layer
- [ ] Test comprehensive market analysis

**Deliverable**: Agent can conduct full market analysis with live MLS data

---

### Phase 4: Browser Automation (Week 7-8)

**Goal**: Scrape competitor websites

**Tasks**:
- [ ] Build Browser MCP server with Playwright
- [ ] Implement data extraction tools
- [ ] Add screenshot capability
- [ ] Create extraction schemas for common sites
- [ ] Test competitive analysis workflow

**Deliverable**: Agent can scrape and analyze competitor listings

---

### Phase 5: Production (Week 9-12)

**Goal**: Production-ready system

**Tasks**:
- [ ] Error handling and retries
- [ ] Monitoring and alerting
- [ ] Performance optimization
- [ ] Security audit
- [ ] Documentation
- [ ] Launch pilot with real clients

**Deliverable**: Live Real Estate Research Organization accepting paying clients

---

## 11. Technical Requirements

### 11.1 Dependencies

**New Cargo.toml additions**:
```toml
[dependencies]
# MCP Protocol
mcp-core = "0.1"  # Official MCP Rust SDK (when available)

# Or custom implementation
serde_json = "1.0"
tokio = { version = "1", features = ["process", "io-util"] }

# Browser automation
playwright = "0.1"

# HTTP clients for API calls
reqwest = { version = "0.11", features = ["json", "stream"] }

# Server-Sent Events
eventsource-client = "0.11"

# Rate limiting
governor = "0.6"

# Caching
redis = { version = "0.23", optional = true }
```

### 11.2 External Requirements

**API Keys Required**:
- Zillow API key (free tier: 1000 calls/month)
- MLS access (requires Realtor license or data provider)
- Google Maps API key
- County records API (varies by location)

**Infrastructure**:
- Redis (optional, for caching)
- Separate process spawning support

---

## 12. Success Metrics

### 12.1 Technical Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| MCP tool latency | < 2s | Average response time |
| Success rate | > 95% | % of successful tool calls |
| Cache hit rate | > 30% | % of queries served from cache |
| Cost per report | < $50 | Total MCP costs |

### 12.2 Business Metrics

| Metric | Month 6 | Month 12 |
|--------|---------|----------|
| Reports delivered | 50 | 200 |
| MCP tool calls | 5,000 | 25,000 |
| Revenue | $20,000 | $100,000 |
| MCP costs | $1,500 | $7,500 |
| Gross margin | 75% | 80% |

---

## 13. Competitive Advantage

### What This Gives You:

1. **Live Data**: Competitors use stale data, you use real-time MLS feeds
2. **Comprehensive**: One query pulls from 5+ data sources automatically
3. **Cost Efficiency**: AI does the research, humans just review
4. **Scale**: Add new data sources by connecting new MCP servers
5. **Customization**: Each client can request specific data sources

### vs. Traditional Real Estate Research:

| Aspect | Traditional | Spree + MCP |
|--------|-------------|-------------|
| Data sources | 1-2 (manual) | 5-10+ (automated) |
| Delivery time | 1-2 weeks | 24-48 hours |
| Cost | $5,000+ | $500-1000 |
| Update frequency | Monthly | Real-time |
| Customization | Limited | Unlimited |

---

## 14. Risks and Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| MLS API access denied | Medium | High | Partner with data aggregators; use public records |
| Rate limiting | High | Medium | Implement caching; queue requests |
| API costs explode | Low | High | Set budgets; monitor usage; alert on anomalies |
| Data quality issues | Medium | High | Validation layers; human spot-checks |
| MCP protocol changes | Low | Medium | Version pinning; abstraction layer |

---

## 15. Conclusion

**MCP integration transforms Spree from a closed system to an open platform that connects to the entire real estate data ecosystem.**

**Key Outcomes**:
- Real-time access to MLS, Zillow, county records
- Automated competitive analysis
- Comprehensive market reports in hours, not weeks
- Scalable to any data source via MCP servers
- Sustainable business model with clear unit economics

**The Result**: A billion-dollar Real Estate research organization powered by AI agents with live data access.

---

**Next Steps**:
1. Review this plan
2. Prioritize Phase 1 implementation
3. Obtain MLS/Zillow API access
4. Begin building MCP infrastructure

**Ready to execute?** This is the technical foundation for your Real Estate empire.
