## ADDED Requirements

### Requirement: Lead CRUD Operations
The system SHALL provide Create, Read, Update, and Delete operations for lead records.

#### Scenario: Create lead
- **WHEN** a POST request is made to /api/leads with valid lead data
- **THEN** the system SHALL create a lead record and return the lead ID

#### Scenario: Get lead
- **WHEN** a GET request is made to /api/leads/:id
- **THEN** the system SHALL return the complete lead record with all fields

#### Scenario: Update lead
- **WHEN** a PUT request is made to /api/leads/:id with updated fields
- **THEN** the system SHALL update the lead record and log the changes

#### Scenario: Delete lead (soft delete)
- **WHEN** a DELETE request is made to /api/leads/:id
- **THEN** the system SHALL mark the lead as "Archived" and preserve the data

### Requirement: Lead Listing and Filtering
The system SHALL support listing leads with filtering and sorting capabilities.

#### Scenario: List all leads
- **WHEN** a GET request is made to /api/leads
- **THEN** the system SHALL return a paginated list of all non-archived leads

#### Scenario: Filter by status
- **WHEN** a GET request is made with query param "status=New"
- **THEN** the system SHALL return only leads with status "New"

#### Scenario: Sort by qualification score
- **WHEN** a GET request is made with query param "sort=score:desc"
- **THEN** the system SHALL return leads sorted by qualification score descending

### Requirement: Lead Scoring Algorithm
The system SHALL calculate lead scores based on explicit criteria.

#### Scenario: Calculate buyer score
- **WHEN** a lead indicates "Buyer" intent with budget, timeline, and location
- **THEN** the system SHALL calculate a score from 0-10 based on: budget clarity (3 pts), timeline (3 pts), location specificity (2 pts), financing status (2 pts)

#### Scenario: Calculate seller score
- **WHEN** a lead indicates "Seller" intent with property details and timeline
- **THEN** the system SHALL calculate a score from 0-10 based on: property type known (2 pts), location known (2 pts), timeline (3 pts), motivation (3 pts)

### Requirement: Lead Status Management
The system SHALL support a status workflow for leads.

#### Scenario: Status transitions
- **WHEN** a lead status changes
- **THEN** the system SHALL validate the transition is allowed: New → Qualified → Contacted → Converted/Archived

#### Scenario: Status history
- **WHEN** a lead status changes
- **THEN** the system SHALL record the change with timestamp and reason

### Requirement: Lead Source Tracking
The system SHALL track and report lead sources.

#### Scenario: Source attribution
- **WHEN** a lead is created
- **THEN** the system SHALL record the source (Website, Referral, Portal, Email, SMS, Walk-in, Other)

#### Scenario: Source analytics
- **WHEN** an agent requests lead analytics
- **THEN** the system SHALL provide counts and conversion rates by source
