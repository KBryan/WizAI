## 1. Foundation & Data Models

- [ ] 1.1 Create `src/models/mod.rs` with module exports
- [ ] 1.2 Create `src/models/lead.rs` with Lead, LeadStatus, LeadSource structs
- [ ] 1.3 Create `src/models/communication.rs` with CommunicationDraft, ApprovalStatus, RiskLevel
- [ ] 1.4 Create `src/models/market.rs` with CMAReport, Comparable, MarketData structs
- [ ] 1.5 Run database migration to create leads table
- [ ] 1.6 Run database migration to create communications table
- [ ] 1.7 Run database migration to create audit_logs table
- [ ] 1.8 Verify all models compile and tests pass

## 2. Lead Intake Specialist AI

- [ ] 2.1 Create `src/agent/roles/lead_intake_specialist.rs` module
- [ ] 2.2 Implement `LeadIntakeSpecialist` struct with AgentExecutor
- [ ] 2.3 Implement `capture_lead()` method for processing lead data
- [ ] 2.4 Implement `qualify_lead()` with scoring algorithm (0-10 scale)
- [ ] 2.5 Implement `route_lead()` for queue assignment (Priority/Standard)
- [ ] 2.6 Add LeadIntakeSpecialist role to AgentRole enum
- [ ] 2.7 Create system prompt for Lead Intake Specialist
- [ ] 2.8 Write unit tests for lead qualification logic
- [ ] 2.9 Integrate lead creation with memory store

## 3. Client Communication Assistant AI

- [ ] 3.1 Create `src/agent/roles/client_communication_assistant.rs` module
- [ ] 3.2 Implement `ClientCommunicationAssistant` struct
- [ ] 3.3 Implement `draft_follow_up()` for email/SMS generation
- [ ] 3.4 Implement `summarize_conversation()` using memory store
- [ ] 3.5 Implement `propose_next_action()` based on lead status
- [ ] 3.6 Add ClientCommunicationAssistant role to AgentRole enum
- [ ] 3.7 Create system prompt emphasizing compliance (no pricing/negotiations)
- [ ] 3.8 Write unit tests for draft generation

## 4. Market Research Analyst Enhancement

- [ ] 4.1 Extend `src/agent/roles/real_estate_researcher.rs`
- [ ] 4.2 Implement `generate_cma()` method with comparable selection
- [ ] 4.3 Implement `analyze_comparables_for_listing()` method
- [ ] 4.4 Implement `analyze_buyer_preferences()` for shortlist comparison
- [ ] 4.5 Add CMA-specific prompts with Durham Region focus
- [ ] 4.6 Integrate market sentiment analysis (browser MCP)
- [ ] 4.7 Write unit tests for CMA generation
- [ ] 4.8 Verify Repliers API integration works for Durham data

## 5. Approval & Compliance Layer

- [ ] 5.1 Create `src/compliance/mod.rs` module structure
- [ ] 5.2 Create `src/compliance/approval_engine.rs` with risk classification
- [ ] 5.3 Implement `classify_risk()` method analyzing content
- [ ] 5.4 Implement `ApprovalQueue` struct with CRUD operations
- [ ] 5.5 Implement approve/reject/edit actions with audit logging
- [ ] 5.6 Create `src/compliance/audit_logger.rs` for immutable logs
- [ ] 5.7 Implement audit trail query interface
- [ ] 5.8 Add fair housing screening to risk classification
- [ ] 5.9 Write unit tests for risk classification logic

## 6. Lead Management Service

- [ ] 6.1 Create `src/services/mod.rs` module exports
- [ ] 6.2 Create `src/services/lead_service.rs`
- [ ] 6.3 Implement `create_lead()` with validation
- [ ] 6.4 Implement `get_lead()` by ID
- [ ] 6.5 Implement `update_lead()` with change logging
- [ ] 6.6 Implement `list_leads()` with filtering and pagination
- [ ] 6.7 Implement lead scoring calculation algorithm
- [ ] 6.8 Implement soft delete (archive) functionality
- [ ] 6.9 Write integration tests for lead service

## 7. Communication Service

- [ ] 7.1 Create `src/services/communication_service.rs`
- [ ] 7.2 Implement `generate_draft()` calling ClientCommunicationAssistant
- [ ] 7.3 Implement template management system
- [ ] 7.4 Integrate with approval workflow (submit to queue)
- [ ] 7.5 Implement conversation history retrieval
- [ ] 7.6 Write integration tests for communication service

## 8. API Endpoints

- [ ] 8.1 Extend `src/server/api.rs` with lead endpoints
- [ ] 8.2 Implement POST /api/leads endpoint
- [ ] 8.3 Implement GET /api/leads endpoint with pagination
- [ ] 8.4 Implement GET /api/leads/:id endpoint
- [ ] 8.5 Implement POST /api/leads/:id/qualify endpoint
- [ ] 8.6 Implement POST /api/communications/draft endpoint
- [ ] 8.7 Implement GET /api/communications/pending endpoint
- [ ] 8.8 Implement POST /api/communications/:id/approve endpoint
- [ ] 8.9 Implement POST /api/research/cma endpoint
- [ ] 8.10 Implement POST /api/research/comparables endpoint
- [ ] 8.11 Write API integration tests

## 9. Database & Persistence

- [ ] 9.1 Create migration for leads table (id, contact_info, source, status, score, etc.)
- [ ] 9.2 Create migration for communications table (id, draft_content, risk_level, status, etc.)
- [ ] 9.3 Create migration for audit_logs table (id, action, timestamp, agent_id, etc.)
- [ ] 9.4 Create migration for approval_queue table (id, draft_id, risk_level, status, etc.)
- [ ] 9.5 Run migrations and verify schema
- [ ] 9.6 Add database indices for performance

## 10. Testing & Verification

- [ ] 10.1 Run `cargo test` and fix any compilation errors
- [ ] 10.2 Run `cargo clippy` and fix linting issues
- [ ] 10.3 Test lead intake flow end-to-end
- [ ] 10.4 Test communication draft generation and approval flow
- [ ] 10.5 Test CMA generation with live Repliers data
- [ ] 10.6 Test compliance audit trail logging
- [ ] 10.7 Verify all specs have passing tests
- [ ] 10.8 Performance test: lead qualification < 30s, CMA < 2min

## 11. Documentation

- [ ] 11.1 Update README.md with new AI team roles
- [ ] 11.2 Document API endpoints in docs/api.md
- [ ] 11.3 Document approval workflow in docs/compliance.md
- [ ] 11.4 Create usage examples for lead management
- [ ] 11.5 Create usage examples for communication drafting
- [ ] 11.6 Document Durham Region market specialization

## 12. Final Verification

- [ ] 12.1 Run full test suite: `cargo test`
- [ ] 12.2 Verify no breaking changes to existing Spree functionality
- [ ] 12.3 Test WebSocket real-time updates still work
- [ ] 12.4 Verify AI roles respect approval boundaries
- [ ] 12.5 Confirm audit logs capture all required events
- [ ] 12.6 Final code review for compliance safeguards
- [ ] 12.7 Update COMPLETION_SUMMARY.md with implementation details
