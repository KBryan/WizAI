# Spec: Communication Hub Widget

## Purpose

Interface for managing AI-generated communication drafts (emails, SMS) with approval workflow and history.

## Requirements

### Requirement: Drafts List

The hub SHALL display a list of communication drafts.

#### Scenario: Display drafts
- **WHEN** agent opens Communications page
- **THEN** drafts SHALL display in a list/table
- **AND** each SHALL show: type (Email/SMS), recipient, subject/preview, status, risk level, date
- **AND** pending drafts SHALL appear at top

#### Scenario: Draft status badges
- **WHEN** a draft is displayed
- **THEN** status SHALL show as colored badge:
  - Draft: Gray
  - Pending Approval: Yellow
  - Approved: Green
  - Sent: Blue
  - Rejected: Red

### Requirement: Risk Classification Display

Drafts SHALL display their risk classification.

#### Scenario: Risk indicators
- **WHEN** a draft is displayed
- **THEN** risk level SHALL show as badge:
  - Low Risk: Green (appointment reminders, general info)
  - Medium Risk: Yellow (follow-ups, general communications)
  - High Risk: Red (price discussions, market value)

#### Scenario: Risk explanation
- **WHEN** agent hovers over risk badge
- **THEN** a tooltip SHALL explain why it's classified as such

### Requirement: Approval Workflow UI

The hub SHALL provide approval workflow for drafts.

#### Scenario: Review pending draft
- **WHEN** agent clicks on a pending draft
- **THEN** a modal SHALL open with full draft content
- **AND** edit capability SHALL be available
- **AND** Approve/Reject buttons SHALL appear

#### Scenario: Approve draft
- **WHEN** agent clicks "Approve"
- **THEN** status SHALL update to "Approved"
- **AND** draft SHALL be marked "Ready to Send"
- **AND** confirmation toast SHALL appear

#### Scenario: Reject draft
- **WHEN** agent clicks "Reject"
- **THEN** a reason field SHALL appear
- **AND** agent SHALL enter rejection reason
- **AND** status SHALL update to "Rejected"
- **AND** rejection SHALL be logged

#### Scenario: Edit and approve
- **WHEN** agent edits draft content
- **THEN** a "Save & Approve" button SHALL appear
- **AND** clicking SHALL save edits and approve

### Requirement: Draft Preview

The hub SHALL provide preview functionality.

#### Scenario: Email preview
- **WHEN** agent previews an email draft
- **THEN** a render SHALL show exactly how it will appear
- **AND** agent SHALL be able to see formatting, links

#### Scenario: SMS preview
- **WHEN** agent previews an SMS draft
- **THEN** a phone mockup SHALL show character count
- **AND** length warnings SHALL appear if > 160 chars

### Requirement: Communication History

The hub SHALL show sent communication history.

#### Scenario: View sent
- **WHEN** agent clicks "Sent" tab
- **THEN** sent communications SHALL display
- **AND** each SHALL show: recipient, content preview, sent date, status

#### Scenario: Resend/edit copy
- **WHEN** agent wants to resend
- **THEN** clicking "Resend" SHALL open draft with original content
- **AND** agent SHALL be able to edit and send again

### Requirement: Quick Compose

The hub SHALL support quick draft creation.

#### Scenario: New email draft
- **WHEN** agent clicks "New Email"
- **THEN** a compose modal SHALL open
- **AND** recipient, template selection, personalization fields SHALL appear
- **AND** AI SHALL generate draft based on inputs

#### Scenario: Template selection
- **WHEN** agent selects a template
- **THEN** AI SHALL fill template with lead data
- **AND** draft SHALL be ready for review

## UI Components

| Component | Description |
|-----------|-------------|
| DraftsList | Main drafts table/list |
| DraftCard | Individual draft row |
| DraftModal | Full draft view/edit modal |
| ApprovalActions | Approve/Reject buttons |
| RiskBadge | Risk level indicator |
| StatusBadge | Draft status badge |
| PreviewPanel | Email/SMS preview |
| ComposeModal | New draft creation |
| TemplateSelector | Template picker |
| SentHistory | Tab/view for sent items |

## State

| State | Type | Description |
|-------|------|-------------|
| `drafts` | Signal<Vec<CommunicationDraft>> | All drafts |
| `selected_draft` | Signal<Option<Draft>> | Modal open state |
| `filter_status` | Signal<StatusFilter> | Active status filter |
| `sent_history` | Signal<Vec<SentItem>> | Sent communications |

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/communications` | GET | List drafts |
| `/api/communications/:id` | GET | Draft detail |
| `/api/communications/:id` | PUT | Update draft/approve/reject |
| `/api/communications/:id/send` | POST | Send draft |
| `/api/communications/draft` | POST | Create new draft |

## Templates (Future)

| Template | Risk Level | Use Case |
|----------|------------|----------|
| Initial Contact | Medium | First response to inquiry |
| Property Inquiry | Medium | Follow-up on listing |
| Showing Confirmation | Low | Appointment reminder |
| Offer Follow-up | High | Price negotiation |
| Closing Reminder | Low | Task reminder |
| Market Update | Medium | Periodic client update |
