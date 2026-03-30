# Spec: CMA Dashboard Widget

## Purpose

Interactive widget for generating Comparative Market Analysis reports with property input, comparable visualization, and price recommendations.

## Requirements

### Requirement: Property Address Input

The CMA widget SHALL accept a property address for CMA generation.

#### Scenario: Address search
- **WHEN** agent enters an address in the search field
- **THEN** autocomplete suggestions SHALL appear
- **AND** selecting a suggestion SHALL populate the field

#### Scenario: Property type selection
- **WHEN** address is entered
- **THEN** property type dropdown SHALL appear
- **AND** options SHALL include: Detached, Semi-Detached, Townhouse, Condo, Commercial

#### Scenario: Generate CMA
- **WHEN** agent clicks "Generate CMA"
- **THEN** a loading state SHALL appear
- **AND** the backend SHALL begin CMA generation
- **AND** polling SHALL begin for completion

### Requirement: Comparable Properties List

The CMA SHALL display comparable properties found.

#### Scenario: Display comparables
- **WHEN** CMA generation completes
- **THEN** a list of 3-5 comparable properties SHALL display
- **AND** each SHALL show: address, price, sqft, beds/baths, distance, sold date
- **AND** match confidence SHALL be shown as percentage

#### Scenario: Comparable card
- **WHEN** a comparable property is displayed
- **THEN** agent SHALL be able to click for details
- **AND** toggle inclusion in final CMA

### Requirement: Price Recommendation

The CMA SHALL display a recommended price range.

#### Scenario: Price range display
- **WHEN** CMA is complete
- **THEN** a price recommendation SHALL show:
  - Low: $[amount]
  - Recommended: $[amount]
  - High: $[amount]
- **AND** confidence percentage SHALL be displayed

#### Scenario: Visual price chart
- **WHEN** CMA is complete
- **THEN** a bar chart SHALL display comparables vs subject
- **AND** the recommendation range SHALL be highlighted
- **AND** agent SHALL be able to adjust recommendation

### Requirement: Market Conditions Summary

The CMA SHALL include market condition analysis.

#### Scenario: Display market context
- **WHEN** CMA is complete
- **THEN** a section SHALL show:
  - Average days on market for comparables
  - List-to-sale ratio
  - Price per square foot
  - Market status (Buyer's/Seller's/Balanced)

### Requirement: Export Functionality

The CMA SHALL support export to PDF.

#### Scenario: Export to PDF
- **WHEN** agent clicks "Export PDF"
- **THEN** a branded PDF SHALL be generated
- **AND** download SHALL begin automatically
- **AND** the PDF SHALL include: property details, comparables table, price chart, market conditions, disclaimers

### Requirement: CMA History

The widget SHALL show recent CMAs for the agent.

#### Scenario: Recent CMAs list
- **WHEN** agent navigates to CMA page
- **THEN** recent CMAs SHALL display in a sidebar/list
- **AND** each SHALL show: address, date, status
- **AND** clicking SHALL load that CMA

## UI Components

| Component | Description |
|-----------|-------------|
| PropertyInput | Address search + property type form |
| ComparablesList | Scrollable list of comp properties |
| ComparableCard | Single comparable with details |
| PriceRecommendation | Price range display with chart |
| MarketConditions | Market stats summary |
| CmaHistory | Sidebar list of recent CMAs |
| ExportButton | PDF export trigger |

## State

| State | Type | Description |
|-------|------|-------------|
| `address` | Signal<String> | Current address input |
| `property_type` | Signal<PropertyType> | Selected property type |
| `cma_status` | Signal<CmaStatus> | Generation status |
| `cma_result` | Signal<Option<CmaResult>> | Completed CMA data |
| `comparables` | Signal<Vec<Comparable>> | List of comparables |
| `selected_comparables` | Signal<Vec<String>> | IDs selected for export |

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/cmas` | POST | Start CMA generation |
| `/api/cmas/:id` | GET | Get CMA status/result |
| `/api/market/comparables` | GET | Search comparables |
| `/api/market/trends/:area` | GET | Area market trends |
