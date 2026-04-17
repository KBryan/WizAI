# Proposal: AI Real Estate Team MVP

## Why

Real estate agents in the Durham Region lose hours daily to repetitive tasks (follow-up, scheduling, drafting listing copy, CMAs, client updates) that could be spent on revenue-generating activities (winning listings, negotiating deals, closing transactions). Traditional team expansion is expensive and difficult to scale for solo agents or small teams. This change builds an AI-assisted operating system where a licensed human agent supervises a team of specialised AI workers, providing leverage before justifying full-time hires.

## What Changes

1. **Lead Intake Specialist AI**: Automated lead capture, qualification, scoring, and routing from multiple sources (web forms, email, SMS, referrals)
2. **Client Communication Assistant AI**: Draft generation for emails/SMS follow-ups with human approval workflow and risk classification
3. **Market Research Analyst AI Enhancement**: CMA generation, comparable sales analysis, buyer/listing support research
4. **Approval & Compliance Layer**: Risk-based approval system (Low/Medium/High), audit trail logging, human-in-the-loop workflow
5. **Lead Management Service**: CRUD operations, lead scoring, routing logic, SQLite persistence
6. **Communication Service**: Draft generation, template management, approval queue, edit/reject/approve workflow
7. **Durham Region Specialization**: Toronto/Durham market data via Repliers API, TREB compliance templates
8. **Human Agent Dashboard**: Internal UI for reviewing AI outputs, approvals, and agent supervision

**Key Constraints**:
- Internal-only (AI assists human agent, not client-facing)
- All pricing/negotiation/legal advice requires human review
- All AI-generated content marked as draft pending approval

## Capabilities

### New Capabilities

- `lead-intake`: Lead capture, qualification, scoring, and routing
- `client-communication`: Draft generation with approval workflow and risk classification
- `market-research-cma`: Comparative market analysis generation and comparable sales analysis
- `approval-workflow`: Risk assessment, approval queue, audit logging, human-in-the-loop
- `lead-management`: Lead CRUD, scoring algorithm, database persistence
- `compliance-audit`: Audit trail, compliance logging, review tracking

### Modified Capabilities

- `real-estate-researcher`: Enhancement to existing agent role with CMA support and buyer/listing workflows

## Impact

**Code Areas**:
- New: `src/models/` (lead, communication, market data structures)
- New: `src/agent/roles/` (lead_intake_specialist, client_communication_assistant)
- New: `src/services/` (lead_service, communication_service)
- New: `src/compliance/` (approval_engine, audit logging)
- Modified: `src/agent/roles/real_estate_researcher.rs` (add CMA, buyer support)
- Modified: `src/agent/roles/mod.rs` (new AgentRole variants)
- Modified: `src/server/api.rs` (new REST endpoints)
- Modified: `src/memory/` (new tables for leads, communications)

**APIs**:
- `POST /api/leads` - Create lead
- `GET /api/leads` - List leads  
- `POST /api/leads/:id/qualify` - AI qualification
- `POST /api/communications/draft` - Generate draft
- `POST /api/communications/:id/approve` - Approve draft
- `POST /api/research/cma` - Generate CMA
- `POST /api/research/comparables` - Find comparables

**Dependencies**:
- Repliers API (existing MCP integration)
- Venice AI (existing LLM client)
- SQLite (existing memory store)
- MCP Browser tools (for sentiment analysis)

**Compliance Considerations**:
- RECO (Real Estate Council of Ontario) guidelines
- TREB (Toronto Real Estate Board) fair housing rules
- Clear AI disclosure in all generated content
- Human approval for Medium/High risk communications
