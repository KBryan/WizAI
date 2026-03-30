# Examples

## Example 1: Generating a CLI from Existing Code

### Scenario
You have a Rust module with useful functions and want to expose them as a CLI.

### Input: Agent Module

```rust
// src/agent/manager.rs
pub fn create_agent(name: String, role: String) -> Agent {
    Agent::new(name, role)
}

pub fn list_agents() -> Vec<Agent> {
    AGENT_REGISTRY.list()
}

pub async fn send_message(agent_id: String, content: String) -> Result<Message> {
    let agent = find_agent(&agent_id)?;
    agent.send(content).await
}

pub fn delete_agent(agent_id: String) -> Result<()> {
    AGENT_REGISTRY.remove(&agent_id)
}
```

### Step 1: Generate CLI

Using the agent to generate the CLI:

```bash
# Via API
POST /api/tasks
{
  "agent_id": "openspec-executor-id",
  "task": "Generate a CLI from the agent module at src/agent/manager.rs"
}
```

Or via command line:

```rust
use spree_agent::cli_generator::generate_cli;

async fn main() {
    let cli = generate_cli(
        Path::new("src/agent"),
        "agent-manager-cli"
    ).await?;
    
    // Generated files are written to:
    // openspec/changes/agent-manager-cli/
}
```

### Step 2: Generated CLI Structure

```
openspec/changes/agent-manager-cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── create_agent.rs
│   ├── list_agents.rs
│   ├── send_message.rs
│   └── delete_agent.rs
└── SKILL.md
```

### Step 3: Build and Use

```bash
cd openspec/changes/agent-manager-cli

# Build
cargo build --release

# Get help
./target/release/cli-anything-agent --help

# Usage examples
./target/release/cli-anything-agent create-agent --name "MyAgent" --role "Specialist"
./target/release/cli-anything-agent list-agents
./target/release/cli-anything-agent send-message --agent-id "123" --content "Hello"
./target/release/cli-anything-agent delete-agent --agent-id "123"
```

---

## Example 2: Manual OpenSpec Workflow

### Scenario
You want to refactor authentication in your project using OpenSpec workflow.

### Step 1: Propose

Create the change scaffold:

```bash
# Via API
POST /api/tasks
{
  "agent_id": "openspec-executor-id",
  "task": "/opsx:propose refactor-authentication"
}

# Or manually
mkdir -p openspec/changes/refactor-authentication/delta
```

Create `openspec/changes/refactor-authentication/proposal.md`:

```markdown
# Refactor Authentication

## Problem
Current auth system is monolithic and hard to test.

## Goals
- Extract auth into separate module
- Add JWT token support
- Improve error handling
- Add unit tests

## Success Criteria
- All existing tests pass
- New auth module has 90%+ coverage
- No breaking changes to API
```

### Step 2: Design

Create `openspec/changes/refactor-authentication/design.md`:

```markdown
# Design: Authentication Refactor

## Architecture

### Current
```
src/
├── main.rs (contains auth logic)
└── handlers.rs (uses auth)
```

### New
```
src/
├── main.rs
├── auth/
│   ├── mod.rs
│   ├── jwt.rs
│   ├── middleware.rs
│   └── errors.rs
└── handlers.rs
```

## Implementation Plan

1. Create `src/auth/` module
2. Move auth logic from `main.rs`
3. Implement JWT tokens
4. Add middleware
5. Update handlers
6. Write tests
7. Update documentation
```

### Step 3: Tasks

Create `openspec/changes/refactor-authentication/tasks.md`:

```markdown
# Tasks

## Phase 1: Setup
- [x] Create change scaffold
- [x] Write proposal.md
- [x] Write design.md
- [x] Create tasks.md

## Phase 2: Implementation
- [ ] Create auth module structure
- [ ] Implement JWT token generation
- [ ] Implement JWT token validation
- [ ] Create auth middleware
- [ ] Refactor main.rs to use auth module
- [ ] Update handler dependencies

## Phase 3: Testing
- [ ] Write unit tests for jwt.rs
- [ ] Write unit tests for middleware.rs
- [ ] Write integration tests
- [ ] Ensure all existing tests pass

## Phase 4: Documentation
- [ ] Update API documentation
- [ ] Add auth examples
- [ ] Update CHANGELOG.md
```

### Step 4: Apply

Implement tasks manually or with agent:

```bash
# Create auth module
cargo run --bin DAppWiz
# User: "Create a Rust module for JWT authentication"

# Write tests
cargo test --lib auth

# Mark tasks complete
# (Edit tasks.md to check off completed items)
```

### Step 5: Archive

Once all tasks are complete:

```bash
# Verify all tasks done
grep "\[ \]" openspec/changes/refactor-authentication/tasks.md
# Should return no incomplete tasks

# Archive the change
mv openspec/changes/refactor-authentication \
   openspec/changes/refactor-authentication-archived-20240328
```

---

## Example 3: Local Tool Calling Flow

### Scenario
User asks the agent to detect OpenSpec project status.

### Request

```bash
POST /api/tasks
{
  "agent_id": "openspec-executor-id",
  "task": "Check if this is an OpenSpec project and show me the current changes"
}
```

### System Prompt (Simplified)

```
You are an OpenSpec Execution Agent.

Available Tools:
1. detect_openspec_project - Detect project structure
2. list_changes - List all changes in openspec/changes/

When you need to use a tool, mention it clearly and I'll execute it for you.
```

### LLM Response (Venice AI)

```
I'll check if this is an OpenSpec project and list the changes for you.

Let me use the detect_openspec_project tool to analyze the project structure,
then the list_changes tool to show you the current changes.

```tool
{"name": "detect_openspec_project", "arguments": {}}
```
```

### Local Execution

The agent executor parses the response:

```rust
// Detect tool block
let tool_call = parse_tool_block(&response)?;
// tool_call.name = "detect_openspec_project"

// Execute locally
let result = execute_tool(&tool_call).await?;
// result.output = "OpenSpec Project Detected\n\nRepository Root: /Users/...\nSpecs Directory: ✓ (2 specs)\nChanges Directory: ✓ (5 changes)..."

// Append result
let response = format!("{}\n\n{}", response, result.output);
```

### Second Tool Call

LLM sees the result and requests next tool:

```
Great! It is an OpenSpec project. Now let me list the changes:

```tool
{"name": "list_changes", "arguments": {"include_archived": false}}
```
```

### Final Execution

```rust
// Execute list_changes
let result = execute_tool(&tool_call).await?;
// result.output = "Active Changes (5):\n  - refactor-authentication (tasks: ✓)\n  - add-jwt-support (tasks: ✗)..."
```

### Final Response to User

```
I'll check if this is an OpenSpec project and list the changes for you.

Let me use the detect_openspec_project tool to analyze the project structure,
then the list_changes tool to show you the current changes.

```tool
{"name": "detect_openspec_project", "arguments": {}}
```

✅ Tool Result:
OpenSpec Project Detected

Repository Root: /Users/.../WizAI
Specs Directory: ✓ (2 specs)
Changes Directory: ✓ (5 changes)
Config File: ✓

Great! It is an OpenSpec project. Now let me list the changes:

```tool
{"name": "list_changes", "arguments": {"include_archived": false}}
```

✅ Tool Result:
Active Changes (5):
  - refactor-authentication (tasks: ✓)
  - add-jwt-support (tasks: ✗)
  - update-docs (tasks: ✓)
  - cli-generator (tasks: ✓)
  - fix-bug-123 (tasks: ✗)

You have 3 completed changes and 2 in progress.
```

---

## Example 4: Multi-Tool Workflow

### Scenario
Create a complete feature with multiple steps requiring different tools.

### Request

```bash
POST /api/tasks
{
  "agent_id": "openspec-executor-id",
  "task": "Create a new feature 'add-user-profiles' with proposal, design, and tasks documents"
}
```

### Execution Flow

**Turn 1:**
- LLM: "I'll create the OpenSpec change scaffold first..."
- Tool: `create_change_scaffold({"change_name": "add-user-profiles"})`
- Result: Scaffold created

**Turn 2:**
- LLM: "Now I'll write the proposal document..."
- Tool: `write_file({"path": "openspec/changes/add-user-profiles/proposal.md", "content": "..."})`
- Result: Proposal written

**Turn 3:**
- LLM: "Creating the design document..."
- Tool: `write_file({"path": "openspec/changes/add-user-profiles/design.md", "content": "..."})`
- Result: Design written

**Turn 4:**
- LLM: "Finally, the tasks document..."
- Tool: `write_file({"path": "openspec/changes/add-user-profiles/tasks.md", "content": "..."})`
- Result: Tasks written

**Final Response:**
```
✅ Created OpenSpec change 'add-user-profiles'

Created:
- openspec/changes/add-user-profiles/proposal.md
- openspec/changes/add-user-profiles/design.md
- openspec/changes/add-user-profiles/tasks.md

Next steps:
1. Review the documents
2. Execute tasks with /opsx:apply
3. Archive with /opsx:archive when complete
```

---

## Example 5: Error Handling

### Scenario
Tool execution fails, agent handles gracefully.

### Request

```bash
POST /api/tasks
{
  "agent_id": "openspec-executor-id",
  "task": "Create change scaffold 'invalid/name'"
}
```

### Execution

```rust
// Tool validation fails
if change_name.contains("/") {
    return Err("Invalid change name: cannot contain path separators");
}
```

### Response

```
I attempted to create the change scaffold, but encountered an error:

❌ Tool Error: Invalid change name: cannot contain path separators

The change name 'invalid/name' is not valid. Please use a name without:
- Forward slashes (/)
- Backslashes (\\)
- Path traversal sequences (..)

Would you like me to try again with a different name, such as 'invalid-name'?
```

---

## Example 6: SoftwareDeveloper Agent Usage

### Scenario
Get TypeScript/React code advice from SoftwareDeveloper agent.

### Create Agent

```bash
POST /api/agents
{
  "name": "Frontend Expert",
  "role": "SoftwareDeveloper"
}
```

### Request

```bash
POST /api/tasks
{
  "agent_id": "software-developer-id",
  "task": "Help me create a React component for a user profile card with TypeScript"
}
```

### Response

The agent uses its TypeScript skill:

```typescript
import React, { useState } from 'react';

interface User {
  id: string;
  name: string;
  email: string;
  avatar?: string;
}

interface UserProfileCardProps {
  user: User;
  onEdit?: (user: User) => void;
  onDelete?: (userId: string) => void;
}

export const UserProfileCard: React.FC<UserProfileCardProps> = ({
  user,
  onEdit,
  onDelete
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div className="user-profile-card">
      <img 
        src={user.avatar || '/default-avatar.png'} 
        alt={user.name}
        className="avatar"
      />
      <h3>{user.name}</h3>
      <p>{user.email}</p>
      
      {isExpanded && (
        <div className="actions">
          {onEdit && (
            <button onClick={() => onEdit(user)}>Edit</button>
          )}
          {onDelete && (
            <button onClick={() => onDelete(user.id)}>Delete</button>
          )}
        </div>
      )}
      
      <button onClick={() => setIsExpanded(!isExpanded)}>
        {isExpanded ? 'Less' : 'More'}
      </button>
    </div>
  );
};
```

**Additional guidance from TypeScript skill:**
- Define interfaces for all props
- Use optional chaining for callbacks
- Keep component pure when possible
- Add error boundaries for production

---

## See Also

- [CLI Generator](CLI_GENERATOR.md) - Full CLI generation documentation
- [Local Tool Calling](LOCAL_TOOL_CALLING.md) - Architecture and implementation
- [API Reference](API_REFERENCE.md) - Complete API documentation
