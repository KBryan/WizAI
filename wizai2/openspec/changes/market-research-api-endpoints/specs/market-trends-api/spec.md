## ADDED Requirements

### Requirement: Market Trends by Area
The system SHALL provide market trend analysis for specified geographic areas in Durham Region.

#### Scenario: Get trends for city
- **WHEN** a GET request is made to `/api/market-research/trends/pickering`
- **THEN** the system SHALL return HTTP 200 with market trends data for Pickering

#### Scenario: Get trends for region
- **WHEN** a GET request is made to `/api/market-research/trends/durham-region`
- **THEN** the system SHALL return aggregate trends for the entire Durham Region

#### Scenario: Invalid area
- **WHEN** a GET request is made for an unsupported area (not in Durham Region)
- **THEN** the system SHALL return HTTP 400 with a list of supported areas

### Requirement: Trend Period Selection
The system SHALL support different time periods for trend analysis.

#### Scenario: Default 90-day period
- **WHEN** a trends request is made without specifying a period
- **THEN** the system SHALL return data for the last 90 days

#### Scenario: 30-day trends
- **WHEN** a GET request is made with query parameter `period=30d`
- **THEN** the system SHALL return trends for the last 30 days

#### Scenario: 1-year trends
- **WHEN** a GET request is made with query parameter `period=1y`
- **THEN** the system SHALL return trends for the last 12 months

#### Scenario: Invalid period
- **WHEN** a request is made with an unsupported period (e.g., `period=10d`)
- **THEN** the system SHALL return HTTP 400 with supported period options

### Requirement: Trend Metrics
The system SHALL provide comprehensive market metrics in the response.

#### Scenario: Key metrics included
- **WHEN** trends data is returned
- **THEN** the response SHALL include: average price, median price, price per square foot, median days on market, inventory levels, sales volume, price trend direction (increase/decrease percentage)

#### Scenario: Market status indicator
- **WHEN** trends are calculated
- **THEN** the response SHALL include a `market_status` field with values: "Buyer's Market", "Seller's Market", or "Balanced Market" based on inventory and sales velocity

#### Scenario: Data freshness
- **WHEN** trends data is returned
- **THEN** the response SHALL include a `generated_at` timestamp and `data_as_of` date indicating when the source data was retrieved

### Requirement: Neighborhood-Level Analysis
The system SHALL support neighborhood-level trend analysis within cities.

#### Scenario: Neighborhood trends
- **WHEN** a GET request is made to `/api/market-research/trends/pickering/amberleigh`
- **THEN** the system SHALL return trends specific to the Amberleigh neighborhood

#### Scenario: Available neighborhoods
- **WHEN** a trends request is made for a city
- **THEN** the response SHALL include a `neighborhoods` array listing available neighborhoods within that city
