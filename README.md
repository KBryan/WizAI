# WizAI

> AI-powered multi-agent organizational system with hierarchical delegation, real-time web UI, and Venice AI integration.

<img width="632" alt="wizai" src="https://github.com/user-attachments/assets/1291d036-1361-49fc-a5c0-71dd8257cc64">

## Overview

WizAI provides two complementary systems:

### 1. Spree Agent Framework (`wizai2/`)

A Rust-native, WASM-based multi-agent organizational system inspired by Agent Zero and pi-mono. Built for speed, simplicity, and easy local deployment.

**Key features:**

- **Hierarchical Agent System** — CEO → C-level → Managers → Specialists with task delegation
- **Venice AI Integration** — Streaming LLM responses via Venice AI API
- **SQLite Memory** — Persistent storage with semantic search
- **Tool System** — Read/write files, execute code, delegate tasks
- **WebSocket API** — Real-time updates and streaming
- **Leptos WASM Dashboard** — Agent dashboard with lead pipeline, CMA widget, market intelligence, and communication hub
- **MCP Client** — Model Context Protocol integration for tool discovery
- **Web Scraping** — Real estate data scraping with SerpAPI and ScrapingBee
- **Payment System** — Per-token LLM pricing, compute billing, and automatic invoicing
- **Skills System** — Create, store, improve, and execute reusable agent skills
- **Compliance & Audit** — Approval workflows and audit logging

### 2. DAppWiz Code Generator (`src/`)

The original WizAI system that automatically creates backend web servers from natural language descriptions using a multi-agent architecture:

- **Managing Agent** — Understands requirements and coordinates other agents
- **Architect Agent** — Designs system architecture and API structure
- **Backend Developer Agent** — Generates Rust code using actix-web
- **Frontend Developer Agent** — Creates frontend components

## Quick Start

### Prerequisites

- **Rust 1.75+** (for wizai2/Spree)
- **Venice AI API key** ([venice.ai](https://venice.ai))

### Run the Spree Agent Framework

```bash
git clone https://github.com/KBryan/WizAI.git
cd WizAI/wizai2

# Configure API key
cp .env.example .env
# Edit .env and add: VENICE_API_KEY=your_key_here

# Build and run
cargo run
```

The server starts at `http://127.0.0.1:3000` with:

- **Web UI** — `http://localhost:3000`
- **REST API** — `http://localhost:3000/api/`
- **WebSocket** — `ws://localhost:3000/ws`

### Run the DAppWiz Code Generator

```bash
cd WizAI
cargo build
cargo run
```

Review the generated code in `server-template/src/main.rs`, then:

```bash
cd server-template
cargo run
```

Open `http://localhost:8080` and check the generated endpoint (e.g., `/block_time`).

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   User Interface                      │
│                (WASM + Leptos Dashboard)              │
└───────────────────────────────┬─────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│                   Axum Server                         │
│  ┌────────────┐ ┌──────────┐ ┌────────────────┐ │
│  │ REST API     │ │ WebSocket│ │ Static Files    │ │
│  └────────────┘ └──────────┘ └────────────────┘ │
└───────────────────────────────┬─────────────────────────────────┘
                       │
                       ▼
┌────────────────────────────────────────────────────────┐
│                   Core System                         │
│  ┌───────────┐ ┌──────────┐ ┌──────┐ ┌─────┐ │
│  │ Agent       │ │ Memory      │ │ LLM    │ │ Tools   │ │
│  │ Hierarchy   │ │ (SQLite)   │ │(Venice)│ │ Reg.    │ │
│  └───────────┘ └──────────┘ └──────┘ └─────┘ │
└────────────────────────────────────────────────────────┘
```

## Project Structure

```
WizAI/
├── wizai2/                          # Spree Agent Framework
│   ├── src/
│   │   ├── main.rs                  # Entry point
│   │   ├── agent/                   # Agent system & roles
│   │   │   ├── core.rs              # Agent types, registry, events
│   │   │   ├── executor.rs          # Task execution with delegation
│   │   │   └── roles/               # Role implementations (CEO, CTO, etc.)
│   │   ├── server/                  # Axum server + WebSocket
│   │   ├── llm/                     # Venice AI client
│   │   ├── memory/                  # SQLite storage & embeddings
│   │   ├── tools/                   # Tool implementations
│   │   ├── skills/                  # Skill system (create, execute, improve)
│   │   ├── mcp/                     # Model Context Protocol client
│   │   ├── models/                  # Data models (leads, market, communication)
│   │   ├── services/                # Business logic (leads, research, communication)
│   │   ├── payments/                # Payment system & invoicing
│   │   ├── compliance/             # Audit logging & approval workflows
│   │   ├── cli_generator/           # CLI generation from code
│   │   └── webscraping/            # Web scraping (SerpAPI, ScrapingBee)
│   ├── ui/                          # Leptos/WASM dashboard
│   ├── migrations/                  # SQLite migrations
│   ├── assets/                      # Static HTML/CSS/JS
│   └── docs/                        # API & feature documentation
├── src/                             # Original DAppWiz code generator
│   ├── ai_functions/                # AI function definitions
│   ├── apis/                        # API call handling
│   ├── models/                      # Agent models & traits
│   └── helpers/                     # General utilities
├── server-template/                 # Generated server output
├── docs/                            # DAppWiz documentation
└── openspec/                        # OpenSpec change management specs
```

## API Endpoints

### Agents

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/agents` | List all agents |
| `POST` | `/api/agents` | Create a new agent |
| `GET` | `/api/agents/:id` | Get agent details |
| `POST` | `/api/agents/:id` | Send message to agent |
| `GET` | `/api/agents/:id/subordinates` | Get agent subordinates |

### Organization

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/organization` | Get organization tree |
| `POST` | `/api/tasks` | Submit a task |

### WebSocket

Connect to `ws://localhost:3000/ws` for real-time updates:

```json
// Subscribe to agent
{ "type": "Subscribe", "agent_id": "uuid" }

// Send message
{ "type": "SendMessage", "agent_id": "uuid", "content": "Hello!" }

// Create agent
{ "type": "CreateAgent", "name": "New Agent", "role": "Manager", "superior_id": "optional-uuid" }

// Get organization
{ "type": "GetOrganization" }
```

## Organizational Roles

The Spree system supports a hierarchical organization structure:

**C-Level:** CEO (Strategic decisions), CTO (Technical architecture), CFO (Financial planning), Chief AI Officer (AI/ML strategy), Chief Product Officer (Product vision)

**Management:** VP, Director, Manager, Lead

**Individual Contributors:** Specialist, Intern

## Configuration

Environment variables (set in `.env`):

| Variable | Default | Description |
|----------|---------|-------------|
| `VENICE_API_KEY` | *required* | Venice AI API key |
| `VENICE_BASE_URL` | `https://api.venice.ai` | Venice API base URL |
| `PORT` | `3000` | Server port |
| `HOST` | `127.0.0.1` | Server host |
| `DATABASE_URL` | `spree.db` | SQLite database path |

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Production build
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

## Dashboard (Leptos/WASM)

The web UI includes:

- **Home** — Dashboard overview with key metrics
- **Leads Pipeline** — Kanban board with drag-and-drop
- **CMA Generator** — Comparative Market Analysis with price recommendations
- **Market Intelligence** — Area data and trends
- **Communication Hub** — Draft management and approval workflow
- **AI Copilot** — Chat interface with streaming responses

## Documentation

- [API Reference](docs/API_REFERENCE.md) — Complete API documentation
- [Examples](docs/EXAMPLES.md) — Practical usage examples
- [CLI Generator](docs/CLI_GENERATOR.md) — CLI generation guide
- [Local Tool Calling](docs/LOCAL_TOOL_CALLING.md) — Tool execution architecture
- [Tutorial](TUTORIAL.md) — OpenSpec workflow tutorial
- [Quick Reference](QUICK_REFERENCE.md) — Command cheatsheet

## Demo

```bash
cd wizai2
./demo.sh
```

The interactive demo creates an organizational hierarchy, shows API responses, and demonstrates agent coordination.

## YouTube

[![WIZAI](https://img.youtube.com/vi/5Ok03ofoTeU/0.jpg)](https://www.youtube.com/watch?v=5Ok03ofoTeU)

## License

MIT

## Acknowledgments

- Inspired by [Agent Zero](https://github.com/agent0ai/agent-zero)
- Architecture influenced by [pi-mono](https://github.com/badlogic/pi-mono)
- Built with [Axum](https://github.com/tokio-rs/axum) and [Leptos](https://github.com/leptos-rs/leptos)
