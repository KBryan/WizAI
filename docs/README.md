# WIZAI Documentation

Welcome to the WIZAI documentation hub. This directory contains comprehensive guides for both the legacy WIZAI system and the new **WizAI2** system.

## Quick Navigation

### 📚 Main Documentation
- **[INDEX.md](INDEX.md)** - Full documentation index and navigation guide
- **[TUTORIAL.md](../TUTORIAL.md)** - Step-by-step OpenSpec Execution Agent tutorial
- **[QUICK_REFERENCE.md](../QUICK_REFERENCE.md)** - Command cheatsheet and quick reference

### 🆕 WizAI2

**WizAI2** (located in `../wizai2/`) is a Rust-native, WASM-based multi-agent organizational system.

#### Key Features
- **Hierarchical Agent System**: Superior/subordinate relationships with delegation
- **SQLite Memory**: Persistent storage with semantic search
- **Venice AI Integration**: Streaming LLM responses
- **Tool System**: Read/write files, execute code, delegate tasks
- **WebSocket**: Real-time updates and streaming
- **WASM UI**: Fast, reactive web interface
- **REST API**: Easy integration with external systems

#### New Capabilities in WizAI2

**Research Agent Integration**
- Autonomous ML experimentation with fixed 5-minute budgets
- Automatic experiment design via LLM
- Cost-aware execution tracking validation metrics (val_bpb)
- Result analysis and recommendations

**Payment System (mppx-inspired)**
- Per-token LLM pricing: $0.10/1K tokens input, $0.20/1K output
- Per-second compute pricing: $0.0001/core-second
- Agent-to-agent transfers for delegation
- Automatic invoicing with detailed line items
- Hierarchical budget allocation and enforcement

**Organizational Hierarchy**
- C-Level: CEO, CTO, CFO, Chief AI Officer, Chief Product Officer
- Management: VP, Director, Manager, Lead
- Individual Contributors: Specialist, Intern
- Clear chain of command for research coordination

#### Quick Start - WizAI2

```bash
cd ../wizai2

# Copy environment file
cp .env.example .env

# Edit .env and add your Venice API key
# VENICE_API_KEY=your_key_here

# Build and run
cargo run

# Access the application
# Web UI: http://localhost:3000
# REST API: http://localhost:3000/api/
# WebSocket: ws://localhost:3000/ws
```

#### WizAI2 Documentation
- **[../wizai2/README.md](../wizai2/README.md)** - Full framework documentation
- **[../wizai2/DEMO_SUMMARY.md](../wizai2/DEMO_SUMMARY.md)** - Integration demo summary
- **[../wizai2/QUICKSTART.md](../wizai2/QUICKSTART.md)** - Quick start guide
- **[../wizai2/docs/SKILLS_SYSTEM.md](../wizai2/docs/SKILLS_SYSTEM.md)** - Skills system documentation
- **[../wizai2/docs/WEB_SCRAPING_SETUP.md](../wizai2/docs/WEB_SCRAPING_SETUP.md)** - Web scraping setup

---

## Legacy WIZAI System

### Core Documentation

#### Architecture
- **[LOCAL_TOOL_CALLING.md](LOCAL_TOOL_CALLING.md)** - How local tool calling works
  - Why we use local tool execution
  - Parsing strategies for tool intent
  - Integration with LLM responses
  - Benefits and comparison with native tool calling

#### Code Generation
- **[CLI_GENERATOR.md](CLI_GENERATOR.md)** - Rust-native CLI generator
  - Generate CLIs from any software
  - Tree-sitter AST analysis
  - Askama template system
  - Usage examples and compilation

#### API & Tools
- **[API_REFERENCE.md](API_REFERENCE.md)** - Complete API documentation
  - All REST endpoints
  - WebSocket interface
  - Tool registry reference
  - Error codes and responses

#### Usage Examples
- **[EXAMPLES.md](EXAMPLES.md)** - Practical examples
  - Generating CLIs from code
  - OpenSpec workflow step-by-step
  - Local tool calling flow
  - Multi-tool workflows
  - Error handling

## Documentation Map

```
WIZAI/
├── README.md                    # Project overview and quick start
├── TUTORIAL.md                  # OpenSpec tutorial
├── QUICK_REFERENCE.md          # Command reference
├── docs/
│   ├── README.md               # This file - documentation overview
│   ├── INDEX.md                # Detailed navigation index
│   ├── CLI_GENERATOR.md        # CLI generation
│   ├── LOCAL_TOOL_CALLING.md   # Tool execution architecture
│   ├── API_REFERENCE.md        # API documentation
│   └── EXAMPLES.md             # Usage examples
├── wizai2/                     # WizAI2 System
│   ├── README.md               # Framework documentation
│   ├── DEMO_SUMMARY.md         # Integration summary
│   ├── QUICKSTART.md           # Quick start guide
│   └── docs/                   # Framework-specific docs
└── .opencode/
    └── skills/                 # Skill definitions
```

## By Use Case

### I want to use WizAI2
→ Start with [../wizai2/README.md](../wizai2/README.md) or [../wizai2/QUICKSTART.md](../wizai2/QUICKSTART.md)

### I want to understand the legacy system
→ Read [INDEX.md](INDEX.md) for navigation guide

### I want to generate a CLI from my code
→ [CLI_GENERATOR.md](CLI_GENERATOR.md) + [EXAMPLES.md](EXAMPLES.md) Example 1

### I want to understand how tools work
→ [LOCAL_TOOL_CALLING.md](LOCAL_TOOL_CALLING.md) + [EXAMPLES.md](EXAMPLES.md) Example 3

### I want to use the API
→ [API_REFERENCE.md](API_REFERENCE.md) + [EXAMPLES.md](EXAMPLES.md)

### I want to create an OpenSpec change
→ [TUTORIAL.md](../TUTORIAL.md) + [EXAMPLES.md](EXAMPLES.md) Example 2

## System Architecture

### High-Level Flow (Legacy)

```
User Request
    ↓
Agent (OpenSpecExecutor/SoftwareDeveloper)
    ↓
LLM (Venice AI / kimi-k2-5)
    ↓
Local Parser (detect tool intent)
    ↓
ToolRegistry (execute locally)
    ↓
Result (file changes, CLI generation, etc.)
```

### WizAI2 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
│                     (WASM + Leptos)                         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                      Axum Server                             │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐ │
│  │ REST API     │  │ WebSocket│  │ Static File Serving  │ │
│  └──────────────┘  └──────────┘  └──────────────────────┘ │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                      Core System                             │
│  ┌─────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────┐│
│  │ Agent       │ │ Memory      │ │ LLM        │ │ Tools   ││
│  │ Hierarchy   │ │ (SQLite)    │ │ (Venice)   │ │ Registry││
│  └─────────────┘ └────────────┘ └────────────┘ └─────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Contributing

When adding new documentation:

1. Add to appropriate section in [INDEX.md](INDEX.md)
2. Update links in this README.md
3. Include practical examples
4. Link to related documents

## Support

- **WizAI2 Issues**: See [../wizai2/README.md](../wizai2/README.md)
- **API Issues**: See [API_REFERENCE.md](API_REFERENCE.md) error codes
- **Tool Execution**: See [LOCAL_TOOL_CALLING.md](LOCAL_TOOL_CALLING.md)
- **Examples**: See [EXAMPLES.md](EXAMPLES.md)
- **Commands**: See [QUICK_REFERENCE.md](../QUICK_REFERENCE.md)

## License

See main [../README.md](../README.md) for license information.
