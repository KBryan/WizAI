# Proposal: Agent Dashboard UI

## Why

Real estate agents waste 40%+ of their time on administrative tasks: switching between CRM, MLS portals, email, and messaging apps. An integrated AI-powered agent dashboard consolidates all core functions (leads, CMAs, market data, communications) into a single interface with AI copilot capabilities, enabling agents to focus on closing deals rather than managing tools.

## What Changes

- **New Leptos/WASM Agent Dashboard** - Single-page application replacing fragmented tools
- **AI Copilot Panel** - Natural language interface for all agent operations
- **Lead Pipeline Board** - Visual Kanban with drag-drop, scoring, and quick actions
- **CMA Widget** - Interactive comparative market analysis with live MLS data
- **Market Intelligence Module** - Area trends, inventory metrics, neighborhood breakdowns
- **Communication Hub** - Draft emails/SMS with approval workflow display
- **Real-time WebSocket Updates** - Live status changes without page refresh

## Capabilities

### New Capabilities

- `agent-dashboard-shell`: Main layout with sidebar navigation, auth state, WebSocket status
- `ai-copilot-panel`: Chat interface with streaming AI responses, command palette, context-aware suggestions
- `lead-pipeline-board`: Kanban view with lead cards, scoring heatmap, quick action buttons
- `cma-dashboard-widget`: Property input, comparable list, price recommendation chart, export
- `market-intelligence-module`: Area trends, charts, inventory metrics display
- `communication-hub-widget`: Email/SMS drafts display with approval workflow status

### Modified Capabilities

(None - new dashboard doesn't change existing spec requirements)

## Impact

- **New Frontend**: `ui/` directory with Leptos components
- **New Dependencies**: Chart library, drag-drop library
- **Existing Backend**: Leverages existing REST API and WebSocket endpoints
- **No Breaking Changes**: Dashboard is additive, existing APIs unchanged
