# Design: Market Research API Endpoints

## Context

The AI Real Estate Team MVP has established the foundation with:
- CMA generation capabilities in the AI agent layer
- Market data models (CMAReport, Comparable, MarketData)
- Compliance and approval workflow infrastructure
- Repliers API integration for live MLS data

Currently, these capabilities are only accessible through the AI agent system. Human agents need REST API endpoints to programmatically request CMAs, search comparables, and access market trends.

**Stakeholders:** Human real estate agents (Durham Region)
**Constraints:** POC scope - focus on core functionality, agent-only access

## Goals / Non-Goals

**Goals:**
- Provide REST API endpoints for CMA generation, comparables search, and market trends
- Maintain existing approval workflow for CMA reports
- Enable async processing for long-running CMA generation
- Support Durham Region market specialization
- Follow existing API patterns in `src/server/api.rs`

**Non-Goals:**
- Public/external API access (agent-only for POC)
- Real-time WebSocket updates (can be added later)
- Advanced rate limiting (basic protection only)
- GraphQL or other query languages (REST only)
- Client SDK generation

## Decisions

### Decision: API Structure
**Choice:** RESTful endpoints under `/api/market-research/`
**Rationale:** 
- Consistent with existing API patterns in the codebase
- Simple for human agents to understand and use
- Easy to document and test

**Alternative considered:** GraphQL
- **Rejected:** Overkill for POC, adds complexity without clear benefit for 3-4 endpoints

### Decision: Async Processing for CMA Generation
**Choice:** Return 202 Accepted immediately, provide status endpoint
**Rationale:**
- CMA generation involves Repliers API calls and AI processing (can take 30-120 seconds)
- Prevents HTTP timeout issues
- Follows REST best practices for long-running operations

**Pattern:**
1. POST /api/market-research/cma → Returns `{ "cma_id": "...", "status": "processing" }`
2. GET /api/market-research/cma/:id/status → Returns current status
3. GET /api/market-research/cma/:id → Returns completed report (when ready)

### Decision: Response Format
**Choice:** JSON with standardized envelope
**Rationale:**
- Consistency across all endpoints
- Easy to parse for agents using various tools

**Format:**
```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

### Decision: Authentication
**Choice:** Reuse existing auth middleware (agent session/token based)
**Rationale:**
- POC scope - no need to reinvent auth
- Human agents already have authentication through existing system

### Decision: Error Handling
**Choice:** Standard HTTP status codes with JSON error details
**Rationale:**
- RESTful and familiar to developers
- Provides machine-readable error information

**Status codes:**
- 200: Success
- 202: Accepted (async processing started)
- 400: Bad Request (validation errors)
- 401: Unauthorized
- 404: Resource not found
- 500: Server error

### Decision: Data Source
**Choice:** Repliers API for live data, cached where appropriate
**Rationale:**
- Already integrated in the system
- Provides accurate Durham Region MLS data
- CMA results can be cached briefly (1 hour) to reduce API costs

### Decision: Rate Limiting
**Choice:** Basic per-agent rate limiting (configurable, default: 10 req/min)
**Rationale:**
- POC scope - prevent abuse without complex rules
- Can be enhanced later with tiered limits

## Risks / Trade-offs

**Risk:** Repliers API latency affects user experience
→ **Mitigation:** Async processing with status endpoints, clear UI messaging about processing time

**Risk:** CMA generation failures (Repliers API down, invalid addresses)
→ **Mitigation:** Proper error handling, retry logic with exponential backoff, clear error messages

**Risk:** Storing sensitive property data
→ **Mitigation:** Existing data retention policies, audit logging, agent-only access

**Risk:** API endpoints expose too much data
→ **Mitigation:** POC uses agent-only auth, field-level filtering can be added later

**Trade-off:** Simple vs. Flexible
- Keeping endpoints simple for POC means less flexibility
- Extensions (filters, sorting, pagination) can be added in v2

**Trade-off:** Sync vs. Async for Comparables
- Comparables search is typically fast (<5s) so can be synchronous
- CMA generation requires AI processing so must be async

## API Endpoints

### POST /api/market-research/cma
Generate a CMA report for a property.

**Request:**
```json
{
  "address": "123 Main St, Pickering",
  "property_type": "Detached",
  "bedrooms": 3,
  "bathrooms": 2,
  "square_feet": 1500
}
```

**Response (202 Accepted):**
```json
{
  "success": true,
  "data": {
    "cma_id": "cma-uuid-123",
    "status": "processing",
    "estimated_completion": "2026-03-29T15:30:00Z"
  }
}
```

### GET /api/market-research/cma/:id
Retrieve a completed CMA report.

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "cma-uuid-123",
    "status": "completed",
    "address": "123 Main St, Pickering",
    "estimated_value": "$850,000 - $900,000",
    "comparables": [...],
    "market_trends": {...},
    "generated_at": "2026-03-29T15:30:00Z"
  }
}
```

### GET /api/market-research/cma/:id/status
Check CMA generation status.

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "cma_id": "cma-uuid-123",
    "status": "processing",
    "progress": 65,
    "message": "Analyzing comparables..."
  }
}
```

### GET /api/market-research/comparables
Search for comparable properties.

**Query Parameters:**
- `address` (required): Property address
- `radius` (optional): Search radius in km (default: 1)
- `property_type` (optional): Filter by type
- `limit` (optional): Max results (default: 5, max: 10)

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "target_property": {...},
    "comparables": [...],
    "count": 5
  }
}
```

### GET /api/market-research/trends/:area
Get market trends for a specific area.

**Path Parameters:**
- `area`: Area name (e.g., "pickering", "ajax", "durham-region")

**Query Parameters:**
- `period` (optional): "30d", "90d", "1y" (default: "90d")

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "area": "Pickering",
    "period": "90d",
    "average_price": "$875,000",
    "median_days_on_market": 12,
    "inventory_level": 145,
    "sales_velocity": "Stable",
    "trend_direction": "+2.3%",
    "generated_at": "2026-03-29T15:30:00Z"
  }
}
```

## Implementation Notes

**Files to modify/create:**
- `src/server/api.rs` - Add endpoint handlers
- `src/server/mod.rs` - Register new routes
- `src/services/market_research_service.rs` - New service (or extend existing)
- `src/models/api.rs` - API request/response structs
- Database migrations for API request logging

**Async Processing:**
- Use existing async runtime (tokio)
- Consider background job queue for CMA generation (or spawn task)
- Store processing status in database

**Caching:**
- Cache comparables results for 1 hour
- Cache market trends for 6 hours
- Cache CMA reports indefinitely (they don't change)

## Open Questions

1. Should we implement background job processing (e.g., using a queue) or just spawn async tasks?
2. Do we need webhook notifications when CMA is complete, or is polling sufficient for POC?
3. What retention policy for CMA reports? (Suggest: keep for 90 days)
4. Should comparables endpoint support POST with full property details for better matching?
