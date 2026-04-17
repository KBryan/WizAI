# Proposal: Market Research API Endpoints

## Why

The AI Real Estate Team MVP has built the foundation with CMA generation capabilities and data models. Now we need REST API endpoints to allow human agents to request CMAs, retrieve comparables, and access market trend data programmatically. This enables integration with agent dashboards and external tools while maintaining the approval workflow safeguards.

## What Changes

- Add REST API endpoints in `src/server/api.rs` for market research operations
- Create CMA generation endpoint with async processing
- Add comparables search endpoint using Repliers API
- Create market trends endpoint for Durham Region analysis
- Implement proper error handling and response formatting
- Add request/response logging for audit trail

## Capabilities

### New Capabilities
- `cma-api`: REST endpoints for generating and retrieving CMA reports
- `comparables-api`: REST endpoints for searching comparable properties
- `market-trends-api`: REST endpoints for market trend analysis

### Modified Capabilities
- `market-research-cma`: Adding API interface requirements (previously only agent-facing)

## Impact

**Code:**
- `src/server/api.rs` - New endpoint handlers
- `src/services/market_research_service.rs` - New service layer (or extend existing)
- `src/models/market.rs` - May need API-specific response structs
- Database migrations for storing API request logs

**APIs:**
- New REST endpoints under `/api/market-research/*`
- Integration with existing Repliers API for live MLS data
- Leverages existing approval workflow for CMA reports

**Dependencies:**
- Existing `market-research-cma` spec capabilities
- Existing `approval-workflow` for CMA review process
- Repliers API credentials (already configured)

**Notes:**
- POC implementation - focus on core functionality over advanced features
- Human agents only (no public/external access in this phase)
- CMAs still require agent approval before being finalized
- Rate limiting can be basic for POC
