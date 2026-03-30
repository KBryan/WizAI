# Local Tool Calling

## Overview

**Local Tool Calling** is an architecture pattern that enables AI agents to execute functions locally without requiring LLM API support for native tool calling. This approach provides full control over tool execution and works with any LLM provider.

## The Problem

### Native Tool Calling Limitations

Most LLM providers (OpenAI, Anthropic) support "function calling" or "tools" in their API:

```json
{
  "model": "gpt-4",
  "messages": [...],
  "tools": [
    {
      "name": "create_change_scaffold",
      "description": "Create OpenSpec change scaffold",
      "parameters": {...}
    }
  ],
  "tool_choice": "auto"
}
```

The LLM can then respond with structured tool calls:

```json
{
  "tool_calls": [
    {
      "name": "create_change_scaffold",
      "arguments": {"change_name": "my-feature"}
    }
  ]
}
```

### Venice AI Limitations

**Venice AI** (and some other providers) don't support this structured tool calling API:

- **venice-uncensored**: Rejects `tools` and `tool_choice` parameters
- **kimi-k2-5**: Has streaming format incompatibilities
- Response: `{"error": "tools is not supported by this model"}`

## The Solution: Local Tool Calling

### Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐
│ User Request│────▶│ Agent        │────▶│ LLM (Venice AI) │
└─────────────┘     │ Executor     │     └─────────────────┘
                    └──────────────┘              │
                          │                       │
                          │                       ▼
                          │              ┌─────────────────┐
                          │              │ Natural Lang    │
                          │              │ Response        │
                          │              │ (with tool hint)│
                          │              └─────────────────┘
                          │                       │
                          ▼                       │
                   ┌──────────────┐              │
                   │ Parse Intent │◀─────────────┘
                   └──────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Execute Tool │
                   │ Locally      │
                   └──────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Return Result│
                   └──────────────┘
```

### Key Insight

Instead of relying on the LLM API to return structured tool calls, we:

1. **Send tool descriptions** in the system prompt (as text)
2. **Ask the LLM** to respond naturally, optionally mentioning tools
3. **Parse the response** locally to detect tool intent
4. **Execute the tool** using our ToolRegistry
5. **Return the result** to the user (or optionally back to LLM)

## Implementation

### Phase 1: Tool Descriptions in System Prompt

Enhance the OpenSpecExecutor role prompt with tool descriptions:

```rust
AgentRole::OpenSpecExecutor => format!(
    "You are {}, an OpenSpec Execution Agent...
    
    Available Tools:
    - detect_openspec_project: Detect OpenSpec project structure
    - list_changes: List all changes in openspec/changes/
    - create_change_scaffold: Create new change scaffold
      Parameters: {{"change_name": "string"}}
    - read_change_artifacts: Read change documentation
    - archive_change: Archive completed change
    
    When you need to use a tool, mention it clearly in your response.
    Example: 'I'll create a change scaffold using the create_change_scaffold tool.'",
    name
),
```

### Phase 2: Intent Parsing

Parse the LLM response to detect tool intent:

```rust
impl ToolRegistry {
    /// Parse natural language for tool intent
    pub fn parse_intent(&self, response: &str) -> Option<ToolCall> {
        // Check for explicit tool blocks
        if let Some(tool_block) = self.extract_tool_block(response) {
            return Some(tool_block);
        }
        
        // Check for intent patterns
        for tool in self.tools.values() {
            if self.detect_tool_intent(response, &tool.name) {
                // Extract arguments from context
                let args = self.extract_arguments(response, tool);
                return Some(ToolCall {
                    id: Uuid::new_v4().to_string(),
                    name: tool.name.clone(),
                    arguments: args,
                });
            }
        }
        
        None
    }
    
    fn detect_tool_intent(&self, response: &str, tool_name: &str) -> bool {
        let patterns = [
            format!("{} tool", tool_name),
            format!("using {}", tool_name),
            format!("I'll {}", tool_name),
            format!("Let me {}", tool_name),
        ];
        
        patterns.iter().any(|p| response.to_lowercase().contains(&p.to_lowercase()))
    }
}
```

### Phase 3: Tool Execution

Execute detected tools locally:

```rust
pub async fn execute_task(&self, task: TaskRequest) -> Result<TaskResult> {
    // Get LLM response
    let response = self.call_llm(&task).await?;
    
    // Check for tool intent
    let tool_registry = self.registry.read().await.get_tools().await;
    
    if let Some(tool_call) = tool_registry.read().await.parse_intent(&response) {
        info!("Detected tool intent: {}", tool_call.name);
        
        // Execute the tool
        let tool_ctx = ToolContext::new(self.get_repo_root());
        match tool_registry.read().await.execute(&tool_ctx, &tool_call).await {
            Ok(result) => {
                // Append tool result to response
                let final_response = format!(
                    "{}\n\n[Tool Result]: {}",
                    response, result.output
                );
                
                return Ok(TaskResult {
                    success: true,
                    response: final_response,
                    actions_taken: vec![format!("Executed: {}", tool_call.name)],
                });
            }
            Err(e) => {
                error!("Tool execution failed: {}", e);
            }
        }
    }
    
    // No tool detected, return LLM response directly
    Ok(TaskResult {
        success: true,
        response,
        actions_taken: vec![],
    })
}
```

### Phase 4: Multi-Turn Tool Execution

For complex workflows requiring multiple tools:

```rust
pub async fn execute_with_tools(&self, task: &str) -> Result<String> {
    let mut conversation = vec![task.to_string()];
    let mut max_iterations = 5;
    
    loop {
        // Get LLM response
        let response = self.call_llm_with_history(&conversation).await?;
        conversation.push(response.clone());
        
        // Check for tool intent
        if let Some(tool_call) = self.parse_tool_intent(&response) {
            // Execute tool
            let result = self.execute_tool(&tool_call).await?;
            
            // Add tool result to conversation
            conversation.push(format!(
                "Tool {} result: {}",
                tool_call.name, result.output
            ));
            
            max_iterations -= 1;
            if max_iterations == 0 {
                return Ok("Max tool iterations reached".to_string());
            }
            continue;
        }
        
        // No more tools needed, return final response
        return Ok(response);
    }
}
```

## Parsing Strategies

### Strategy 1: Explicit Tool Blocks

LLM outputs structured tool calls in code blocks:

**System Prompt Instruction**:
```
When you need to use a tool, output it in this format:

```tool
{"name": "create_change_scaffold", "arguments": {"change_name": "my-feature"}}
```
```

**Parser**:
```rust
pub fn parse_tool_block(content: &str) -> Option<ToolCall> {
    if let Some(start) = content.find("```tool") {
        if let Some(end) = content[start..].find("```") {
            let json_str = &content[start + 7..start + end].trim();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                return Some(ToolCall {
                    id: Uuid::new_v4().to_string(),
                    name: json["name"].as_str()?.to_string(),
                    arguments: json["arguments"].clone(),
                });
            }
        }
    }
    None
}
```

### Strategy 2: Intent Detection

Parse natural language for tool mentions:

```rust
pub fn detect_intent(&self, response: &str) -> Vec<ToolIntent> {
    let mut intents = Vec::new();
    
    // Pattern matching
    let patterns = [
        (r"create.*change.*scaffold", "create_change_scaffold"),
        (r"list.*changes", "list_changes"),
        (r"archive.*change", "archive_change"),
        (r"detect.*openspec", "detect_openspec_project"),
    ];
    
    for (pattern, tool_name) in patterns {
        if regex::Regex::new(pattern).unwrap().is_match(&response.to_lowercase()) {
            intents.push(ToolIntent {
                tool_name: tool_name.to_string(),
                confidence: 0.8,
            });
        }
    }
    
    intents
}
```

### Strategy 3: Keyword Matching

Simple keyword-based detection:

```rust
pub fn match_keywords(&self, response: &str, tool: &Tool) -> bool {
    let tool_keywords: HashMap<&str, Vec<&str>> = [
        ("create_change_scaffold", vec!["create", "scaffold", "change", "new"]),
        ("list_changes", vec!["list", "changes", "show", "display"]),
        ("archive_change", vec!["archive", "complete", "finish", "done"]),
    ].into_iter().collect();
    
    if let Some(keywords) = tool_keywords.get(tool.name.as_str()) {
        let response_lower = response.to_lowercase();
        let matches = keywords.iter()
            .filter(|k| response_lower.contains(*k))
            .count();
        
        return matches >= 2; // Require at least 2 keyword matches
    }
    
    false
}
```

## Argument Extraction

Extract arguments from natural language:

```rust
pub fn extract_arguments(&self, response: &str, tool: &Tool) -> serde_json::Value {
    let mut args = serde_json::Map::new();
    
    // Extract quoted strings as potential arguments
    let quoted_pattern = regex::Regex::new(r#""([^"]+)""#).unwrap();
    let quoted: Vec<&str> = quoted_pattern.captures_iter(response)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str())
        .collect();
    
    // Map to tool parameters
    for (i, param) in tool.parameters.iter().enumerate() {
        if let Some(value) = quoted.get(i) {
            args.insert(param.clone(), serde_json::json!(value));
        }
    }
    
    serde_json::Value::Object(args)
}
```

## Benefits

### 1. Provider Independence
Works with any LLM:
- Venice AI
- Local models (Llama, Mistral)
- OpenAI (if you switch)
- Anthropic

### 2. Full Control
- Decide when to execute tools
- Validate arguments before execution
- Handle errors gracefully
- Custom retry logic

### 3. Cost Efficiency
- Single LLM call can trigger multiple local tools
- No need for provider's expensive tool-calling API
- Tools execute locally at zero marginal cost

### 4. Extensibility
- Add new tools without API changes
- Custom tool implementations
- Integration with existing codebases

## Comparison: Native vs Local

| Feature | Native Tool Calling | Local Tool Calling |
|---------|--------------------|-------------------|
| **Provider Support** | Limited (OpenAI, Anthropic) | Universal |
| **Latency** | Lower (single API call) | Higher (parsing + execution) |
| **Control** | Limited | Full |
| **Error Handling** | API-level | Application-level |
| **Extensibility** | Restricted | Unlimited |
| **Cost** | Higher per call | Lower (local execution) |

## Example: OpenSpec Workflow

### User Request
```
"Create a new OpenSpec change called 'refactor-auth'"
```

### LLM Response (Natural Language)
```
I'll help you create a new OpenSpec change for refactoring authentication.
Let me use the create_change_scaffold tool to set up the change structure.

```tool
{"name": "create_change_scaffold", "arguments": {"change_name": "refactor-auth"}}
```
```

### Local Execution
```rust
// Parse detects tool block
let tool_call = parse_tool_block(&response)?;

// Execute locally
let result = execute_tool(&tool_call).await?;
// Result: "Created change scaffold for 'refactor-auth'"

// Append to response
let final_response = format!(
    "{}\n\n✅ Tool executed successfully:\n{}",
    response, result.output
);
```

### Final Output
```
I'll help you create a new OpenSpec change for refactoring authentication.
Let me use the create_change_scaffold tool to set up the change structure.

```tool
{"name": "create_change_scaffold", "arguments": {"change_name": "refactor-auth"}}
```

✅ Tool executed successfully:
Created change scaffold for 'refactor-auth'

Directories created:
- openspec/changes/refactor-auth/
- openspec/changes/refactor-auth/delta/

Next steps:
1. Write proposal.md
2. Write design.md
3. Write tasks.md
```

## Integration with CLI Generator

Local tool calling works seamlessly with the CLI generator:

```rust
// Agent detects user wants to generate a CLI
if intent == "generate_cli" {
    // Execute generate_cli tool locally
    let result = generate_cli(&software_path, &change_name).await?;
    
    // Result includes generated CLI paths
    println!("Generated CLI at: {}", result.output);
    
    // Can then execute the generated CLI
    let cli_output = std::process::Command::new(&cli_path)
        .arg("--help")
        .output()?;
}
```

## Testing

### Unit Tests

```rust
#[test]
fn test_parse_tool_block() {
    let response = r#"
    I'll create the scaffold.
    
    ```tool
    {"name": "create_change_scaffold", "arguments": {"change_name": "test"}}
    ```
    "#;
    
    let tool_call = parse_tool_block(response).unwrap();
    assert_eq!(tool_call.name, "create_change_scaffold");
}

#[test]
fn test_intent_detection() {
    let response = "Let me list all the changes";
    let intent = detect_intent(response);
    assert_eq!(intent.tool_name, "list_changes");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_tool_execution_flow() {
    // Setup
    let executor = AgentExecutor::new(...);
    
    // Execute task
    let result = executor.execute_task(TaskRequest {
        task: "Create change scaffold 'test-change'".to_string(),
        context: None,
    }).await?;
    
    // Verify tool was executed
    assert!(result.actions_taken.contains("create_change_scaffold"));
    assert!(Path::new("openspec/changes/test-change").exists());
}
```

## Configuration

### Environment Variables

```bash
# Tool execution settings
MAX_TOOL_ITERATIONS=5
TOOL_TIMEOUT_SECONDS=30
ENABLE_TOOL_AUTO_EXEC=true

# Parsing settings
INTENT_DETECTION_THRESHOLD=0.7
USE_EXPLICIT_BLOCKS=true
```

### Runtime Configuration

```rust
pub struct ToolCallingConfig {
    pub max_iterations: usize,
    pub timeout: Duration,
    pub auto_execute: bool,
    pub require_confirmation: bool,
    pub parsing_strategy: ParsingStrategy,
}

pub enum ParsingStrategy {
    ExplicitBlocks,  // Only parse ```tool blocks
    IntentDetection, // Detect natural language intent
    Hybrid,          // Try both
}
```

## See Also

- [CLI Generator](CLI_GENERATOR.md) - Generates CLIs that can be called as tools
- [Tool Registry](API_REFERENCE.md) - Tool registration and execution
- [Examples](EXAMPLES.md) - Practical local tool calling examples
