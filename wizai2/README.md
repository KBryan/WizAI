# Spree Agent Framework

A Rust-native, WASM-based multi-agent organizational system inspired by Agent Zero and pi-mono. Built for speed, simplicity, and easy local deployment.

## Architecture Overview

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

## Quick Start

### Prerequisites

- Rust 1.75+
- Venice AI API key

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd wizai2

# Copy environment file
cp .env.example .env

# Edit .env and add your Venice API key
# VENICE_API_KEY=your_key_here

# Build and run
cargo run
```

### Access the Application

- Web UI: http://localhost:3000
- REST API: http://localhost:3000/api/
- WebSocket: ws://localhost:3000/ws

## API Endpoints

### Agents

```bash
# List all agents
GET /api/agents

# Create a new agent
POST /api/agents
{
  "name": "My Manager",
  "role": "Manager",
  "superior_id": "optional-uuid"
}

# Get agent details
GET /api/agents/:id

# Send message to agent
POST /api/agents/:id
{
  "content": "Hello!"
}

# Get subordinates
GET /api/agents/:id/subordinates
```

### Organization

```bash
# Get organization tree
GET /api/organization

# Submit task
POST /api/tasks
{
  "agent_id": "agent-uuid",
  "task": "Do something"
}
```

### WebSocket

Connect to `ws://localhost:3000/ws` for real-time updates.

**Messages:**

```json
// Subscribe to agent
{
  "type": "Subscribe",
  "agent_id": "uuid"
}

// Send message
{
  "type": "SendMessage",
  "agent_id": "uuid",
  "content": "Hello!"
}

// Create agent
{
  "type": "CreateAgent",
  "name": "New Agent",
  "role": "Manager",
  "superior_id": "optional-uuid"
}

// Get organization
{
  "type": "GetOrganization"
}
```

## Organizational Roles

The system supports a hierarchical organization structure:

### C-Level
- **CEO**: Chief Executive Officer - Strategic decisions, overall coordination
- **CTO**: Chief Technology Officer - Technical architecture and engineering
- **CFO**: Chief Financial Officer - Financial planning and analysis
- **Chief AI Officer**: AI/ML strategy and model evaluation
- **Chief Product Officer**: Product vision and user experience

### Management
- **VP**: Vice President - Multiple teams and strategic execution
- **Director**: Department management and resource allocation
- **Manager**: Day-to-day operations and team coordination
- **Lead**: Technical leadership and mentorship

### Individual Contributors
- **Specialist**: Domain expert, task execution
- **Intern**: Learning and skill development

## Features

✅ **Hierarchical Agent System**: Superior/subordinate relationships with delegation
✅ **SQLite Memory**: Persistent storage with semantic search
✅ **Venice AI Integration**: Streaming LLM responses
✅ **Tool System**: Read/write files, execute code, delegate tasks
✅ **WebSocket**: Real-time updates and streaming
✅ **WASM UI**: Fast, reactive web interface
✅ **REST API**: Easy integration with external systems

## Development

### Project Structure

```
spree/
├── src/
│   ├── agent/          # Agent system and roles
│   ├── llm/            # Venice AI client
│   ├── memory/         # SQLite storage
│   ├── tools/          # Tool implementations
│   ├── server/         # Axum server + WebSocket
│   └── ui/             # Leptos WASM components
├── assets/             # Static files
├── prompts/            # Role-specific prompts
└── migrations/         # SQL migrations
```

### Running Tests

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

## Architecture Decisions

1. **Single Crate**: Easy to develop and run locally (workspace for production)
2. **SQLite**: Embedded, no external dependencies
3. **Axum**: Fast, modern Rust web framework
4. **Leptos**: Fine-grained reactivity for WASM
5. **Venice AI**: Uncensored, streaming support

## Roadmap

### Phase 1 (PoC) ✅
- [x] Core agent system
- [x] Basic memory
- [x] Tool system
- [x] WebSocket API
- [x] Simple UI

### Phase 2 (Production)
- [ ] Split into workspace
- [ ] Qdrant for vector DB
- [ ] Redis for pub/sub
- [ ] Authentication
- [ ] Advanced UI components
- [ ] Kubernetes deployment

### Phase 3 (Advanced)
- [ ] Multi-modal support
- [ ] Advanced tool calling
- [ ] Agent marketplace
- [ ] Workflow automation

## License

MIT

## Acknowledgments

- Inspired by [Agent Zero](https://github.com/agent0ai/agent-zero)
- Architecture influenced by [pi-mono](https://github.com/badlogic/pi-mono)
- Built with [Axum](https://github.com/tokio-rs/axum) and [Leptos](https://github.com/leptos-rs/leptos)
