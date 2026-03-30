### WIZAI


<img width="632" alt="wizai" src="https://github.com/user-attachments/assets/1291d036-1361-49fc-a5c0-71dd8257cc64">

## What is WIZAI?

WIZAI is an AI-powered code generation system that automatically creates backend web servers from natural language descriptions. It uses a multi-agent architecture where specialized AI agents handle different aspects of the development process:

- **Managing Agent**: Understands user requirements and coordinates other agents
- **Architect Agent**: Designs the system architecture and API structure
- **Backend Developer Agent**: Generates Rust code using actix-web
- **Frontend Developer Agent**: Creates frontend components

## OpenSpec Execution Agent (New!)

We've added a powerful OpenSpec Execution Agent built in Rust with Venice AI integration. This provides structured workflows for managing code changes with safety and verification.

**Quick Start:**
```bash
cd wizai2
cargo run
```

Then access the **Web UI** at `http://127.0.0.1:3000` for an intuitive interface with:
- 🎯 Workflow buttons (Propose, Apply, Archive)
- 💬 Chat-style conversation interface
- 📋 Active changes sidebar
- 📄 Specs browser
- 💻 CLI mode for power users (toggle with button or Cmd/Ctrl+K)
- ⌨️ Keyboard shortcuts (Cmd/Ctrl+P, A, R)

**Available Commands:**
- `/opsx:propose <change-name>` - Create planning artifacts
- `/opsx:apply` - Implement tasks
- `/opsx:archive` - Complete changes

**📚 [Full Tutorial](TUTORIAL.md)** - Learn how to use the OpenSpec Execution Agent

**📋 [Quick Reference](QUICK_REFERENCE.md)** - Command cheatsheet

## New Features

### 🤖 SoftwareDeveloper Agent
Full-stack software development agent with:
- TypeScript, React, Vue, Node.js expertise
- PRD creation and feature implementation
- Integration with skills system

### 🛠️ TypeScript Developer Skill
Professional TypeScript development guidance:
- Type safety best practices
- React patterns and hooks
- Testing with Jest
- Project setup templates

### 🔧 CLI Generator (Rust-Native)
Automatically generate CLIs from any software:
- Analyzes code with tree-sitter AST parsing
- Generates clap-based Rust CLIs
- Supports Rust, Python, JavaScript
- Creates professional CLI structure

**Learn more:** [CLI Generator Documentation](docs/CLI_GENERATOR.md)

### ⚡ Local Tool Calling
Execute tools locally without LLM API dependencies:
- Works with any LLM provider
- Full control over tool execution
- Parse natural language intent
- Zero marginal cost for tool execution

**Learn more:** [Local Tool Calling](docs/LOCAL_TOOL_CALLING.md)

### 📖 Documentation
- **[API Reference](docs/API_REFERENCE.md)** - Complete API documentation
- **[Examples](docs/EXAMPLES.md)** - Practical usage examples
- **[CLI Generator](docs/CLI_GENERATOR.md)** - CLI generation guide
- **[Local Tool Calling](docs/LOCAL_TOOL_CALLING.md)** - Tool execution architecture

## How to use
* cargo build
* cargo run
<img width="984" alt="1" src="https://github.com/user-attachments/assets/9096964f-0132-4f22-ad8f-727c95bd8c80">
* review the code generated in `server-template/src/main.rs`

<img width="1148" alt="5" src="https://github.com/user-attachments/assets/1b3a-22f2-4b88-4866-a772-7c5d4fadf251">

* change into the server-template folder
* cargo run
* view the api_schema.json file for the created endpoints
* open your browser and navigate to your localhost:8080 and the endpoint created i.e /block_time
* The expected outcome should look like the following.

<img width="899" alt="3" src="https://github.com/user-attachments/assets/440ae950-dec6-4d56-b3bc-eefb1507a1a8">

## YouTube Video

[![WIZAI](https://img.youtube.com/vi/5Ok03ofoTeU/0.jpg)](https://www.youtube.com/watch?v=5Ok03ofoTeU)
