# Tasks: Agent Dashboard UI

## 1. Project Setup

- [x] 1.1 Create `ui/` directory for Leptos WASM project
- [x] 1.2 Set up Cargo.toml with leptos, wasm-bindgen, web-sys dependencies
- [x] 1.3 Configure wasm-pack/trunk for WASM compilation
- [x] 1.4 Set up Tailwind CSS with configuration
- [x] 1.5 Create index.html entry point
- [x] 1.6 Set up main.rs with dev server entry
- [x] 1.7 Configure routing with leptos-router

## 2. Shell Layout

- [x] 2.1 Create Dashboard app component with router
- [x] 2.2 Implement Sidebar component with navigation items
- [x] 2.3 Implement Header component with agent profile
- [x] 2.4 Add WebSocket connection status indicator
- [x] 2.5 Implement Toast notification container
- [x] 2.6 Create Modal container for shared dialogs
- [x] 2.7 Add basic styling with Tailwind classes

## 3. WebSocket Client

- [x] 3.1 Implement WebSocket client module
- [x] 3.2 Add connection management with auto-reconnect
- [x] 3.3 Implement message parsing for real-time updates
- [x] 3.4 Create WebSocket state context provider
- [x] 3.5 Add reconnection logic with exponential backoff
- [x] 3.6 Display connection status in UI

## 4. API Client

- [x] 4.1 Create HTTP client module for REST calls
- [x] 4.2 Implement leads API functions (GET list, PUT update, GET detail)
- [x] 4.3 Implement CMA API functions (POST generate, GET status, GET result)
- [x] 4.4 Implement market API functions (GET trends, GET comparables, GET stats)
- [x] 4.5 Implement communications API functions (GET drafts, PUT status)
- [x] 4.6 Add error handling and retry logic
- [x] 4.7 Create loading and error states

## 5. AI Copilot Panel

- [x] 5.1 Create CopilotPanel component (collapsible, docked bottom)
- [x] 5.2 Implement ChatHistory component with scroll
- [x] 5.3 Create ChatMessage component (user/AI styles)
- [x] 5.4 Implement ChatInput with send button
- [x] 5.5 Add typing indicator animation
- [x] 5.6 Implement command suggestion chips
- [x] 5.7 Add streaming response display
- [x] 5.8 Integrate with WebSocket for AI responses
- [x] 5.9 Add minimize/expand functionality

## 6. Lead Pipeline Board

- [x] 6.1 Create PipelineBoard container component
- [x] 6.2 Implement PipelineColumn with drop zone
- [x] 6.3 Create LeadCard component with score badge
- [x] 6.4 Implement drag-and-drop for cards
- [x] 6.5 Add quick action buttons (phone, email, SMS)
- [x] 6.6 Create LeadModal for full details
- [x] 6.7 Implement lead filtering (score, source)
- [x] 6.8 Add StatsBar with summary metrics
- [x] 6.9 Connect to leads API

## 7. CMA Dashboard Widget

- [x] 7.1 Create PropertyInput form component
- [ ] 7.2 Implement address autocomplete
- [x] 7.3 Add property type selector
- [x] 7.4 Create ComparablesList component
- [x] 7.5 Implement ComparableCard with details
- [x] 7.6 Add PriceRecommendation display
- [ ] 7.7 Create simple bar chart for comparables
- [x] 7.8 Implement MarketConditions summary
- [x] 7.9 Add Export to PDF button
- [ ] 7.10 Create CmaHistory sidebar
- [x] 7.11 Connect to CMA API

## 8. Market Intelligence Module

- [x] 8.1 Create AreaSelector component
- [x] 8.2 Add PeriodSelector (30d/90d/1y toggle)
- [x] 8.3 Implement TrendChart line chart
- [x] 8.4 Create MetricsGrid with metric cards
- [x] 8.5 Add MarketStatusBadge indicator
- [x] 8.6 Implement NeighborhoodTable with sorting
- [ ] 8.7 Create InventoryChart bar chart
- [x] 8.8 Connect to market API
- [ ] 8.9 Add loading skeletons

## 9. Communication Hub Widget

- [x] 9.1 Create DraftsList component
- [x] 9.2 Implement DraftCard with status badges
- [x] 9.3 Add RiskBadge indicators
- [x] 9.4 Create DraftModal for review
- [x] 9.5 Implement ApprovalActions (Approve/Reject)
- [ ] 9.6 Add edit capability in modal
- [ ] 9.7 Create PreviewPanel for email/SMS
- [ ] 9.8 Implement ComposeModal for new drafts
- [ ] 9.9 Add TemplateSelector component
- [ ] 9.10 Create SentHistory tab/view
- [ ] 9.11 Connect to communications API

## 10. State Management

- [x] 10.1 Create Auth context provider
- [ ] 10.2 Implement Theme context (light/dark if needed)
- [x] 10.3 Add WebSocket state to context
- [x] 10.4 Create leads state management
- [x] 10.5 Create CMA state management
- [x] 10.6 Create market data state management
- [ ] 10.7 Create communications state management

## 11. Polish & Integration

- [x] 11.1 Add loading states and skeletons
- [x] 11.2 Implement error handling and toasts
- [ ] 11.3 Add keyboard shortcuts
- [x] 11.4 Ensure responsive layout (min-width for desktop)
- [x] 11.5 Add empty states for lists
- [ ] 11.6 Connect copilot commands to page actions
- [ ] 11.7 Test real-time updates via WebSocket
- [ ] 11.8 Add animations for transitions

## 12. Build & Deploy

- [ ] 12.1 Configure WASM build with trunk
- [ ] 12.2 Set up production build script
- [ ] 12.3 Integrate with existing Axum server
- [ ] 12.4 Configure CORS for API calls
- [ ] 12.5 Test locally with `trunk serve`
- [ ] 12.6 Verify all features work end-to-end
