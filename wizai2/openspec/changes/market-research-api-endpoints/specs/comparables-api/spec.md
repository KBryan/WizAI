## ADDED Requirements

### Requirement: Comparable Property Search
The system SHALL provide an endpoint to search for comparable properties based on address and criteria.

#### Scenario: Search by address
- **WHEN** a GET request is made to `/api/market-research/comparables` with query parameter `address=123 Main St, Pickering`
- **THEN** the system SHALL return HTTP 200 with a list of comparable properties within default 1km radius

#### Scenario: Search with radius filter
- **WHEN** a GET request is made with `address` and `radius=2` (kilometers)
- **THEN** the system SHALL return comparables within the specified radius

#### Scenario: Filter by property type
- **WHEN** a GET request is made with `property_type=Detached`
- **THEN** the system SHALL return only detached homes as comparables

#### Scenario: Limit results
- **WHEN** a GET request is made with `limit=3`
- **THEN** the system SHALL return maximum 3 comparable properties

### Requirement: Comparable Property Details
The system SHALL return detailed information for each comparable property.

#### Scenario: Comparable details included
- **WHEN** comparables are returned in the response
- **THEN** each property SHALL include: address, MLS number, list/sold price, property type, bedrooms, bathrooms, square footage, days on market, distance from target

#### Scenario: Data source attribution
- **WHEN** comparables are returned
- **THEN** the response SHALL include a `data_source` field indicating "Repliers API" and the timestamp of data retrieval

#### Scenario: Confidence scoring
- **WHEN** comparables are matched
- **THEN** each comparable SHALL include a `match_confidence` score (0-100) based on similarity to the target property

### Requirement: Target Property Information
The system SHALL return information about the target property when searching comparables.

#### Scenario: Target property included
- **WHEN** a comparables search is performed
- **THEN** the response SHALL include a `target_property` object with the input address and any known details

#### Scenario: Property validation
- **WHEN** an invalid or non-existent address is provided
- **THEN** the system SHALL return HTTP 400 with an error message indicating the address could not be validated
