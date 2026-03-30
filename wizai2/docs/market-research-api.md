# Market Research API Documentation

REST API endpoints for real estate market research operations including CMA generation, comparable property search, and market trend analysis for Durham Region.

**Base URL:** `/api/market-research`

## Overview

The Market Research API provides human agents with programmatic access to:
- **CMA Reports** - Generate Comparative Market Analysis reports for properties
- **Comparable Properties** - Search for similar properties within a radius
- **Market Trends** - Get market analysis for Durham Region areas

All endpoints return JSON responses with a standardized envelope format.

## Response Format

All API responses follow this structure:

```json
{
  "success": true|false,
  "data": { ... },  // Response data (omitted if error)
  "error": {        // Error details (omitted if success)
    "message": "Error description",
    "code": "ERROR_CODE"
  }
}
```

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Invalid request parameters |
| `INVALID_AREA` | 400 | Unsupported geographic area |
| `NOT_FOUND` | 404 | Resource not found |
| `QUERY_ERROR` | 500 | Database query failed |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

## Endpoints

### CMA Operations

#### POST /api/market-research/cma

Create a new CMA report for a property. This is an asynchronous operation.

**Request Body:**
```json
{
  "address": "123 Main St, Pickering",
  "property_type": "Detached",
  "bedrooms": 3,
  "bathrooms": 2,
  "square_feet": 1500
}
```

**Required Fields:**
- `address` - Full property address
- `property_type` - Property category (Detached, SemiDetached, Townhouse, Condo, etc.)

**Optional Fields:**
- `bedrooms` - Number of bedrooms
- `bathrooms` - Number of bathrooms
- `square_feet` - Property square footage

**Response (202 Accepted):**
```json
{
  "success": true,
  "data": {
    "cma_id": "cma-550e8400-e29b-41d4-a716-446655440000",
    "status": "processing",
    "estimated_completion": "2026-03-29T15:30:00Z"
  }
}
```

**Error Response:**
```json
{
  "success": false,
  "error": {
    "message": "Address is required",
    "code": "VALIDATION_ERROR"
  }
}
```

---

#### GET /api/market-research/cma/:id

Retrieve a completed CMA report.

**Path Parameters:**
- `id` - CMA report ID (UUID)

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "cma-550e8400-e29b-41d4-a716-446655440000",
    "subject_address": "123 Main St, Pickering",
    "property_type": "Detached",
    "status": "draft",
    "estimated_value": "$850,000 - $900,000",
    "comparables": [
      {
        "mls_number": "E1234567",
        "address": "125 Main St, Pickering",
        "price": 875000,
        "bedrooms": 3,
        "bathrooms": 2,
        "square_feet": 1450,
        "days_on_market": 8,
        "similarity_score": 95
      }
    ],
    "market_analysis": {
      "avg_days_on_market": 12,
      "price_per_sqft": 580,
      "market_conditions": "Balanced market with stable inventory levels"
    },
    "generated_by": "api-request",
    "created_at": "2026-03-29T15:00:00Z",
    "updated_at": "2026-03-29T15:02:00Z"
  }
}
```

**Response (202 Processing):**
```json
{
  "success": true,
  "data": {
    "status": "processing",
    "estimated_completion": "2026-03-29T15:02:00Z"
  }
}
```

---

#### GET /api/market-research/cma/:id/status

Check the status of a CMA generation job.

**Path Parameters:**
- `id` - CMA report ID (UUID)

**Response:**
```json
{
  "success": true,
  "data": {
    "cma_id": "cma-550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "progress": 100,
    "message": "CMA completed and awaiting review"
  }
}
```

**Status Values:**
- `queued` - Waiting to be processed
- `processing` - Currently generating
- `analyzing_comparables` - Analyzing comparable properties
- `generating_report` - Compiling final report
- `completed` - Ready for review
- `failed` - Generation failed

---

#### GET /api/market-research/cma

List CMA reports with filtering and pagination.

**Query Parameters:**
- `status` - Filter by status (draft, under_review, approved, rejected)
- `sort` - Sort order (date:desc, date:asc, score:desc)
- `page` - Page number (default: 1)
- `limit` - Items per page (default: 20, max: 100)

**Response:**
```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "cma-550e8400-e29b-41d4-a716-446655440000",
        "subject_address": "123 Main St, Pickering",
        "property_type": "Detached",
        "status": "draft",
        "price_recommendation_mid": 875000,
        "comparable_count": 5,
        "confidence": 85,
        "created_at": "2026-03-29T15:00:00Z"
      }
    ],
    "total": 45,
    "page": 1,
    "limit": 20
  }
}
```

---

### Comparables Operations

#### GET /api/market-research/comparables

Search for comparable properties near a target address.

**Query Parameters:**
- `address` (required) - Target property address
- `radius` - Search radius in kilometers (default: 1.0, max: 10.0)
- `property_type` - Filter by property type
- `limit` - Max results (default: 5, max: 10)

**Example Request:**
```
GET /api/market-research/comparables?address=123+Main+St+Pickering&radius=2&limit=5
```

**Response:**
```json
{
  "success": true,
  "data": {
    "target_property": {
      "address": "123 Main St, Pickering",
      "property_type": "Detached",
      "bedrooms": null,
      "bathrooms": null,
      "square_feet": null
    },
    "comparables": [
      {
        "address": "125 Main St, Pickering",
        "mls_number": "E1234567",
        "price": 875000,
        "price_type": "sold",
        "property_type": "Detached",
        "bedrooms": 3,
        "bathrooms": 2,
        "square_feet": 1450,
        "days_on_market": 8,
        "distance_km": 0.2,
        "match_confidence": 95
      },
      {
        "address": "127 Main St, Pickering",
        "mls_number": "E1234568",
        "price": 899000,
        "price_type": "sold",
        "property_type": "Detached",
        "bedrooms": 3,
        "bathrooms": 2,
        "square_feet": 1520,
        "days_on_market": 12,
        "distance_km": 0.3,
        "match_confidence": 92
      }
    ],
    "count": 2
  }
}
```

---

### Market Trends Operations

#### GET /api/market-research/trends/:area

Get market trends for a specific area in Durham Region.

**Path Parameters:**
- `area` - Area name (pickering, ajax, whitby, oshawa, durham-region, etc.)

**Query Parameters:**
- `period` - Analysis period: `30d`, `90d`, or `1y` (default: 90d)

**Supported Areas:**
- Cities: pickering, ajax, whitby, oshawa, courtice, bowmanville, newcastle
- Townships: uxbridge, scugog, brock
- Region: durham-region (aggregate)

**Response:**
```json
{
  "success": true,
  "data": {
    "area": "Pickering",
    "period": "90d",
    "average_price": "$875,000",
    "median_price": "$850,000",
    "price_per_sqft": "$580",
    "median_days_on_market": 12,
    "inventory_level": 145,
    "sales_volume": 89,
    "price_trend_direction": "+2.3%",
    "market_status": "balanced_market",
    "generated_at": "2026-03-29T15:30:00Z",
    "data_as_of": "2026-03-29T15:30:00Z",
    "neighborhoods": [
      "amberleigh",
      "highlands",
      "rouge-park",
      "village-east",
      "rosebank"
    ]
  }
}
```

**Market Status Values:**
- `buyers_market` - More inventory than demand
- `sellers_market` - High demand, limited inventory
- `balanced_market` - Balanced supply and demand

**Error Response (Invalid Area):**
```json
{
  "success": false,
  "error": {
    "message": "Area 'toronto' is not supported. Supported areas: ["pickering", "ajax", "whitby", ...]",
    "code": "INVALID_AREA"
  }
}
```

---

#### GET /api/market-research/areas

Get list of supported areas in Durham Region.

**Response:**
```json
{
  "success": true,
  "data": {
    "areas": [
      "pickering",
      "ajax",
      "whitby",
      "oshawa",
      "courtice",
      "bowmanville",
      "newcastle",
      "uxbridge",
      "scugog",
      "brock",
      "durham-region"
    ],
    "message": "Durham Region market areas supported"
  }
}
```

---

#### GET /api/market-research/neighborhoods/:city

Get neighborhoods for a specific city.

**Path Parameters:**
- `city` - City name (pickering, ajax, whitby, oshawa)

**Response:**
```json
{
  "success": true,
  "data": [
    "amberleigh",
    "highlands",
    "rouge-park",
    "village-east",
    "rosebank"
  ]
}
```

---

## Usage Examples

### Generate a CMA

```bash
# Request CMA generation
curl -X POST http://localhost:3000/api/market-research/cma \
  -H "Content-Type: application/json" \
  -d '{
    "address": "123 Main St, Pickering",
    "property_type": "Detached",
    "bedrooms": 3,
    "bathrooms": 2,
    "square_feet": 1500
  }'

# Response: { "cma_id": "...", "status": "processing" }

# Check status until complete
curl http://localhost:3000/api/market-research/cma/{cma_id}/status

# Retrieve completed report
curl http://localhost:3000/api/market-research/cma/{cma_id}
```

### Search Comparables

```bash
curl "http://localhost:3000/api/market-research/comparables?address=123+Main+St+Pickering&radius=2&limit=5"
```

### Get Market Trends

```bash
# 90-day trends for Pickering (default)
curl http://localhost:3000/api/market-research/trends/pickering

# 30-day trends
curl "http://localhost:3000/api/market-research/trends/pickering?period=30d"

# Full region
curl http://localhost:3000/api/market-research/trends/durham-region
```

---

## Implementation Notes

### Current Status

**Core Features (POC):**
- ✅ All API endpoints functional
- ✅ Request/response validation
- ✅ Standardized JSON envelope
- ✅ API request logging
- ✅ Async CMA processing

**Repliers Integration:**
- ✅ Direct HTTP API client
- ✅ Live sold properties for comparables
- ✅ Live market statistics for trends
- ✅ Active listings search
- ✅ Property details by MLS
- ✅ Extended area statistics
- ✅ Automatic fallback to mock data when Repliers unavailable

**Pending:**
- Caching layer (comparables: 1hr, trends: 6hr)
- Authentication middleware
- Rate limiting (10 req/min per agent)
- Production testing

### CMA Approval Workflow

All CMA reports follow the approval workflow:
1. **Draft** - AI generates initial report
2. **Under Review** - Human agent reviews
3. **Approved** - Ready for client presentation
4. **Rejected** - Needs regeneration

CMAs remain in "Draft" status until reviewed by a human agent.

### Data Sources

**With Repliers API:**
- ✅ Live MLS sold properties for comparables
- ✅ Live market statistics from MLS data
- ✅ Active property listings
- ✅ Detailed property information
- ✅ Price history and market trends

**Fallback:**
- Realistic mock data when Repliers unavailable
- Based on typical Durham Region market values

---

## Rate Limiting

**Current:** No rate limiting (POC)
**Planned:** 10 requests per minute per authenticated agent

## Authentication

**Current:** Test agent ID (POC)
**Planned:** Session-based authentication with agent credentials

---

### Additional Repliers Features

#### GET /api/market-research/listings/active

Get active property listings in a specific area with optional filters.

**Query Parameters:**
- `area` (required) - Area name (e.g., "pickering", "ajax", "whitby")
- `property_type` - Filter by property type
- `price_min` - Minimum price
- `price_max` - Maximum price
- `bedrooms_min` - Minimum number of bedrooms
- `bathrooms_min` - Minimum number of bathrooms
- `limit` - Maximum results (default: 20, max: 50)
- `offset` - Pagination offset

**Response:**
```json
{
  "success": true,
  "data": {
    "listings": [
      {
        "mls_number": "E1234567",
        "address": "123 Main St, Pickering, ON",
        "city": "Pickering",
        "province": "ON",
        "property_type": "Detached",
        "status": "Active",
        "list_price": 899000,
        "bedrooms": 4,
        "bathrooms": 3,
        "square_feet": 2100,
        "days_on_market": 5,
        "features": ["Garage", "Pool", "Finished Basement"],
        "photos": ["https://..."],
        "agent_name": "John Smith",
        "brokerage": "Royal LePage"
      }
    ],
    "total": 145,
    "page": 1,
    "has_more": true
  }
}
```

**Requirements:** Requires REPLIERS_API_KEY environment variable.

---

#### GET /api/market-research/property/:mls_number

Get detailed information about a specific property by MLS number.

**Path Parameters:**
- `mls_number` - MLS number (e.g., "E1234567")

**Response:**
```json
{
  "success": true,
  "data": {
    "property": {
      "mls_number": "E1234567",
      "address": "123 Main St, Pickering, ON",
      "city": "Pickering",
      "property_type": "Detached",
      "status": "Active",
      "price": 899000,
      "bedrooms": 4,
      "bathrooms": 3,
      "square_feet": 2100,
      "year_built": 2015,
      "lot_size": "45x120",
      "description": "Beautiful detached home in sought-after neighborhood...",
      "features": ["Hardwood floors", "Updated kitchen", "Finished basement"],
      "photos": ["https://..."],
      "virtual_tour_url": "https://...",
      "price_history": [
        {"date": "2026-03-01", "price": 925000, "event": "listed"},
        {"date": "2026-03-15", "price": 899000, "event": "price_change"}
      ],
      "property_taxes": 4500,
      "parking_spaces": 4,
      "garage_spaces": 2
    }
  }
}
```

**Requirements:** Requires REPLIERS_API_KEY environment variable.

---

#### GET /api/market-research/statistics/:area

Get detailed market statistics for an area.

**Path Parameters:**
- `area` - Area name (e.g., "pickering", "ajax")

**Query Parameters:**
- `property_type` - Filter by property type (optional)

**Response:**
```json
{
  "success": true,
  "data": {
    "statistics": {
      "area": "Pickering",
      "property_type": null,
      "avg_list_price": 925000,
      "avg_sold_price": 898000,
      "median_sold_price": 875000,
      "price_per_sqft": 585,
      "total_active": 142,
      "total_sold_30d": 28,
      "total_sold_90d": 89,
      "total_sold_1y": 428,
      "avg_days_on_market": 11.5,
      "months_of_inventory": 3.2,
      "new_listings_30d": 45,
      "price_change_percent": 2.3,
      "updated_at": "2026-03-29T15:30:00Z"
    }
  }
}
```

**Requirements:** Requires REPLIERS_API_KEY environment variable.

---

## Rate Limiting

**Current:** No rate limiting (POC)
**Planned:** 10 requests per minute per authenticated agent

## Authentication

**Current:** Test agent ID (POC)
**Planned:** Session-based authentication with agent credentials

## Changelog

**2026-03-29** - Repliers API Integration
- ✅ Direct Repliers HTTP API client
- ✅ Live sold property data for comparables
- ✅ Live market statistics for trends
- ✅ Active listings search
- ✅ Property details by MLS number
- ✅ Extended area statistics
- ✅ Graceful fallback to mock data when Repliers unavailable

**2026-03-29** - Initial POC implementation
- Basic CMA generation with async processing
- Comparables search with mock data
- Market trends for Durham Region
- API request logging

**Next Release:**
- Caching layer (comparables: 1hr, trends: 6hr)
- Rate limiting
- Production authentication
