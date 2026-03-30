# Market Research CMA

## Purpose
Generate Comparative Market Analysis (CMA) reports and market research insights using live MLS data for Durham Region real estate.

## Requirements

### Requirement: CMA Generation
The Market Research AI SHALL generate Comparative Market Analysis reports using live MLS data from Repliers API.

#### Scenario: Standard CMA request
- **WHEN** an agent requests a CMA for a property at "123 Main St, Pickering"
- **THEN** the AI SHALL search for active listings, recent sales, and expired listings within 1km radius and generate a report within 2 minutes

#### Scenario: Comparable selection
- **WHEN** generating a CMA, the AI SHALL identify 3-5 comparable properties
- **THEN** the system SHALL prioritize properties: same property type, ±15% size, ±20% price, within 1km

### Requirement: Property Comparison Analysis
The AI SHALL analyze and compare multiple properties for buyer clients.

#### Scenario: Buyer shortlist comparison
- **WHEN** an agent requests comparison of 3 shortlisted properties
- **THEN** the AI SHALL generate a side-by-side comparison including price, features, pros/cons

#### Scenario: Investment opportunity analysis
- **WHEN** an agent requests analysis for an investment property
- **THEN** the AI SHALL calculate potential rental yield, appreciation trends, and cash flow estimates

### Requirement: Market Trend Analysis
The AI SHALL analyze and summarize market trends for specified geographic areas.

#### Scenario: Monthly market update
- **WHEN** an agent requests "Durham Region market trends for last 90 days"
- **THEN** the AI SHALL analyze inventory levels, days on market, price trends, and sales velocity

#### Scenario: Neighborhood deep dive
- **WHEN** an agent requests "Ajax South market analysis"
- **THEN** the AI SHALL provide specific data for that neighborhood including average prices, inventory, DOM

### Requirement: Agent-Facing Reports Only
All CMAs and market reports SHALL be presented to the human agent, not directly to clients.

#### Scenario: CMA generation
- **WHEN** a CMA report is generated
- **THEN** the system SHALL store it as "Draft - Agent Review Required" and notify the agent

#### Scenario: Agent reviews CMA
- **WHEN** an agent reviews a CMA draft
- **THEN** the system SHALL allow editing, approval, or rejection with notes

### Requirement: Data Source Attribution
All reports SHALL clearly indicate data sources and confidence levels.

#### Scenario: CMA with comparables
- **WHEN** displaying comparable properties
- **THEN** the report SHALL show: MLS numbers, source (Repliers), sold/listed dates, and data confidence level

### Requirement: Market Sentiment Integration
The AI SHALL incorporate market sentiment data from web sources.

#### Scenario: Sentiment analysis
- **WHEN** generating a market report
- **THEN** the AI SHALL include sentiment indicators from local news sources and Reddit discussions where available
