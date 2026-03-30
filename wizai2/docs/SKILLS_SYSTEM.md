# Skills System Documentation

The Skills System is a self-improving capability inspired by [Nous Research's Hermes Agent](https://github.com/nousresearch/hermes-agent). It allows agents to create, store, execute, and automatically improve reusable capabilities extracted from successful task completions.

## Overview

A **Skill** is a reusable capability that agents can:
- **Create** from successful task completions (automatic or manual)
- **Execute** by name or ID with custom inputs
- **Improve** based on usage analytics and failure patterns
- **Share** via the agentskills.io open standard format

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Skills System                           │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐ │
│  │ Storage      │  │ Executor     │  │ Generator      │ │
│  │ (SQLite)     │  │              │  │                │ │
│  └──────────────┘  └──────────────┘  └──────────────────┘ │
│  ┌──────────────┐  ┌──────────────┐                      │
│  │ Improver     │  │ Tool Registry│                      │
│  │              │  │              │                      │
│  └──────────────┘  └──────────────┘                      │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### 1. Access the Skills System

The Skills System is automatically initialized when you start Spree. The following tools are available:

### 2. Available Tools

#### `execute_skill`
Execute a skill by name or ID.

```json
{
  "name": "execute_skill",
  "arguments": {
    "skill_identifier": "code-reviewer",
    "input": {
      "code": "fn main() { println!(\"Hello\"); }"
    },
    "agent_id": "agent-uuid"
  }
}
```

#### `list_skills`
List all available skills, optionally filtered by category.

```json
{
  "name": "list_skills",
  "arguments": {
    "category": "code_generation"
  }
}
```

#### `search_skills`
Search for skills by keyword.

```json
{
  "name": "search_skills",
  "arguments": {
    "query": "rust"
  }
}
```

#### `create_skill`
Manually create a new skill.

```json
{
  "name": "create_skill",
  "arguments": {
    "name": "rust-formatter",
    "description": "Format Rust code using best practices",
    "prompt_template": "Format the following Rust code: {{input.code}}",
    "author": "user-123",
    "category": "code_generation"
  }
}
```

#### `get_skill_info`
Get detailed information about a specific skill.

```json
{
  "name": "get_skill_info",
  "arguments": {
    "skill_id": "skill-uuid"
  }
}
```

#### `suggest_skills`
Find skills relevant to a task description.

```json
{
  "name": "suggest_skills",
  "arguments": {
    "task_description": "I need to refactor this Python function"
  }
}
```

#### `get_skill_stats`
Get usage statistics across all skills.

```json
{
  "name": "get_skill_stats",
  "arguments": {}
}
```

## Skill Types

### 1. Generator
Single-turn skills that generate content based on input.

**Example:** Code reviewer, documentation generator, error explainer

### 2. Workflow
Multi-step skills that execute a sequence of actions.

**Example:** Complex refactoring workflow, multi-file operations

### 3. Analyzer
Skills that process input and return structured analysis.

**Example:** Code quality analyzer, security scanner, performance profiler

### 4. Composite
Skills that combine multiple sub-skills.

**Example:** Full code review pipeline (analyze → generate → validate)

## Automatic Skill Generation

Skills can be automatically generated from successful task completions:

### Process

1. **Task Execution** → Agent completes a task
2. **Quality Assessment** → Success rate must be ≥ 70%
3. **Pattern Extraction** → LLM analyzes the execution trace
4. **Skill Creation** → New skill created with prompt template
5. **Storage** → Skill saved to SQLite database

### Example Trigger

```rust
use spree::skills::{SkillGenerationRequest, TaskOutcome};

let request = SkillGenerationRequest {
    task_description: "Refactor error handling in auth module",
    task_context: "The auth module has inconsistent error handling...",
    execution_trace: vec![
        ExecutionStep { /* ... */ },
    ],
    outcome: TaskOutcome::Success {
        quality_score: 0.85,
        user_feedback: Some("Great work!"),
    },
    agent_id: "agent-123".to_string(),
};

let skill = skill_generator.generate_from_task(request).await?;
```

## Skill Improvement

The system automatically improves skills based on usage:

### When Skills Are Improved

- Usage count ≥ 5
- Success rate < 95%
- Recent failures detected

### Improvement Process

1. Analyze execution history
2. Identify failure patterns
3. Generate improved prompt template
4. Update triggers and examples
5. Bump version number
6. Store new version

## Database Schema

### Tables

**skills**
- `id` - Unique identifier
- `name` - Human-readable name
- `description` - What the skill does
- `version` - Semantic version
- `author` - Creator agent ID
- `category` - Skill category
- `triggers` - Keywords/patterns that trigger the skill
- `implementation` - JSON with prompt template, schemas, examples
- `usage_count` - Number of executions
- `success_rate` - Historical success rate
- `average_execution_time_ms` - Performance metric

**skill_executions**
- `id` - Execution ID
- `skill_id` - Foreign key to skills
- `agent_id` - Executing agent
- `input` - JSON input
- `output` - JSON output
- `success` - Boolean result
- `execution_time_ms` - Performance
- `executed_at` - Timestamp

**skill_generations**
- `id` - Generation ID
- `skill_id` - Created skill (nullable)
- `agent_id` - Generating agent
- `task_description` - Source task
- `execution_trace` - JSON trace
- `outcome` - Task outcome

### FTS5 Search

Full-text search is available on skill names, descriptions, and tags via the `skills_fts` virtual table.

## Agentskills.io Compatibility

Export skills in the open standard format:

```rust
let export = skill_storage.export_to_agentskills(None).await?;
let json = export.to_json()?;
```

Import from agentskills.io format:

```rust
let imported_ids = skill_storage.import_from_agentskills(&export, "agent-123").await?;
```

## Integration with Agent System

### During Task Execution

1. Agent receives task
2. System checks for relevant skills (`suggest_skills`)
3. If match found with high confidence, execute skill
4. Otherwise, proceed with normal execution
5. After completion, evaluate for skill generation

### Best Practices

1. **Name skills clearly** - Use descriptive names with keywords
2. **Add good triggers** - Include common search terms
3. **Provide examples** - Help the LLM understand expected inputs/outputs
4. **Version skills** - Update rather than replace existing skills
5. **Monitor success rates** - Remove or improve low-performing skills

## Configuration

No additional configuration required. Skills are stored in the same SQLite database as other Spree data (`data/spree.db`).

## API Examples

### Create a Simple Skill

```bash
curl -X POST http://localhost:3000/api/tools \
  -H "Content-Type: application/json" \
  -d '{
    "name": "create_skill",
    "arguments": {
      "name": "rust-linter",
      "description": "Check Rust code for common issues",
      "prompt_template": "Review this Rust code for common issues:\n\n{{input.code}}\n\nProvide specific suggestions for improvement.",
      "author": "api-user",
      "category": "code_analysis"
    }
  }'
```

### Execute a Skill

```bash
curl -X POST http://localhost:3000/api/tools \
  -H "Content-Type: application/json" \
  -d '{
    "name": "execute_skill",
    "arguments": {
      "skill_identifier": "rust-linter",
      "input": {
        "code": "fn main() { let x = 5; }"
      },
      "agent_id": "api-agent"
    }
  }'
```

## Troubleshooting

### Skills Not Found

- Check skill triggers match task keywords
- Use `search_skills` with broader terms
- Verify skill exists with `get_skill_info`

### Skill Execution Fails

- Check skill input schema matches provided input
- Review skill execution history for patterns
- Consider manual skill improvement

### Performance Issues

- Monitor `average_execution_time_ms` in skill stats
- Composite skills with many sub-skills may be slow
- Consider breaking large skills into smaller ones

## Future Enhancements

- [ ] Web UI for skill management
- [ ] Skill marketplace integration
- [ ] Advanced skill composition
- [ ] Multi-modal skill support
- [ ] Skill sharing between organizations

## References

- [Nous Research Hermes Agent](https://github.com/nousresearch/hermes-agent)
- [Agentskills.io Specification](https://agentskills.io)
- [SQLite FTS5 Documentation](https://www.sqlite.org/fts5.html)
