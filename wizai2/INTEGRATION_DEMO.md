# Spree + autoresearch + mppx Integration Demo

This document describes the integration of three powerful concepts into the Spree Agent Framework.

## Overview

**Spree** provides the organizational structure for multi-agent coordination.
**autoresearch** provides the methodology for autonomous ML experimentation.
**mppx** provides the payment infrastructure for resource monetization.

Together, they create a self-sustaining research organization that:
1. Automatically designs and runs ML experiments
2. Tracks resource usage and costs
3. Manages budgets hierarchically
4. Generates invoices and payments

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Spree Agent Framework                     │
├─────────────────────────────────────────────────────────────┤
│  Organization                                                │
│  ├── Chief AI Officer (budget: $100/month)                  │
│  │   └── Research Lead (budget: $50/month)                  │
│  │       ├── Researcher 1                                   │
│  │       ├── Researcher 2                                   │
│  │       └── Researcher 3                                   │
│  └── CFO (monitors costs)                                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Research System                           │
│  (autoresearch-style experiments)                              │
│                                                                │
│  1. Design Experiment (LLM)                                   │
│     └── Cost: $0.05                                           │
│  2. Execute Experiment (5-min budget)                       │
│     └── Cost: $0.50 (compute)                                 │
│  3. Analyze Results (LLM)                                     │
│     └── Cost: $0.05                                           │
│  4. Iterate or Report                                         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Payment System                            │
│  (mppx-style micropayments)                                   │
│                                                                │
│  Resources:                                                   │
│  • LLM calls: $0.10/1K tokens input, $0.20/1K output         │
│  • Compute: $0.0001/core-second (~$0.36/core-hour)          │
│  • Storage: $0.02/GB-month                                    │
│  • Experiments: Fixed budget (e.g., $0.50 per run)           │
│                                                                │
│  Features:                                                    │
│  • Real-time cost tracking                                    │
│  • Per-agent budget management                                │
│  • Automatic invoicing                                        │
│  • Cost breakdown by resource type                            │
└─────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Research Agent (`src/agent/roles/research.rs`)

Specialized agent for ML experimentation following autoresearch principles:

```rust
impl ResearchAgent {
    // Designs experiments based on hypotheses
    pub async fn design_experiment(&self, hypothesis: &str) -> Result<ExperimentConfig>;
    
    // Runs experiments with fixed budget (5-minute default)
    pub async fn run_experiment(&self, config: ExperimentConfig) -> Result<ExperimentResult>;
    
    // Analyzes results and recommends next steps
    pub async fn analyze_results(&self, results: &[ExperimentResult]) -> Result<String>;
}
```

**Features:**
- Automatic experiment design via LLM
- Fixed time budget for fair comparison
- Tracks validation bits-per-byte (val_bpb)
- Cost-aware execution

### 2. Payment System (`src/payments/`)

Inspired by mppx's Machine Payments Protocol:

```rust
impl PaymentSystem {
    // Per-token LLM pricing
    pub async fn charge_llm(&self, agent_id, model, tokens_in, tokens_out);
    
    // Per-second compute pricing
    pub async fn charge_compute(&self, agent_id, duration_secs, cpu_cores);
    
    // Per-experiment fixed pricing
    pub async fn charge_experiment(&self, agent_id, experiment_id, budget_usd);
    
    // Agent-to-agent transfers
    pub async fn transfer(&self, from, to, amount_usd, description);
}
```

**Features:**
- Per-token pricing (like mppx)
- Multiple resource types (LLM, compute, storage, experiments)
- Agent-specific balance tracking
- Automatic invoicing
- Budget alerts

### 3. Integration Layer

The three systems work together through:

1. **Task Delegation**: Spree's hierarchy allows budget allocation down the chain
2. **Cost Attribution**: Every resource usage is tracked per-agent
3. **Budget Enforcement**: Agents can't spend beyond their allocated budget
4. **Invoice Generation**: Automatic billing for resource usage

## Pricing Model

### LLM Pricing (per 1K tokens)

| Model | Input | Output |
|-------|-------|--------|
| venice-uncensored | $0.10 | $0.20 |
| venice-llama-3.3-70b | $0.50 | $1.00 |
| venice-qwen-2.5-coder-32b | $0.30 | $0.60 |
| gpt-4 | $30.00 | $60.00 |
| gpt-3.5-turbo | $0.50 | $1.50 |

### Compute Pricing

- **Rate**: $0.0001 per core-second
- **Equivalent**: $0.36 per core-hour
- **Example**: 5-minute experiment on 1 core = $0.03

### Tool Pricing

| Tool | Base Price |
|------|-----------|
| read_file | $0.0005 |
| write_file | $0.0005 |
| bash | $0.001 |
| python | $0.0015 |
| subordinate | $0.01 |

## Running the Demo

### Option 1: Quick Demo Script

```bash
# Run the shell demo (doesn't require compilation)
cd wizai2
./demo.sh
```

This will:
1. Start the Spree server
2. Create an organization hierarchy
3. Run a research campaign
4. Show the organization tree
5. Display results

### Option 2: Full Integration Demo

```bash
# Set API key
export VENICE_API_KEY="your_key_here"

# Run the Rust demo
cargo run --example research_demo
```

This comprehensive demo shows:
- Agent creation and delegation
- Payment tracking for each resource
- Cost breakdown by agent
- Automatic invoice generation

### Option 3: Manual Testing

```bash
# 1. Start the server
cargo run

# 2. In another terminal, create agents

# Create CEO
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{"name":"Chief AI","role":"ChiefAI"}'

# Create Research Lead under CEO
# (Replace <ceo-id> with actual ID)
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name":"Research Lead",
    "role":"Lead",
    "superior_id":"<ceo-id>"
  }'

# Create Researchers under Lead
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name":"Researcher 1",
    "role":"Specialist",
    "superior_id":"<lead-id>"
  }'

# Submit a research task
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id":"<ceo-id>",
    "task":"Design an ML experiment to optimize nano-LLM training"
  }'

# Check organization
curl http://localhost:3000/api/organization
```

## Example Workflow

### 1. Budget Allocation

```
User allocates $100 to Chief AI Officer
├── Chief AI allocates $50 to Research Lead
│   └── Research Lead allocates $10 per researcher
└── CFO monitors spending
```

### 2. Experiment Execution

```
Researcher designs experiment
├── LLM call: 2000 tokens in, 1500 tokens out
│   └── Cost: $0.05
├── Experiment runs for 5 minutes
│   └── Cost: $0.03 (compute)
└── Analysis LLM call
    └── Cost: $0.03

Total: $0.11 per experiment
```

### 3. Cost Tracking

Each agent has a ledger showing:
- Total spent
- Remaining budget
- Cost breakdown by resource type
- Payment history

### 4. Invoicing

Monthly invoices generated automatically:
```
Invoice ID: INV-2026-03-001
Agent: Chief AI Officer
Period: 2026-03-01 to 2026-03-31

Line Items:
  - LLM calls (venice-uncensored): $45.20
  - Compute (300 core-hours): $108.00
  - Experiments (50 runs): $25.00

Total: $178.20
```

## Configuration

### Environment Variables

```bash
# Required
VENICE_API_KEY=your_venice_api_key

# Optional
VENICE_BASE_URL=https://api.venice.ai
PORT=3000
HOST=127.0.0.1

# Pricing (optional overrides)
LLM_INPUT_PRICE_PER_1K=0.0001
LLM_OUTPUT_PRICE_PER_1K=0.0002
COMPUTE_PRICE_PER_SEC=0.0001
DEFAULT_EXPERIMENT_BUDGET=0.50
```

### Custom Pricing

Update `src/payments/pricing.rs`:

```rust
let mut pricing = PricingModel::default();
pricing.add_model_pricing("custom-model", 0.05, 0.10);
```

## Production Considerations

### Phase 2: Real Payment Processing

Replace the mock payment processor with real providers:

```rust
// Stripe integration
pub async fn process_stripe_payment(
    amount_usd: f64,
    payment_method: &str,
) -> Result<String> {
    // Use Stripe SDK
}

// Crypto payments (like mppx)
pub async fn process_crypto_payment(
    amount_usd: f64,
    wallet_address: &str,
) -> Result<String> {
    // Use ethers-rs or similar
}
```

### Phase 3: Advanced Features

1. **Payment Channels**: Like mppx's streaming payments
2. **Prepaid Balances**: Agents load funds before execution
3. **Cost Optimization**: Automatically choose cheaper models
4. **Budget Alerts**: Notify when approaching limits
5. **Cost Sharing**: Split costs between multiple agents

## Integration Points

### Connecting to External Systems

**autoresearch Repository:**
```bash
# Spree Research Agent can clone and manage autoresearch repos
curl -X POST http://localhost:3000/api/agents \
  -d '{
    "name":"Autoresearch Manager",
    "role":"Manager",
    "task":"Clone KBryan/autoresearch and run experiments"
  }'
```

**mppx Integration:**
```typescript
// In a TypeScript client
import { Mppx } from 'mppx/client';

// Spree agents can use mppx for external API payments
const mppx = Mppx.create({
  methods: [tempo({ account: privateKey })],
});

// Now fetch calls automatically handle 402 payments
const response = await fetch('https://api.venice.ai/v1/chat/completions');
```

## Success Metrics

A successful integration demonstrates:

✅ **Agent Hierarchy**: Clear chain of command
✅ **Autonomous Research**: Self-directed experimentation
✅ **Cost Tracking**: Every resource usage accounted for
✅ **Budget Management**: Hierarchical allocation and enforcement
✅ **Payment Processing**: Automated billing and invoicing
✅ **Transparency**: Full visibility into costs and results

## Next Steps

1. **Test the demo**: Run `./demo.sh` to see the system in action
2. **Extend functionality**: Add more research tools and payment methods
3. **Scale up**: Deploy multiple research teams in parallel
4. **Integrate real systems**: Connect to actual ML training infrastructure
5. **Monitor costs**: Track actual spending vs. budgets

## References

- **autoresearch**: https://github.com/karpathy/autoresearch
- **mppx**: https://mpp.dev/sdk/typescript
- **Spree**: See README.md for core documentation

## License

MIT - See LICENSE file
