# Spec: Lead Pipeline Board

## Purpose

Visual Kanban-style board for managing leads through the sales pipeline with drag-and-drop, scoring visualization, and quick actions.

## Requirements

### Requirement: Pipeline Columns

The board SHALL display leads in pipeline columns representing status stages.

#### Scenario: Display columns
- **WHEN** the board loads
- **THEN** columns SHALL display: New, Qualified, Contacted, Negotiating, Closed Won, Closed Lost
- **AND** each column SHALL show lead count in header

#### Scenario: Empty column
- **WHEN** a column has no leads
- **THEN** a placeholder message SHALL display: "No leads in this stage"

### Requirement: Lead Cards

Each lead SHALL be displayed as a card within its pipeline column.

#### Scenario: Lead card content
- **WHEN** a lead card is displayed
- **THEN** it SHALL show: name, score badge (color-coded), source icon, days in stage, property interest
- **AND** quick action buttons SHALL appear on hover

#### Scenario: Score badge colors
- **WHEN** lead score is displayed
- **THEN** scores 8-10 SHALL show red badge "Hot"
- **AND** scores 5-7 SHALL show yellow badge "Warm"
- **AND** scores 1-4 SHALL show gray badge "Cold"

### Requirement: Drag and Drop

Leads SHALL be draggable between pipeline columns.

#### Scenario: Move lead to new stage
- **WHEN** agent drags a lead card to a different column
- **THEN** the card SHALL visually follow cursor
- **AND** drop zones SHALL highlight on hover
- **AND** on drop, status SHALL update via API
- **AND** optimistic UI update SHALL occur

#### Scenario: Invalid status transition
- **WHEN** agent attempts to drag lead to an invalid status
- **THEN** the drop zone SHALL show as disabled
- **AND** a toast SHALL indicate: "Cannot move directly from [X] to [Y]"

### Requirement: Quick Actions

Lead cards SHALL provide quick action buttons.

#### Scenario: Action buttons on hover
- **WHEN** agent hovers over a lead card
- **THEN** buttons SHALL appear: Phone, Email, SMS, View Details
- **AND** clicking Phone SHALL open click-to-call (if supported)

### Requirement: Lead Detail Modal

Clicking a lead card SHALL open a detail modal.

#### Scenario: Open detail modal
- **WHEN** agent clicks a lead card (not a drag)
- **THEN** a modal SHALL open with full lead details
- **AND** editable fields SHALL be form inputs
- **AND** save SHALL update via API

#### Scenario: Modal tabs
- **WHEN** detail modal is open
- **THEN** tabs SHALL show: Overview, Activity, Communications, Notes
- **AND** each tab SHALL load relevant data

### Requirement: Filtering

The board SHALL support filtering leads.

#### Scenario: Filter by score
- **WHEN** agent selects score filter "Hot only"
- **THEN** only leads with score 8+ SHALL display
- **AND** other leads SHALL be hidden

#### Scenario: Filter by source
- **WHEN** agent selects source filter "Website"
- **THEN** only leads from website SHALL display

### Requirement: Statistics Summary

Above the board, summary statistics SHALL display.

#### Scenario: Display stats
- **WHEN** leads page loads
- **THEN** a stats bar SHALL show:
  - Total leads
  - Hot leads count
  - Avg score
  - Conversion rate (Closed Won / Total)
  - Leads this week

## UI Components

| Component | Description |
|-----------|-------------|
| PipelineBoard | Container for all columns |
| PipelineColumn | Single status column with drop zone |
| LeadCard | Draggable lead card |
| LeadModal | Full lead detail modal |
| ScoreBadge | Color-coded score indicator |
| StatsBar | Summary statistics row |
| QuickActions | Icon buttons for lead actions |
| FilterBar | Filter dropdowns and search |

## State

| State | Type | Description |
|-------|------|-------------|
| `leads` | Signal<Vec<Lead>> | All leads from API |
| `dragged_lead` | Signal<Option<Lead>> | Currently dragging |
| `filters` | Signal<LeadFilters> | Active filters |
| `selected_lead` | Signal<Option<Lead>> | Modal open state |
| `stats` | Signal<LeadStats> | Summary statistics |

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/leads` | GET | List all leads |
| `/api/leads/:id` | PUT | Update lead status |
| `/api/leads/:id` | GET | Lead detail |
