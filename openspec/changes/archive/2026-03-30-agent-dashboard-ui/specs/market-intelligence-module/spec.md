# Spec: Market Intelligence Module

## Purpose

Dashboard module for viewing area market trends, inventory metrics, and neighborhood-level analysis.

## Requirements

### Requirement: Area Selector

The module SHALL provide a way to select geographic areas.

#### Scenario: Area dropdown
- **WHEN** agent opens Market page
- **THEN** an area selector dropdown SHALL appear
- **AND** options SHALL include: Pickering, Ajax, Whitby, Oshawa, Durham Region
- **AND** selecting an area SHALL trigger data fetch

#### Scenario: Neighborhood filter
- **WHEN** area is selected
- **THEN** a neighborhood multi-select SHALL appear
- **AND** agent SHALL be able to filter to specific neighborhoods

### Requirement: Price Trend Chart

The module SHALL display price trends over time.

#### Scenario: 30-day trend
- **WHEN** agent views market page
- **THEN** a line chart SHALL display average price trend for past 30 days
- **AND** data points SHALL show daily/weekly granularity

#### Scenario: 90-day trend
- **WHEN** agent selects "90 days" period
- **THEN** the chart SHALL update to show 90-day trend
- **AND** comparison to previous period SHALL display (+X%)

#### Scenario: 1-year trend
- **WHEN** agent selects "1 year" period
- **THEN** the chart SHALL show annual trend
- **AND** YoY comparison SHALL display

### Requirement: Key Metrics Cards

The module SHALL display key market metrics.

#### Scenario: Metrics display
- **WHEN** area is selected
- **THEN** metric cards SHALL display:
  - Average Price: $[amount]
  - Median Price: $[amount]
  - Days on Market: [X] days
  - Inventory: [X] homes
  - Sales Volume: [X] (this period)
  - Price per Sq Ft: $[amount]

#### Scenario: Trend indicators
- **WHEN** metrics are displayed
- **THEN** each SHALL show trend arrow (up/down) with percentage
- **AND** green SHALL indicate positive, red negative (context-dependent)

### Requirement: Market Status Indicator

The module SHALL indicate current market conditions.

#### Scenario: Seller's market
- **WHEN** months of inventory < 3
- **THEN** status SHALL show "Seller's Market" with red/orange indicator
- **AND** description SHALL explain implications

#### Scenario: Buyer's market
- **WHEN** months of inventory > 6
- **THEN** status SHALL show "Buyer's Market" with blue indicator
- **AND** description SHALL explain implications

#### Scenario: Balanced market
- **WHEN** months of inventory 3-6
- **THEN** status SHALL show "Balanced Market" with green indicator
- **AND** description SHALL explain implications

### Requirement: Neighborhood Comparison Table

The module SHALL provide neighborhood-level comparison.

#### Scenario: Display neighborhoods
- **WHEN** area is selected
- **THEN** a table SHALL display neighborhoods within that area
- **AND** columns SHALL include: Neighborhood, Avg Price, DOM, Inventory, Trend

#### Scenario: Sort neighborhood table
- **WHEN** agent clicks column header
- **THEN** table SHALL sort by that column
- **AND** ascending/descending SHALL toggle

### Requirement: Inventory Analysis

The module SHALL show inventory levels over time.

#### Scenario: Inventory chart
- **WHEN** agent views inventory section
- **THEN** a bar chart SHALL show active listings over past 90 days
- **AND** new listings vs removed SHALL be distinguished

## UI Components

| Component | Description |
|-----------|-------------|
| AreaSelector | Dropdown + neighborhood filter |
| TrendChart | Line chart for price trends |
| MetricsGrid | Grid of metric cards |
| MarketStatusBadge | Visual market condition indicator |
| NeighborhoodTable | Sortable comparison table |
| InventoryChart | Bar chart for inventory levels |
| PeriodSelector | 30d/90d/1y toggle |

## State

| State | Type | Description |
|-------|------|-------------|
| `selected_area` | Signal<Area> | Currently selected area |
| `selected_period` | Signal<Period> | 30d, 90d, or 1y |
| `market_data` | Signal<MarketData> | Fetched market data |
| `neighborhoods` | Signal<Vec<Neighborhood>> | Neighborhood list |
| `is_loading` | Signal<bool> | Loading state |

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/market/trends/:area` | GET | Area trend data |
| `/api/market/stats/:area` | GET | Area statistics |
| `/api/market/neighborhoods/:area` | GET | Neighborhood data |
