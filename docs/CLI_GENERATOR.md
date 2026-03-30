# CLI Generator

## Overview

The **CLI Generator** is a Rust-native tool that automatically generates command-line interfaces from any software codebase. It analyzes source code using tree-sitter AST parsing and generates professional clap-based Rust CLIs with full type safety.

## Architecture

```
Source Code → Analyzer → SoftwareModel → Code Generator → Rust CLI
     ↑                                              ↓
  (Rust/Python/JS)                          (Cargo.toml + main.rs + commands)
```

### Components

### 1. SoftwareAnalyzer (`src/cli_generator/analyzer.rs`)

Parses source code to extract CLI-relevant structures:

- **Language Support**: Rust, Python, JavaScript
- **AST Parsing**: Uses tree-sitter for accurate code analysis
- **Extracts**:
  - Public APIs (functions, methods)
  - Module structure
  - Entry points (main functions)
  - Stateful operations
  - Function parameters and types

**Key Methods**:
- `new()` - Initialize with language parsers
- `analyze(path)` - Analyze entire codebase
- `detect_language(path)` - Auto-detect primary language
- `analyze_file(path, language)` - Parse individual files

### 2. SoftwareModel (`src/cli_generator/model.rs`)

Data structures representing analyzed software:

```rust
pub struct SoftwareModel {
    pub name: String,           // Project name
    pub language: Language,     // Rust/Python/JavaScript
    pub modules: Vec<Module>,   // Code modules
    pub public_apis: Vec<Api>,  // Public functions
    pub stateful_ops: Vec<Operation>,
    pub entry_points: Vec<EntryPoint>,
}
```

**Api Structure**:
```rust
pub struct Api {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, type)
    pub is_async: bool,
}
```

**Generated Artifacts**:
- `generate_command_groups()` - Group APIs into CLI commands
- `generate_cli_args()` - Convert API params to CLI arguments
- `has_async_operations()` - Detect async functions

### 3. CliGenerator (`src/cli_generator/codegen.rs`)

Generates Rust CLI code using Askama templates:

**Generated Files**:
1. **Cargo.toml** - Project configuration with clap dependencies
2. **main.rs** - Entry point with clap derive macros
3. **command modules** - Individual command handlers
4. **SKILL.md** - Documentation for the generated CLI

**Template System**:
- `CargoTomlTemplate` - Cargo configuration
- `MainTemplate` - CLI entry point with subcommands
- `CommandTemplate` - Individual command implementations

**Features**:
- Type-safe argument parsing
- Automatic help generation
- Async/sync support detection
- Error handling
- JSON output support

### 4. Templates (`src/cli_generator/templates.rs`)

Askama template structures for code generation:

- Type mapping (Rust types → CLI arg types)
- Clap attribute generation
- Command grouping logic

## Usage

### Basic Usage

```rust
use spree_agent::cli_generator::generate_cli;
use std::path::Path;

async fn create_cli() {
    // Generate CLI from software
    let cli = generate_cli(
        Path::new("/path/to/software"),
        "my-generated-cli"
    ).await?;
    
    // Access generated files
    println!("CLI Name: {}", cli.name);
    println!("Cargo.toml:\n{}", cli.cargo_toml);
    println!("main.rs:\n{}", cli.main_rs);
    
    // Write to disk
    std::fs::write("output/Cargo.toml", cli.cargo_toml)?;
    std::fs::write("output/src/main.rs", cli.main_rs)?;
    
    for cmd in cli.commands {
        std::fs::write(
            format!("output/src/{}.rs", cmd.name),
            cmd.content
        )?;
    }
}
```

### Via Tool Registry

The `generate_cli` tool is registered and can be invoked by agents:

```json
{
  "tool": "generate_cli",
  "arguments": {
    "software_path": "src/agent",
    "change_name": "agent-cli"
  }
}
```

**Tool Schema**:
- `software_path`: Path to software to analyze (relative to repo root)
- `change_name`: Name for the generated CLI project

**Output**:
Generated CLI is written to `openspec/changes/{change_name}/`

## Example Output

### Input: Simple Rust Module

```rust
// src/calculator.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub async fn fetch_data(url: String) -> Result<String> {
    // async operation
}
```

### Generated CLI Structure

```
cli-anything-calculator/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── add.rs          # Command for add function
│   └── fetch_data.rs   # Command for fetch_data function
└── SKILL.md
```

### Generated main.rs

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cli-anything-calculator")]
#[command(about = "Generated CLI for calculator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add two numbers
    Add(AddArgs),
    /// Fetch data from URL
    FetchData(FetchDataArgs),
}

#[derive(Parser)]
struct AddArgs {
    #[arg(short, long)]
    pub a: i32,
    #[arg(short, long)]
    pub b: i32,
}

#[derive(Parser)]
struct FetchDataArgs {
    #[arg(short, long)]
    pub url: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Add(args) => {
            let result = calculator::add(args.a, args.b);
            println!("{}", result);
        }
        Commands::FetchData(args) => {
            let result = calculator::fetch_data(args.url).await;
            match result {
                Ok(data) => println!("{}", data),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}
```

### Generated Cargo.toml

```toml
[package]
name = "cli-anything-calculator"
version = "1.0.0"
edition = "2021"

[dependencies]
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Compilation and Usage

After generating the CLI:

```bash
# Navigate to generated CLI
cd openspec/changes/agent-cli

# Build the CLI
cargo build --release

# Run the generated CLI
./target/release/cli-anything-agent --help

# Use specific commands
./target/release/cli-anything-agent add --a 5 --b 3
./target/release/cli-anything-agent fetch-data --url "https://api.example.com"
```

## Advanced Features

### Language-Specific Analysis

**Rust**:
- Parses `pub fn` functions
- Detects async/await
- Extracts parameter types from signatures
- Handles generic types

**Python**:
- Parses `def` functions
- Detects `async def`
- Extracts type hints
- Handles default arguments

**JavaScript**:
- Parses function declarations
- Detects async functions
- Extracts JSDoc types
- Handles ES6+ syntax

### Coverage Analysis

The model includes coverage analysis to identify:
- Which APIs are exposed as CLI commands
- Missing coverage gaps
- Suggestions for new commands

```rust
let analysis = model.analyze_coverage();
println!("Coverage: {}%", analysis.coverage_percent());
for gap in analysis.gaps {
    println!("Missing in {}: {:?}", gap.module, gap.missing);
}
```

## Integration with Tool Registry

The CLI generator integrates with the Tool Registry:

```rust
// In tool registry
self.register(Tool {
    name: "generate_cli".to_string(),
    description: "Generate a CLI from software source code".to_string(),
    parameters: schema(),
    handler: Box::new(|ctx, call| {
        // Execute CLI generation
        generate_cli(&path, &change_name)
    }),
});
```

This allows agents to generate CLIs automatically:

```
User: "Create a CLI for the agent module"
Agent: [Executes generate_cli tool]
      → Analyzes src/agent/
      → Generates CLI code
      → Returns success with file paths
```

## Error Handling

The CLI generator handles various error conditions:

- **Invalid path**: Returns descriptive error
- **Unsupported language**: Falls back to generic parsing
- **Parse errors**: Skips malformed files, continues analysis
- **Template errors**: Returns detailed error messages

## Future Enhancements

Planned improvements:

1. **More Languages**: Go, TypeScript, Java, C++
2. **Smart Grouping**: AI-powered command categorization
3. **Interactive Mode**: Generated CLIs with interactive prompts
4. **Plugin System**: Custom templates and transformations
5. **Web Interface**: GUI for CLI generation

## See Also

- [Local Tool Calling](LOCAL_TOOL_CALLING.md) - How tools integrate with agents
- [API Reference](API_REFERENCE.md) - Tool registry and execution
- [Examples](EXAMPLES.md) - Practical usage examples
