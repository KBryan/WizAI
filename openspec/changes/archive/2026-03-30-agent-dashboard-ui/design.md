# Design: Agent Dashboard UI

## Context

WizAI is a Rust-native multi-agent framework with Leptos/WASM frontend. Currently has a basic dashboard but lacks a unified agent interface. Agents must switch between multiple tools for lead management, CMA generation, market research, and client communications.

This design covers the creation of a comprehensive Agent Dashboard that:
- Consolidates all agent operations into a single SPA
- Provides AI copilot interface for natural language commands
- Integrates with existing REST API and WebSocket endpoints
- Builds on existing Leptos/WASM architecture

## Goals / Non-Goals

**Goals:**
- Single-page application with sidebar navigation
- AI Copilot panel with streaming chat interface
- Lead pipeline Kanban board with drag-drop
- CMA generation widget with comparable visualization
- Market intelligence charts and metrics
- Communication draft management with approval workflow
- Real-time updates via existing WebSocket endpoint

**Non-Goals:**
- Mobile-responsive design (desktop-first)
- Native mobile apps
- Client-facing portal (separate future change)
- Document generation/editor (separate future change)
- Multi-tenant support

## Decisions

### 1. Frontend Framework: Leptos/WASM

**Decision:** Use Leptos for the dashboard SPA, compiled to WASM.

**Rationale:**
- Matches existing codebase architecture
- Type-safe Rust frontend/backend communication
- Fine-grained reactivity reduces unnecessary re-renders
- Small WASM bundle size for fast loading
- Strong integration with Rust backend

**Alternative:** Next.js/React was considered for faster development, but would introduce JavaScript/TypeScript divergence from existing Rust codebase.

### 2. Styling: Tailwind CSS + Custom Components

**Decision:** Use Tailwind CSS for utility styling + custom component styles.

**Rationale:**
- Rapid UI development with utility classes
- Consistent design system
- Easy to create custom components (cards, modals, buttons)
- Good Leptos integration via `leptos-tailwind` or manual classes

**Alternative:** Custom CSS modules were considered but Tailwind provides faster iteration.

### 3. Charts: Plotly or Custom Canvas

**Decision:** Use `plotters` crate for Rust-based charting compiled to WASM, or lightweight JS charting library loaded via web worker.

**Rationale:**
- `plotters` keeps everything in Rust ecosystem
- WASM-based charts have good performance
- Fallback: simple SVG generation for basic charts

**Alternative:** Chart.js via CDN was considered but adds external JS dependency.

### 4. Drag-and-Drop: Custom Implementation

**Decision:** Implement custom drag-and-drop for Kanban using Leptos signals and mouse events.

**Rationale:**
- Avoids adding heavy JS library
- Full control over UX
- Leptos signals make state management natural

**Alternative:** `dnd-kit` via web worker was considered but adds complexity.

### 5. Real-time Updates: WebSocket

**Decision:** Use existing WebSocket endpoint (`ws://localhost:3000/ws`) for real-time updates.

**Rationale:**
- Already implemented in backend
- Efficient for live status updates
- Single connection for all real-time features

**Implementation:**
- Reconnect logic with exponential backoff
- Connection status indicator in UI
- Message queue during reconnection

### 6. State Management: Leptos Signals + Context

**Decision:** Use Leptos signals for component state, context for global state (auth, theme, WebSocket).

**Rationale:**
- Leptos signals are reactive and fine-grained
- Context API provides dependency injection pattern
- No external state library needed

### 7. API Communication: ReST + Fetch

**Decision:** Use `reqwest` or `fetch` API from WASM for REST calls.

**Rationale:**
- `reqwest` is already likely in Cargo.toml for backend
- WASM has built-in `fetch` API
- Simple JSON serialization with serde

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Agent Dashboard (WASM)                 │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ Shell Layout                                             │ │
│  │  ┌─────────┐ ┌─────────────────────────────────────────┐ │ │
│  │  │ Sidebar │ │ Main Content Area                        │ │ │
│  │  │         │ │  ┌─────────────────────────────────────┐  │ │ │
│  │  │ • Home  │ │  │ AI Copilot Panel                   │  │ │ │
│  │  │ • Leads │ │  │ (Collapsible, docked bottom)       │  │ │ │
│  │  │ • CMA   │ │  └─────────────────────────────────────┘  │ │ │
│  │  │ • Market│ │  ┌─────────────────────────────────────┐  │ │ │
│  │  │ • Comms │ │  │ Module Content                      │  │ │ │
│  │  │         │ │  │ (Leads Pipeline / CMA / etc.)       │  │ │ │
│  │  └─────────┘ │  └─────────────────────────────────────┘  │ │ │
│  └─────────────┴───────────────────────────────────────────┘ │ │
└─────────────────────────────────────────────────────────────┘
                         │
                         │ WebSocket + REST API
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum Server (Existing)                   │
│  • /api/leads                                                │
│  • /api/cmas                                                 │
│  • /api/market/*                                             │
│  • /api/communications                                       │
│  • /ws (WebSocket for real-time)                            │
└─────────────────────────────────────────────────────────────┘
```

## Component Hierarchy

```
Dashboard
├── Sidebar
│   ├── Logo
│   ├── NavItems (Home, Leads, CMA, Market, Comms, Settings)
│   └── ConnectionStatus
├── Header
│   ├── PageTitle
│   ├── AgentProfile
│   └── Notifications
├── Router (leptos-router)
│   ├── HomePage
│   ├── LeadsPage
│   │   ├── LeadStats
│   │   ├── PipelineBoard
│   │   │   └── LeadCard (draggable)
│   │   └── LeadDetailModal
│   ├── CmaPage
│   │   ├── PropertyInput
│   │   ├── ComparablesList
│   │   └── PriceRecommendationChart
│   ├── MarketPage
│   │   ├── AreaSelector
│   │   ├── TrendCharts
│   │   └── InventoryMetrics
│   └── CommsPage
│       ├── DraftsList
│       └── DraftApprovalWorkflow
└── CopilotPanel (global, collapsible)
    ├── ChatHistory
    ├── ChatInput
    └── CommandSuggestions
```

## API Endpoints Used

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/leads` | GET | List leads with filters |
| `/api/leads/:id` | GET/PUT | Lead detail/update |
| `/api/leads` | POST | Create lead |
| `/api/cmas` | GET | List CMAs |
| `/api/cmas` | POST | Generate CMA |
| `/api/cmas/:id` | GET | CMA detail |
| `/api/market/comparables` | GET | Search comparables |
| `/api/market/trends/:area` | GET | Market trends |
| `/api/communications` | GET | List drafts |
| `/api/communications/:id` | PUT | Update draft status |
| `/ws` | WS | Real-time updates |

## File Structure

```
ui/                          # New dashboard UI
├── Cargo.toml               # WASM crate
├── src/
│   ├── lib.rs               # WASM entry point
│   ├── main.rs              # Dev server entry
│   ├── app.rs               # Main app component
│   ├── router.rs            # leptos-router setup
│   ├── components/
│   │   ├── shell.rs         # Layout shell
│   │   ├── sidebar.rs       # Navigation sidebar
│   │   ├── header.rs        # Top header
│   │   ├── copilot.rs       # AI chat panel
│   │   ├── leads/
│   │   │   ├── pipeline.rs  # Kanban board
│   │   │   ├── card.rs      # Lead card
│   │   │   └── detail.rs    # Lead detail modal
│   │   ├── cma/
│   │   │   ├── input.rs     # Property input
│   │   │   ├── comparables.rs
│   │   │   └── chart.rs     # Price chart
│   │   ├── market/
│   │   │   ├── trends.rs    # Trend charts
│   │   │   └── metrics.rs   # Stats cards
│   │   └── comms/
│   │       ├── drafts.rs    # Drafts list
│   │       └── approval.rs  # Approval workflow
│   ├── api/
│   │   ├── client.rs        # HTTP client
│   │   ├── leads.rs         # Lead API
│   │   ├── cma.rs           # CMA API
│   │   └── market.rs        # Market API
│   ├── ws/
│   │   └── client.rs        # WebSocket client
│   ├── state/
│   │   ├── auth.rs          # Auth context
│   │   ├── theme.rs         # Theme context
│   │   └── websocket.rs     # WS connection state
│   └── styles/
│       └── main.css         # Tailwind + custom styles
├── index.html               # Entry HTML
├── build.sh                 # WASM build script
└── tailwind.config.js       # Tailwind configuration
```

## Risks / Trade-offs

| Risk | Impact | Mitigation |
|------|--------|-----------|
| WASM bundle size large | Initial load slow | Lazy load modules, code splitting |
| WebSocket reconnection | Brief UI staleness | Visual indicator, message queue |
| Leptos learning curve | Development speed | Reference existing codebase patterns |
| Chart library complexity | Development time | Start with simple SVG, add charts later |
| Drag-drop on touch | No mobile support | Accept non-goal, can add later |
| API rate limiting | Requests fail | Implement client-side retry with backoff |

## Migration Plan

**Phase 1: Shell + Home**
1. Create `ui/` directory with Leptos project scaffold
2. Set up Tailwind CSS
3. Build basic layout shell with sidebar
4. Add placeholder home page

**Phase 2: Leads Pipeline**
1. Implement API client
2. Build lead listing and filtering
3. Create Kanban board with drag-drop
4. Add lead detail modal

**Phase 3: CMA Widget**
1. Property address input form
2. Comparables list display
3. Simple price visualization
4. CMA generation flow

**Phase 4: Market Intelligence**
1. Area selector component
2. Trend data fetching
3. Basic chart implementation
4. Metrics dashboard

**Phase 5: AI Copilot**
1. WebSocket client implementation
2. Chat interface component
3. Command parsing
4. Streaming response display

**Phase 6: Communication Hub**
1. Drafts list display
2. Approval workflow UI
3. Status updates

## Open Questions

1. **Authentication**: How should the dashboard authenticate? Token in localStorage? Cookie-based?
2. **Role-based views**: Should agents see different features than admins?
3. **Notification preferences**: Push notifications for hot leads?
4. **Export formats**: PDF for CMAs? What else?
5. **Performance targets**: What's acceptable WASM load time?
