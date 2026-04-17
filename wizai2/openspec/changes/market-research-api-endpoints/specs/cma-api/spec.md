## ADDED Requirements

### Requirement: CMA Generation Request
The system SHALL accept CMA generation requests via REST API and return an async job identifier.

#### Scenario: Valid CMA request
- **WHEN** a POST request is made to `/api/market-research/cma` with valid property data including address and property type
- **THEN** the system SHALL return HTTP 202 with a CMA job ID and status "processing"

#### Scenario: Invalid property data
- **WHEN** a POST request is made with missing or invalid address
- **THEN** the system SHALL return HTTP 400 with error details indicating the validation failure

#### Scenario: Unauthorized request
- **WHEN** a CMA request is made without valid authentication
- **THEN** the system SHALL return HTTP 401 Unauthorized

### Requirement: CMA Status Tracking
The system SHALL provide endpoints to track CMA generation progress.

#### Scenario: Check processing status
- **WHEN** a GET request is made to `/api/market-research/cma/{id}/status` for an in-progress CMA
- **THEN** the system SHALL return the current status ("queued", "processing", "analyzing_comparables", "generating_report") and progress percentage

#### Scenario: Check completed status
- **WHEN** a GET request is made to `/api/market-research/cma/{id}/status` for a completed CMA
- **THEN** the system SHALL return status "completed" and a link to view the full report

#### Scenario: Check non-existent CMA
- **WHEN** a GET request is made for a CMA ID that does not exist
- **THEN** the system SHALL return HTTP 404 Not Found

### Requirement: CMA Report Retrieval
The system SHALL allow retrieval of completed CMA reports via REST API.

#### Scenario: Retrieve completed report
- **WHEN** a GET request is made to `/api/market-research/cma/{id}` for a completed CMA
- **THEN** the system SHALL return HTTP 200 with the full CMA report including estimated value range, comparables, and market analysis

#### Scenario: Retrieve incomplete report
- **WHEN** a GET request is made for a CMA that is still processing
- **THEN** the system SHALL return HTTP 202 with current status and estimated completion time

#### Scenario: Include approval status
- **WHEN** a CMA report is retrieved
- **THEN** the response SHALL include the approval status ("pending_review", "approved", "rejected") indicating if the human agent has reviewed the AI-generated report

### Requirement: CMA List Retrieval
The system SHALL provide endpoints to list CMAs for the authenticated agent.

#### Scenario: List all CMAs
- **WHEN** a GET request is made to `/api/market-research/cma`
- **THEN** the system SHALL return a paginated list of CMAs created by the authenticated agent

#### Scenario: Filter by status
- **WHEN** a GET request is made with query parameter `status=completed`
- **THEN** the system SHALL return only CMAs with status "completed"

#### Scenario: Sort by date
- **WHEN** a GET request is made with query parameter `sort=created_at:desc`
- **THEN** the system SHALL return CMAs sorted by creation date descending
