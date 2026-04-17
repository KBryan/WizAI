# Spec: Agent Dashboard Shell

## Purpose

Main layout shell providing navigation, authentication state, WebSocket connection status, and routing for all dashboard modules.

## Requirements

### Requirement: Layout Shell

The dashboard SHALL provide a consistent layout shell with sidebar navigation and main content area.

#### Scenario: Initial load
- **WHEN** an agent navigates to the dashboard
- **THEN** the shell SHALL render sidebar navigation on the left
- **AND** the main content area on the right
- **AND** a collapsible AI copilot panel docked at the bottom

#### Scenario: Sidebar navigation
- **WHEN** an agent clicks a nav item (Leads, CMA, Market, Comms)
- **THEN** the main content area SHALL route to the corresponding page
- **AND** the active nav item SHALL be highlighted

### Requirement: WebSocket Connection Status

The dashboard SHALL display WebSocket connection status and auto-reconnect on disconnect.

#### Scenario: Connected state
- **WHEN** the WebSocket successfully connects
- **THEN** the status indicator SHALL show "Connected" (green)
- **AND** real-time updates SHALL be enabled

#### Scenario: Disconnected state
- **WHEN** the WebSocket disconnects (network issue, server restart)
- **THEN** the status indicator SHALL show "Disconnected" (red)
- **AND** auto-reconnect SHALL begin with exponential backoff
- **AND** a toast notification SHALL appear

#### Scenario: Reconnecting state
- **WHEN** attempting to reconnect after disconnect
- **THEN** the status indicator SHALL show "Reconnecting..." (yellow)
- **AND** retries SHALL occur at 1s, 2s, 4s, 8s intervals (max 30s)

### Requirement: Agent Profile Display

The dashboard SHALL display the logged-in agent's profile in the header.

#### Scenario: Profile display
- **WHEN** an agent is authenticated
- **THEN** the header SHALL show agent name and avatar
- **AND** clicking the profile SHALL show a dropdown with settings/logout

### Requirement: Page Routing

The dashboard SHALL support client-side routing for all pages.

#### Scenario: Route to leads page
- **WHEN** agent clicks "Leads" in sidebar
- **THEN** the URL SHALL change to `/leads`
- **AND** the LeadsPage component SHALL render

#### Scenario: Direct URL access
- **WHEN** agent navigates directly to `/cma`
- **THEN** the CMAPage component SHALL render

## UI Components

| Component | Description |
|-----------|-------------|
| Sidebar | Vertical nav with icons and labels |
| Header | Agent profile, notifications, connection status |
| CopilotPanel | Collapsible AI chat panel (global) |
| ToastContainer | Notification toasts |
| ModalContainer | Modal dialogs (shared) |

## State

| State | Type | Description |
|-------|------|-------------|
| `ws_connected` | Signal<bool> | WebSocket connection state |
| `current_route` | Signal<Route> | Active page route |
| `agent_profile` | Signal<Option<Agent>> | Authenticated agent |
| `copilot_open` | Signal<bool> | Copilot panel visibility |
| `toasts` | Signal<Vec<Toast>> | Active notifications |
