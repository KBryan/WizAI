# Design: AI Real Estate Team MVP

## Context

The Spree Agent Framework already provides a solid foundation: hierarchical agent system, SQLite memory, Venice AI LLM integration, tool registry, WebSocket API, and a WASM UI. The existing `RealEstateResearcher` role demonstrates MCP integration with the Repliers API for Durham Region market data.

This design extends the framework to create an AI Real Estate Team with three core AI roles working under human agent supervision, with strict compliance controls and internal-only interfaces.

## Goals / Non-Goals

**Goals:**
- Create 3 specialized AI agents (Lead Intake, Communication, Market Research) with clear responsibilities
- Implement risk-based approval workflow (Low/Medium/High) for all AI-generated client communications
- Build audit trail system for compliance tracking
- Integrate Durham Region market data via existing Repliers MCP
- Ensure all regulated advice flows through human agent approval
- Provide fast response times (< 30s for drafts, < 2min for CMAs)

**Non-Goals:**
- Client-facing AI chatbots (internal-only for MVP)
- Direct contract editing or negotiation by AI
- Voice AI calling capabilities
- Multi-market support beyond Durham/Toronto
- Autonomous social media posting
- Advanced predictive pricing models

## Decisions

### Decision: Single-Process Architecture (Keep Current)
**Rationale**: The existing Spree architecture uses a single Rust process with async Tokio. This is sufficient for MVP and avoids distributed system complexity.
**Alternative Considered**: Microservices architecture - rejected for MVP complexity

### Decision: Extend Existing Agent System vs New Framework
**Rationale**: Leverage existing `AgentRole` enum, `AgentRegistry`, and `AgentExecutor`. Add new role variants to the hierarchy rather than creating parallel systems.
**Alternative Considered**: Separate "team" system - rejected to avoid code duplication

### Decision: Risk-Based Approval Levels
**Rationale**: Not all AI outputs require the same scrutiny. Categorize into:
- **Low Risk**: Appointment reminders, internal summaries (auto-prepared, human can review)
- **Medium Risk**: Client follow-ups, listing copy drafts (requires approval before sending)
- **High Risk**: Pricing advice, contract language, negotiations (requires explicit approval + agent acknowledgment)
**Alternative Considered**: All AI outputs require approval - rejected as too slow for practical use

### Decision: SQLite for Lead/Communication Storage
**Rationale**: Use existing SQLite infrastructure. Add new tables rather than external database.
**Alternative Considered**: PostgreSQL - rejected to maintain zero-dependency deployment

### Decision: Prompt-Based AI vs Fine-Tuned Models
**Rationale**: Use Venice AI with carefully crafted system prompts containing real estate context, compliance rules, and Durham market knowledge. No fine-tuning required for MVP.
**Alternative Considered**: Fine-tuned models - rejected for cost and time

### Decision: Durham Region Specialization
**Rationale**: Focus prompts and examples on Toronto/Durham market (Pickering, Ajax, Whitby, Oshawa, Clarington, Scugog, Uxbridge, Brock Township). Deep local knowledge in prompts beats generic real estate advice.
**Alternative Considered**: Generic real estate - rejected, local expertise is a differentiator

### Decision: MCP Tools for Market Data
**Rationale**: Continue using Repliers MCP for MLS data. Add new MCP tools for approval workflow operations rather than hardcoding in Rust.
**Alternative Considered**: Direct Repliers API calls - MCP provides better abstraction

## Risks / Trade-offs

**Risk: AI Hallucination in Pricing Advice**
→ **Mitigation**: Never auto-send pricing. All CMAs and market reports are agent-facing only. Human agent reviews before client presentation.

**Risk: Compliance Violation (RECO/TREB)**
→ **Mitigation**: Hardcoded compliance rules in prompts. "High Risk" classification for regulated topics. Audit trail logs every AI action. Clear AI disclosure headers on all drafts.

**Risk: User Experience Too Slow**
→ **Mitigation**: Stream LLM responses (existing Venice client supports this). Cache market data for 1 hour. Low-risk drafts auto-prepared in background.

**Risk: Lead Routing Errors**
→ **Mitigation**: Simple scoring algorithm based on explicit fields (budget, timeline, type). Human agent can override. All leads go to queue, not auto-contacted.

**Risk: Prompt Injection in Lead Forms**
→ **Mitigation**: Sanitize all input fields. Never execute user input as system commands. Treat all lead data as untrusted.

**Risk: Over-Reliance on AI**
→ **Mitigation**: Clear UI indicators: "AI Draft - Review Before Sending". Human agent must approve every external communication. AI cannot bypass approval queue.

## Migration Plan

**Deployment Steps:**
1. Run database migrations to create leads/communications tables
2. Deploy new agent roles to existing Spree instance
3. Create initial human agent user with full approval permissions
4. Import existing leads (if any) via batch script
5. Enable approval workflow in "dry-run" mode (log but don't block) for 1 week
6. Switch to enforcement mode after validation

**Rollback Strategy:**
- Database migrations are additive only (no deletions)
- New API endpoints are additive (no breaking changes)
- If critical issue: disable AI roles via feature flag, fall back to manual workflow
- Database state preserved for re-enabling later

## Open Questions

1. **Approval Queue UI**: Should we build a dedicated approval dashboard or integrate into existing agent view?
2. **Email/SMS Integration**: Do we need actual email/SMS sending infrastructure, or drafts only for MVP?
3. **Lead Sources**: Which web form platforms to integrate (existing form builder, custom, third-party)?
4. **Multi-Agent Support**: Should we design for multiple human agents per AI team from the start?
