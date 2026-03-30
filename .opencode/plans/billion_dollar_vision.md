# Spree + OpenCode: Billion-Dollar Research Organization

## Executive Summary

**Vision**: Build the world's first autonomous Web3 research organization that coordinates AI agents to deliver billion-dollar insights through seamless OpenCode integration, human-in-the-loop quality control, and a sustainable payment-driven business model.

**Differentiation**: While others build tools, we build organizations. Spree's hierarchical coordination + OpenCode's execution capabilities = unmatched research velocity at enterprise scale.

---

## 1. OpenCode Integration Architecture ("The Seamless Bridge")

### 1.1 Bidirectional Communication Layer

**Spree → OpenCode Flow**:
```
Spree Agent (Research Lead)
    ↓ Identifies need for implementation
    ↓ Generates structured research findings
    ↓ Formats as OpenCode-compatible spec
OpenCode Skill Trigger
    ↓ `/skill:openspec-new-change <research-implementation>`
    ↓ Creates change scaffold with artifacts
OpenCode Execution
    ↓ Implements the research findings
    ↓ Returns implementation status
Spree Agent (CTO)
    ↓ Validates implementation
    ↓ Tracks costs across both systems
```

**OpenCode → Spree Flow**:
```
OpenCode detects need for research
    ↓ Queries Spree Research API
Spree Researcher Agent
    ↓ Conducts analysis
    ↓ Returns findings
OpenCode receives insights
    ↓ Updates implementation plan
    ↓ Continues execution
```

### 1.2 Technical Implementation

**New Module: `src/integrations/opencode.rs`**

```rust
pub struct OpenCodeBridge {
    workspace_path: PathBuf,
    venice_client: Arc<VeniceClient>,
    payment_system: Arc<PaymentSystem>,
}

impl OpenCodeBridge {
    /// Spree agent triggers OpenCode workflow
    pub async fn trigger_change_proposal(&self, research_findings: ResearchReport) -> Result<ChangeId> {
        // 1. Format research as OpenSpec proposal
        let proposal = self.format_research_as_proposal(research_findings);
        
        // 2. Execute OpenCode command via subprocess or API
        let change_id = self.execute_opencode_command(&format!(
            "/opsx-new {}", 
            proposal.sanitize_name()
        )).await?;
        
        // 3. Track cost in Spree payment system
        self.payment_system.charge_tool(
            agent_id,
            "opencode_change_creation",
            0.001
        ).await?;
        
        Ok(change_id)
    }
    
    /// OpenCode calls back to Spree for research
    pub async fn request_research(&self, query: ResearchQuery) -> Result<ResearchReport> {
        // Route to appropriate Spree researcher
        let researcher = self.find_researcher(query.domain);
        
        // Execute research through Spree hierarchy
        let report = researcher.conduct_research(query).await?;
        
        // Charge for research time
        self.payment_system.charge_research(
            researcher.id,
            query.estimated_hours
        ).await?;
        
        Ok(report)
    }
}
```

### 1.3 Shared Context Layer

**Unified Workspace**:
```
workspace/
├── spree/
│   ├── agents/           # Agent configurations
│   ├── research/         # Research reports
│   └── payments/         # Cost tracking
├── opencode/
│   ├── changes/          # OpenSpec changes
│   ├── specs/            # Technical specs
│   └── skills/           # Skill definitions
└── shared/
    ├── context.md        # Shared project context
    ├── research_db/      # Research data (SQLite)
    └── cost_ledger/      # Unified payments (SQLite)
```

**Cost Attribution**:
- Spree tracks agent coordination costs
- OpenCode tracks implementation costs
- Unified ledger in `shared/cost_ledger/`
- Per-project billing across both systems

---

## 2. Human-in-the-Loop Workflow ("The Quality Flywheel")

### 2.1 Three-Tier Quality Control

```
┌─────────────────────────────────────────────────────────────┐
│                    TIER 1: Autonomous                        │
│  AI Agents conduct initial research and analysis             │
│  ↓ Confidence Score < 0.7 triggers escalation              │
├─────────────────────────────────────────────────────────────┤
│                    TIER 2: Human Review                      │
│  Domain expert validates findings and recommendations        │
│  ↓ High-value or novel insights trigger executive review     │
├─────────────────────────────────────────────────────────────┤
│                    TIER 3: Executive Sign-off                │
│  Leadership validates strategic recommendations              │
│  ↓ Final report delivered to client                         │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Implementation Details

**Confidence Scoring**:
```rust
pub struct ResearchConfidence {
    pub data_quality: f32,      // 0.0 - 1.0 based on source reliability
    pub sample_size: f32,       // Based on number of DApps analyzed
    pub consistency: f32,       // Cross-protocol pattern consistency
    pub novelty: f32,           // How new/unique are the findings
    pub overall: f32,           // Weighted average
}

impl ResearchConfidence {
    pub fn requires_human_review(&self) -> bool {
        self.overall < 0.7 || self.novelty > 0.8
    }
    
    pub fn requires_executive_review(&self) -> bool {
        self.overall < 0.9 && self.sample_size > 100
    }
}
```

**Human Review Interface**:
- Web dashboard for researchers to review AI findings
- Annotation tools for adding human insights
- Approval workflows before delivery
- Feedback loop to train better AI models

**Training Loop**:
```
AI generates research → Human reviews → Human adds insights
                                           ↓
                    AI improves based on feedback ← Feedback captured
```

### 2.3 Cost Structure with Humans

**Pricing Model**:
```
Autonomous Research: $50/hour (AI only)
Human-Reviewed Research: $150/hour (AI + domain expert)
Executive Research: $500/hour (AI + expert + executive sign-off)

Volume Discounts:
- 10+ reports/month: 10% off
- 50+ reports/month: 25% off
- Enterprise (unlimited): Custom pricing
```

**Revenue Distribution**:
```
Client pays: $10,000 for comprehensive DeFi UX audit
    ↓
70% - Research Organization (us)
    ├── 40% - AI infrastructure costs
    ├── 30% - Human researchers
    └── 30% - Platform profit

30% - OpenCode (execution partner)
    └── Payment for implementation work
```

---

## 3. Business Model: Research-as-a-Service ("RaaS")

### 3.1 Revenue Streams

**Primary: Subscription Tiers**

| Tier | Price | Includes |
|------|-------|----------|
| **Starter** | $499/month | 5 research reports, basic DeFi coverage |
| **Professional** | $2,499/month | 25 reports, all Web3 domains, quarterly strategy calls |
| **Enterprise** | $10,000/month | Unlimited reports, custom research, dedicated research team |
| **Bespoke** | Custom | White-glove service, on-call researchers, executive briefings |

**Secondary: À La Carte**
- Single report: $300-800 depending on scope
- Urgent delivery (24hr): +50% surcharge
- Custom analysis: $200/hour

**Tertiary: Data Licensing**
- API access to research database: $0.01/query
- Raw dataset access: $5,000/month
- White-label reports for resale: Revenue share

### 3.2 Payment System Integration

**Prepaid Credits Model**:
```
1. Client purchases credits (e.g., $10,000)
2. Credits allocated to Research Lead
3. Research Lead distributes to Specialists
4. Each resource consumption deducts credits:
   - AI researcher time: $50/hour
   - Human researcher time: $150/hour  
   - Compute for analysis: $0.36/core-hour
   - LLM API calls: $0.10-0.50/1K tokens
5. Real-time dashboard shows credit balance
6. Auto-replenishment when < $500 remaining
```

**Automatic Invoicing**:
```
Monthly Invoice: ACME Protocol
Period: 2026-03-01 to 2026-03-31

Research Services:
  - Autonomous research (42 hours): $2,100
  - Human-reviewed analysis (8 hours): $1,200
  - Executive briefing (1 hour): $500

Infrastructure:
  - Compute resources: $245
  - LLM API usage: $127

Total: $4,172
Credits remaining: $5,828

Payment due: 2026-04-15
```

### 3.3 Unit Economics (Target)

**Cost Structure per Report**:
```
Revenue per report: $400 (blended average)
Cost to deliver:
  - AI research (4 hrs @ $10/hr*): $40
  - Human review (1 hr @ $80/hr**): $80
  - Compute/infra: $15
  - OpenCode execution: $25
  - Platform overhead: $40
  ─────────────────────────────────
  Total cost: $200
  
Gross margin: 50%
* Fully loaded AI cost including LLM, compute, storage
** Human researcher rate after platform cut
```

**Scale Targets**:
- Year 1: 100 reports/month = $480K revenue, $240K margin
- Year 2: 1,000 reports/month = $4.8M revenue, $2.4M margin
- Year 3: 10,000 reports/month = $48M revenue, $24M margin
- Year 5: 100,000 reports/month = $480M revenue, $240M margin

**Billion-Dollar Path**:
- Achieve 100K+ reports/month + enterprise contracts
- Add adjacent services (implementation, training)
- Expand to AI-powered hedge fund (proprietary trading on research)
- IPO or acquisition by major consulting firm

---

## 4. Scaling Strategy ("The Exponential Curve")

### 4.1 Phase 1: Foundation (Months 1-6) - Human-Heavy

**Focus**: Prove model with manual oversight

**Team**: 
- 1 CEO (you)
- 1 CTO (technical co-founder)
- 2 Research Leads (DeFi + Loyalty)
- 4 Human Researchers
- 1 Client Success Manager

**Target**: 20 reports/month
**Revenue**: ~$8K/month
**Key Metrics**:
- Report quality score > 4.5/5
- Human review time per report < 2 hours
- Client retention > 80%

**Technology**:
- Basic Spree hierarchy
- Manual OpenCode integration (CLI commands)
- Simple Stripe billing
- Human approval dashboard

### 4.2 Phase 2: Automation (Months 7-18) - AI-Heavy

**Focus**: Reduce human hours per report through better AI

**Team**:
- Keep existing leadership
- 1 Research Lead per vertical (add NFT, Gaming, etc.)
- 10 Human Researchers
- 2 Engineers (OpenCode integration)
- 2 Sales

**Target**: 200 reports/month
**Revenue**: ~$80K/month
**Key Metrics**:
- Human review time < 30 minutes per report
- AI confidence > 0.8 on 70% of reports
- Cost per report < $100

**Technology**:
- Seamless OpenCode API integration
- Automated quality scoring
- Self-service client portal
- Credit-based billing system

### 4.3 Phase 3: Scale (Months 19-36) - Platform

**Focus**: Remove human bottleneck through better AI + self-service

**Team**:
- Full C-suite
- 5 Research verticals (each with Lead + team)
- 20 Human Researchers (review only)
- 5 Engineers
- 5 Sales
- 3 Marketing

**Target**: 2,000 reports/month
**Revenue**: ~$800K/month
**Key Metrics**:
- Human review required on <20% of reports
- 24-hour delivery standard
- Self-service > 50% of orders

**Technology**:
- Fully autonomous report generation
- AI-only tier for simple queries
- Marketplace for specialized researchers
- Mobile app for clients

### 4.4 Phase 4: Billion-Dollar (Year 4+) - Ecosystem

**Focus**: Platform ecosystem + adjacent revenue

**Team**: 100+ employees
**Target**: 20,000+ reports/month + enterprise services
**Revenue**: $10M+/month

**Expansion**:
- Web3 research → AI research (general ML)
- Research → Implementation (via OpenCode)
- Reports → Real-time API feeds
- Services → Training + certification
- Platform → Research marketplace

---

## 5. IP Ownership Strategy ("Open Core Model")

### 5.1 Phase 1: Proprietary (Now - Year 2)

**Ownership**: Spree Inc. owns all IP

**Assets**:
- Spree source code (private repo)
- Research database (proprietary)
- Analysis methodologies (trade secrets)
- Client relationships (CRM data)

**Protection**:
- Patents on coordination algorithms
- Trade secret protection for research methods
- NDAs with all employees and contractors

### 5.2 Phase 2: Open Core (Year 3-4)

**Open Source**:
- Spree framework (core coordination)
- Basic agent roles
- Payment system primitives

**Proprietary**:
- Research methodologies
- Training data (anonymized)
- Client-specific insights
- Advanced AI models

**Revenue Model**:
- Free: Open source tools
- Paid: Hosted service, premium features, research data

### 5.3 Phase 3: Foundation (Year 5+)

**Structure**:
- Spree Foundation (non-profit): Owns core protocol
- Spree Inc. (for-profit): Builds on foundation

**Benefits**:
- Protocol owned by community
- Enterprise trust (not locked to vendor)
- Ecosystem growth
- Still monetize via services

**Similar to**: Linux Foundation + Red Hat model

---

## 6. Risk Mitigation

### 6.1 Technical Risks

| Risk | Mitigation |
|------|------------|
| AI hallucinations | Human review + confidence scoring + source attribution |
| OpenCode breaking changes | Version pinning + integration tests + dedicated engineer |
| Scaling bottlenecks | Horizontal scaling + async processing + caching |
| Data loss | Daily backups + multi-region + disaster recovery |

### 6.2 Business Risks

| Risk | Mitigation |
|------|------------|
| Competition | First-mover advantage + network effects + data moat |
| Client churn | Subscription model + usage-based + exceptional service |
| Regulatory (crypto) | Legal review + compliance team + insurance |
| Economic downturn | Multiple revenue streams + cost flexibility |

### 6.3 Operational Risks

| Risk | Mitigation |
|------|------------|
| Key person dependency | Document everything + cross-training + succession plan |
| Quality degradation | Metrics dashboard + human spot-checks + client feedback |
| Security breach | Security audits + bug bounties + encryption + access controls |

---

## 7. Immediate Next Steps (Week 1-4)

### Week 1: Foundation
- [ ] Finalize OpenCode integration architecture
- [ ] Set up development environment for both Spree + OpenCode
- [ ] Create shared workspace structure
- [ ] Define API contracts between systems

### Week 2: Integration
- [ ] Implement `OpenCodeBridge` module
- [ ] Build first end-to-end flow (research → OpenCode → implementation)
- [ ] Create cost attribution system
- [ ] Test with simple DeFi protocol

### Week 3: Human Interface
- [ ] Build human review dashboard
- [ ] Implement confidence scoring
- [ ] Create approval workflow
- [ ] Train first human researchers

### Week 4: MVP
- [ ] Launch with 2-3 pilot clients
- [ ] Price at $300/report (below target to prove value)
- [ ] Collect feedback aggressively
- [ ] Iterate based on learnings

---

## 8. Competitive Positioning

### "We're Not Building Another Tool. We're Building the First AI Research Organization."

**vs. Consulting Firms (McKinsey, Deloitte)**:
- ❌ They charge $500-1000/hour
- ✅ We charge $50-150/hour (AI + human)
- ✅ 10x faster delivery (24hr vs 2 weeks)
- ✅ Always-on availability

**vs. AI Tools (ChatGPT, Claude)**:
- ❌ They provide raw answers
- ✅ We provide structured research with methodology
- ✅ We validate with human experts
- ✅ We implement findings via OpenCode

**vs. Research Platforms (Messari, Delphi)**:
- ❌ They write about what's already known
- ✅ We discover what's not yet known
- ✅ We customize to client needs
- ✅ We implement, not just report

**vs. Dev Agencies**:
- ❌ They build without understanding market
- ✅ We research first, then build
- ✅ Data-driven implementation
- ✅ Continuous optimization

---

## 9. Key Metrics Dashboard (Track Weekly)

### Product Metrics
- Report quality score (client rating)
- Time to delivery
- AI confidence distribution
- Human review time

### Business Metrics
- Monthly recurring revenue (MRR)
- Customer acquisition cost (CAC)
- Lifetime value (LTV)
- Churn rate
- Revenue per report
- Gross margin

### Operational Metrics
- Reports delivered per week
- Active researchers (AI + human)
- System uptime
- Cost per report
- Credits consumed per client

---

## 10. Conclusion

**The Vision**: Build the Bloomberg of Web3 + the McKinsey of AI, powered by the first truly autonomous research organization.

**The Moat**: No one else has Spree's coordination capabilities + OpenCode's execution power + your domain expertise.

**The Path**: Start human-heavy, automate aggressively, scale to thousands of reports, expand to adjacent markets.

**The Destination**: A billion-dollar company that fundamentally changes how research is conducted and implemented.

---

**Ready to execute?** Let's build this.

