# Lead Intake

## Purpose
AI-powered lead capture and qualification system for real estate inquiries from multiple sources.

## Requirements

### Requirement: Lead Capture
The system SHALL capture lead inquiries from multiple sources (web forms, email, SMS, referrals) and store structured lead data.

#### Scenario: Web form submission
- **WHEN** a visitor submits a web form with name, email, phone, and inquiry type
- **THEN** the system SHALL create a new lead record with status "New"

#### Scenario: Email lead import
- **WHEN** an email arrives at the lead inbox from a recognized source
- **THEN** the system SHALL parse the email and create a lead with source "Email"

### Requirement: Lead Qualification
The Lead Intake AI SHALL score leads based on explicit criteria (budget, timeline, property type, location) and assign qualification scores.

#### Scenario: High-intent buyer lead
- **WHEN** a lead submits budget ($800k-$1M), timeline ("3 months"), and location ("Pickering")
- **THEN** the AI SHALL assign qualification score 8-10 and flag as "Hot Lead"

#### Scenario: Low-intent inquiry
- **WHEN** a lead submits only "just browsing" with no budget or timeline
- **THEN** the AI SHALL assign qualification score 3-5 and flag as "Nurture"

### Requirement: Lead Routing
The system SHALL route qualified leads to the human agent queue based on urgency and lead score.

#### Scenario: Hot lead routing
- **WHEN** a lead scores 8+ and requests immediate callback
- **THEN** the system SHALL add to "Priority Queue" with notification to agent

#### Scenario: Standard lead routing
- **WHEN** a lead scores 5-7 and timeline is "6+ months"
- **THEN** the system SHALL add to "Standard Queue" for follow-up within 24 hours

### Requirement: Lead Data Structure
The system SHALL maintain a structured lead profile with required and optional fields.

#### Scenario: Complete lead profile
- **WHEN** lead data includes name, contact info, source, property type, budget, timeline, notes
- **THEN** the system SHALL store all fields and mark lead as "Complete"

#### Scenario: Partial lead profile
- **WHEN** lead data includes only name and email
- **THEN** the system SHALL store available fields and flag for data enrichment
