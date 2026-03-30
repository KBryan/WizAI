# Durham Real Estate Analysis - Technical Design

## Overview

Build a Rust-based data pipeline to collect, process, and analyze Durham Region real estate data.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Data Collection Layer                     │
├─────────────────────────────────────────────────────────────┤
│  Web Scraper        │  API Client        │  File Parser       │
│  - MLS listings     │  - Toronto Real    │  - CSV/Excel       │
│  - Public records   │    Estate Board    │  - PDF reports     │
│  - Kijiji/Zolo      │  - Open data APIs  │                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Data Processing Layer                   │
├─────────────────────────────────────────────────────────────┤
│  Data Cleaner       │  Normalizer         │  Aggregator      │
│  - Remove dupes     │  - Standardize      │  - By area       │
│  - Validate         │    addresses        │  - By type       │
│  - Handle missing     │  - Parse prices     │  - By month      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Analysis Layer                          │
├─────────────────────────────────────────────────────────────┤
│  Statistics         │  Trend Detection    │  ML Models       │
│  - Avg/median       │  - Moving averages  │  - Price         │
│  - Price/sqft       │  - Seasonality      │    prediction    │
│  - Distribution     │  - Anomalies        │  - Clustering    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Output Layer                            │
├─────────────────────────────────────────────────────────────┤
│  Reports            │  Visualizations     │  CLI Tool        │
│  - Markdown         │  - Charts           │  - Query data    │
│  - CSV exports      │  - Heat maps        │  - Generate      │
│  - JSON API         │  - Trend graphs     │    reports       │
└─────────────────────────────────────────────────────────────┘
```

## Data Sources

### Primary Sources
1. **Zolo.ca** - Public listing data (scrapable)
2. **Realtor.ca** - MLS listings with API access
3. **HouseSigma** - Historical sales data
4. **Toronto Real Estate Board (TREB)** - Market reports
5. **Durham Region Open Data** - Municipal data

### Data Points to Collect
- Property address (parsed to municipality)
- List price
- Sold price (if available)
- Property type (detached, semi, condo, townhouse)
- Bedrooms/bathrooms
- Square footage
- Days on market
- Listing date
- Municipality (Ajax, Pickering, Oshawa, Whitby, etc.)

## Implementation Approach

### Phase 1: Data Collection CLI
Build a Rust CLI tool that:
- Scrapes data from public sources
- Stores in SQLite database
- Handles rate limiting and caching
- Incremental updates (only fetch new listings)

### Phase 2: Analysis Engine
- Calculate statistics by area/property type
- Detect trends and patterns
- Generate automated reports

### Phase 3: Visualization
- Create charts and graphs
- Build heat maps by area
- Export to web dashboard

## Technology Stack

- **Language**: Rust (for speed and reliability)
- **Web Scraping**: `reqwest` + `scraper` (HTML parsing)
- **Database**: SQLite (embedded, no server needed)
- **Analysis**: `polars` (DataFrame operations)
- **Visualization**: `plotters` (charts) or export to Python
- **CLI**: `clap` (command-line interface)

## Risks & Mitigation

| Risk | Mitigation |
|------|------------|
| Website blocking | Rotate user agents, add delays, respect robots.txt |
| Data quality issues | Validation rules, manual spot checks |
| Rate limiting | Implement exponential backoff |
| Legal concerns | Only use public data, check ToS |

## Output Artifacts

1. **CLI Tool**: `durham-realestate` command
2. **Database**: `durham_realestate.db`
3. **Reports**: Markdown files with analysis
4. **Visualizations**: PNG/SVG charts
5. **Raw Data**: CSV exports

## Timeline

- **Week 1**: Data collection CLI
- **Week 2**: Analysis engine
- **Week 3**: Visualization and reporting
- **Week 4**: Documentation and polish
