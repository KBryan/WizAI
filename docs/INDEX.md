# WIZAI Documentation

Welcome to the WIZAI documentation. This directory contains comprehensive guides for understanding and using the WIZAI system.

## Quick Navigation

### Getting Started
- **[TUTORIAL.md](../TUTORIAL.md)** - Step-by-step tutorial for OpenSpec Execution Agent
- **[QUICK_REFERENCE.md](../QUICK_REFERENCE.md)** - Command cheatsheet and quick reference

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
│   ├── INDEX.md                # This file
│   ├── CLI_GENERATOR.md        # CLI generation
│   ├── LOCAL_TOOL_CALLING.md   # Tool execution architecture
│   ├── API_REFERENCE.md        # API documentation
│   └── EXAMPLES.md             # Usage examples
└── .opencode/
    └── skills/                 # Skill definitions
        ├── openspec-*/
        └── typescript-dev/
```

## Feature Guides

### OpenSpec Workflow
1. Start with [TUTORIAL.md](../TUTORIAL.md)
2. Reference [LOCAL_TOOL_CALLING.md](LOCAL_TOOL_CALLING.md) for tool execution
3. See [EXAMPLES.md](EXAMPLES.md) for workflow examples

### CLI Generation
1. Read [CLI_GENERATOR.md](CLI_GENERATOR.md) for architecture
2. See [EXAMPLES.md](EXAMPLES.md) for usage examples
3. Use via `generate_cli` tool in [API_REFERENCE.md](API_REFERENCE.md)

### Software Development
1. [TUTORIAL.md](../TUTORIAL.md) for agent usage
2. `.opencode/skills/typescript-dev/SKILL.md` for TypeScript guidance
3. [EXAMPLES.md](EXAMPLES.md) for code examples

## By Use Case

### I want to...

**Generate a CLI from my code**
→ [CLI_GENERATOR.md](CLI_GENERATOR.md) + [EXAMPLES.md](EXAMPLES.md) Example 1

**Understand how tools work**
→ [LOCAL_TOOL_CALLING.md](LOCAL_TOOL_CALLING.md) + [EXAMPLES.md](EXAMPLES.md) Example 3

**Use the API**
→ [API_REFERENCE.md](API_REFERENCE.md) + [EXAMPLES.md](EXAMPLES.md)

**Create an OpenSpec change**
→ [TUTORIAL.md](../TUTORIAL.md) + [EXAMPLES.md](EXAMPLES.md) Example 2

**Build a full-stack app**
→ [TUTORIAL.md](../TUTORIAL.md) + TypeScript skill + SoftwareDeveloper agent

## System Architecture

### High-Level Flow

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

### Key Components

1. **Agent System** ([TUTORIAL.md](../TUTORIAL.md))
   - Hierarchical agent organization
   - Role-based permissions
   - Message passing

2. **Tool Registry** ([LOCAL_TOOL_CALLING.md](LOCAL_TOOL_CALLING.md))
   - 13 registered tools
   - Local execution
   - Intent parsing

3. **CLI Generator** ([CLI_GENERATOR.md](CLI_GENERATOR.md))
   - Tree-sitter analysis
   - Code generation
   - Template system

4. **OpenSpec Workflow** ([TUTORIAL.md](../TUTORIAL.md))
   - Structured changes
   - Task management
   - Version control

## Contributing

When adding new documentation:

1. Add to appropriate section in this INDEX
2. Update links in README.md
3. Include practical examples
4. Link to related documents

## Support

- API Issues: See [API_REFERENCE.md](API_REFERENCE.md) error codes
- Tool Execution: See [LOCAL_TOOL_CALLING.md](LOCAL_TOOL_CALLING.md)
- Examples: See [EXAMPLES.md](EXAMPLES.md)
- Commands: See [QUICK_REFERENCE.md](../QUICK_REFERENCE.md)

## License

See main README.md for license information.
