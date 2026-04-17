# Web3 UI/UX Researcher Agent Implementation Plan

## Overview
Create a specialized Web3 UI/UX Researcher agent that reports to a Research Lead, focusing on DeFi protocols and Loyalty systems. The agent will extend the existing ResearchAgent capabilities with Web3-specific research methodologies.

## User Requirements Confirmed:
1. ✓ Create Research Lead agent role
2. ✓ Focus on DeFi and Loyalty systems
3. ✓ Save reports to `research_output/web3/`
4. ✓ Use Venice AI API as default
5. ✓ Include examples in demo.sh

## Implementation Steps

### Phase 1: Core Backend Changes

#### 1.1 Add Agent Roles to `src/agent/core.rs`
**Line 28-43**: Extend the `AgentRole` enum:
```rust
pub enum AgentRole {
    // ... existing roles ...
    ResearchLead,    // Leads research teams and manages research projects
    Web3Researcher,  // Specialized in Web3 UI/UX research
    // ... rest of roles ...
}
```

**Update `name()` method** to return:
- "Research Lead" for ResearchLead
- "Web3 UI/UX Researcher" for Web3Researcher

**Update `can_create()`** to establish hierarchy:
- ResearchLead can create Web3Researcher
- Web3Researcher can execute research tasks

#### 1.2 Create Role Module Structure
**New file: `src/agent/roles/research_lead.rs`**
Purpose: Manages research projects and coordinates Web3 researchers
Key capabilities:
- Create research projects
- Assign researchers
- Review research outputs
- Track research budgets
- Aggregate findings

**New file: `src/agent/roles/web3_researcher.rs`**
Purpose: Specialized Web3 UI/UX research agent
Key capabilities:
- DeFi protocol analysis
- Loyalty system UX review
- Generate structured reports
- Create UX scorecards
- Provide code pattern recommendations
- Visual interface annotations

#### 1.3 Extend `src/agent/roles/mod.rs`
Add:
```rust
pub mod research_lead;
pub mod web3_researcher;

pub use research_lead::*;
pub use web3_researcher::*;
```

Update `generate_role_prompt()` to add:
- ResearchLead prompt
- Web3Researcher prompt with Web3-specific instructions

### Phase 2: Web3 Research Capabilities

#### 2.1 Research Request Types
**New file: Define research request structures**

```rust
pub struct Web3ResearchRequest {
    pub research_type: ResearchType,
    pub target_urls: Vec<String>,
    pub focus_areas: Vec<String>,
    pub output_format: OutputFormat,
}

pub enum ResearchType {
    DeFiProtocolAnalysis,
    LoyaltySystemReview,
    WalletUXStudy,
    ComparativeAnalysis,
}

pub enum OutputFormat {
    MarkdownReport,
    UXScorecard,
    CodeRecommendations,
    VisualAnnotations,
    All,
}
```

#### 2.2 DeFi Protocol Analysis
Focus areas:
- Liquidity pool interfaces
- Lending/borrowing flows
- Yield farming UX
- Transaction confirmation screens
- Gas fee transparency
- Slippage warnings
- Security indicators

#### 2.3 Loyalty System Analysis
Focus areas:
- Token reward visualization
- Points systems
- Tier progression displays
- Redemption flows
- Wallet connection UX
- Reward claim processes

#### 2.4 Output Generation
Each research produces:
1. **Structured Report** (Markdown)
   - Executive summary
   - Interface inventory
   - UX strengths and weaknesses
   - Recommendations with priorities

2. **UX Scorecard** (JSON)
   - Accessibility score
   - Clarity score
   - Trust indicators score
   - Security visibility score
   - Overall rating

3. **Code Recommendations** (Markdown)
   - Pattern examples
   - Component suggestions
   - Implementation guides

4. **Visual Annotations** (Data structure for UI display)
   - Screenshot references
   - Improvement markers
   - Color/contrast notes

### Phase 3: Research Data Sources

#### 3.1 Web Scraping Capability
- Automated DApp interface capture
- Screenshot functionality
- DOM structure analysis
- Interactive element detection

#### 3.2 API Integrations
- **Dune Analytics**: Query on-chain metrics
- **DeFiLlama**: TVL and protocol data
- **CoinGecko**: Token price feeds
- **The Graph**: Subgraph queries for protocol usage

#### 3.3 Manual Input
- URL submission form
- Screenshot upload
- Research parameters configuration
- Custom focus areas

### Phase 4: UI Integration

#### 4.1 Add Research Button to Sidebar
**File: `spree/assets/index.html`**
Add new sidebar section:
```html
<div class="sidebar-section">
    <div class="section-header">
        <h3>Web3 Research</h3>
    </div>
    <div class="quick-actions">
        <button class="action-btn" data-action="web3-research">
            <svg>...</svg>
            New Research
        </button>
        <button class="action-btn" data-action="view-research-reports">
            <svg>...</svg>
            View Reports
        </button>
    </div>
</div>
```

#### 4.2 Research Submission Modal
**File: `spree/assets/index.html`**
Create modal for research configuration:
- Research type selector (DeFi, Loyalty, Comparative)
- URL input field
- Focus areas checkboxes
- Output format selector
- Budget/depth selector

#### 4.3 Update JavaScript Handlers
**File: `spree/assets/app.js`**
Add to `handleQuickAction()`:
```javascript
case 'web3-research':
    this.showWeb3ResearchModal();
    break;
case 'view-research-reports':
    this.loadAndShowResearchReports();
    break;
```

Add new methods:
- `showWeb3ResearchModal()`
- `submitWeb3ResearchRequest()`
- `loadAndShowResearchReports()`

### Phase 5: Demo Script Updates

#### 5.1 Update `demo.sh`
Add new section after existing research demo:
```bash
echo ""
echo "8. Creating Web3 UI/UX Research Team..."
# Create Research Lead under Chief AI Officer
# Create Web3 Researcher under Research Lead
# Submit DeFi protocol research task
# Submit Loyalty system research task
```

#### 5.2 Add Example Research Tasks
Include sample research requests:
1. Analyze Uniswap V3 interface
2. Review Aave lending UX
3. Evaluate a loyalty program on Polygon
4. Compare bridge interfaces

### Phase 6: Research Output Storage

#### 6.1 Directory Structure
```
research_output/
├── web3/
│   ├── defi/
│   │   ├── uniswap_v3_analysis_2024.md
│   │   ├── aave_ux_review_2024.md
│   │   └── scorecards/
│   ├── loyalty/
│   │   ├── polygon_loyalty_program_review.md
│   │   └── scorecards/
│   └── comparative/
│       ├── dex_comparison_q4_2024.md
│       └── bridge_ux_comparison.md
```

#### 6.2 File Naming Convention
`{protocol_name}_{research_type}_{YYYYMMDD}.{format}`

## Implementation Order

1. **Phase 1.1** - Add agent roles to core.rs
2. **Phase 1.2** - Create research_lead.rs module
3. **Phase 1.3** - Create web3_researcher.rs module
4. **Phase 1.4** - Update mod.rs with new modules
5. **Phase 2** - Implement research capabilities
6. **Phase 3** - Add data source integrations
7. **Phase 4** - UI integration
8. **Phase 5** - Demo script updates
9. **Phase 6** - Output storage setup

## Files to Create/Modify

### New Files:
- `src/agent/roles/research_lead.rs`
- `src/agent/roles/web3_researcher.rs`
- `research_output/web3/` (directory)

### Modified Files:
- `src/agent/core.rs` - Add new roles
- `src/agent/roles/mod.rs` - Add modules and prompts
- `src/agent/roles/research.rs` - Extend capabilities
- `spree/assets/index.html` - Add UI components
- `spree/assets/app.js` - Add JavaScript handlers
- `demo.sh` - Add research examples

## Success Criteria

1. ✓ Can create Research Lead agent
2. ✓ Research Lead can create Web3Researcher subordinates
3. ✓ Web3Researcher can analyze DeFi protocols
4. ✓ Web3Researcher can review Loyalty systems
5. ✓ Research outputs saved to `research_output/web3/`
6. ✓ Reports generated in all formats (markdown, scorecards, code, annotations)
7. ✓ UI buttons functional in web interface
8. ✓ Demo script includes working examples
9. ✓ Venice AI API used for all LLM calls

## Notes

- Research Lead should inherit from Manager or Director level in hierarchy
- Web3Researcher should report to Research Lead
- Default to Venice AI API for all LLM interactions
- Support combination of web scraping, APIs, and manual input
- Focus on DeFi and Loyalty as primary research domains
- Reports should be comprehensive yet actionable
- Include code pattern recommendations where applicable
