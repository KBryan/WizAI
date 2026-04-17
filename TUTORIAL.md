# OpenSpec Execution Agent Tutorial

## Overview

The OpenSpec Execution Agent is a workflow-aware AI agent built in Rust that operates on OpenSpec-compliant repositories. It provides structured workflows for proposing changes, implementing them, and archiving completed work.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenSpec Execution Agent                  │
├─────────────────────────────────────────────────────────────┤
│  Venice AI          │  OpenSpec Workflow  │  Rust Runtime   │
│  - Function Calling  │  - Propose          │  - Tools        │
│  - Reasoning         │  - Apply            │  - Sandbox      │
│  - Tool Selection    │  - Archive          │  - Validation   │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
            ┌──────────────┐    ┌──────────────┐
            │  AGENTS.md   │    │  SKILL.md    │
            │  (Policy)    │    │  (Guidance)  │
            └──────────────┘    └──────────────┘
```

## Quick Start

### 1. Prerequisites

- Rust 1.75+ installed
- Venice AI API key set in environment: `VENICE_API_KEY`
- OpenSpec repository structure

### 2. Verify Installation

```bash
cd wizai2
cargo check
```

### 3. Run the Agent

```bash
cargo run
```

The agent will start a server on `http://127.0.0.1:3000`.

## Workflow Modes

### Mode 1: Propose (`/opsx:propose <change-name>`)

**Purpose**: Create planning artifacts for a new change without modifying production code.

**When to use**:
- Starting a new feature
- Planning a refactoring
- Documenting technical design

**Steps**:

1. **Detect OpenSpec Project**
   ```
   Tool: detect_openspec_project
   ```

2. **Inspect Repository**
   ```
   Tools: list_dir, read_file, grep_code
   ```

3. **Create Change Scaffold**
   ```
   Tool: create_change_scaffold
   Arguments: {"change_name": "my-feature"}
   ```

4. **Generate Artifacts**
   ```
   Tools: write_file
   Paths:
     - openspec/changes/my-feature/proposal.md
     - openspec/changes/my-feature/design.md
     - openspec/changes/my-feature/tasks.md
   ```

**Example Session**:

```
User: /opsx:propose add-user-authentication

Agent: I'll help you create a proposal for user authentication.

[Tool] detect_openspec_project
[Tool] list_dir(path="openspec/specs/")
[Tool] create_change_scaffold("add-user-authentication")
[Tool] write_file(path="openspec/changes/add-user-authentication/proposal.md", content="...")
[Tool] write_file(path="openspec/changes/add-user-authentication/design.md", content="...")
[Tool] write_file(path="openspec/changes/add-user-authentication/tasks.md", content="...")

Done! Created change scaffold at openspec/changes/add-user-authentication/
```

**Artifacts Created**:

```
openspec/changes/my-feature/
├── proposal.md          # Problem, motivation, goals
├── design.md            # Technical approach, risks
├── tasks.md             # Implementation checklist
└── delta/               # Spec deltas (optional)
```

### Mode 2: Apply (`/opsx:apply`)

**Purpose**: Implement tasks from the active change's tasks.md.

**When to use**:
- After proposal is approved
- Ready to write code
- Tasks are well-defined

**Steps**:

1. **Load Context**
   ```
   Tools: read_agents_rules, load_skill
   Tool: list_changes
   ```

2. **Read Active Change**
   ```
   Tool: read_change_artifacts
   Arguments: {"change_name": "<active>"}
   ```

3. **Process Each Task**
   ```
   For each incomplete task:
   - read_file (relevant specs)
   - read_file (source files)
   - edit_file or write_file (implement)
   - bash (validate: cargo check/test)
   - update checklist
   ```

**Example Session**:

```
User: /opsx:apply

Agent: I'll implement the active change.

[Tool] read_agents_rules
[Tool] load_skill("openspec-apply")
[Tool] list_changes
[Tool] read_change_artifacts("add-user-authentication")

Found 3 tasks:
1. [ ] Create User model
2. [ ] Add authentication middleware
3. [ ] Write tests

Starting with task 1...

[Tool] read_file(path="openspec/specs/user-model.md")
[Tool] read_file(path="src/models/mod.rs")
[Tool] write_file(path="src/models/user.rs", content="...")
[Tool] edit_file(path="src/models/mod.rs", old="...", new="...")
[Tool] bash(command="cargo check")

Task 1 complete ✓

[Updating tasks.md...]

1. [x] Create User model (validated: cargo check passed)
2. [ ] Add authentication middleware
3. [ ] Write tests

...

All tasks complete! Validated with cargo test.
```

**Key Points**:
- Tasks are processed in order
- Each task requires validation
- Checklist is updated with evidence
- No shortcuts - every claim verified

### Mode 3: Archive (`/opsx:archive`)

**Purpose**: Safely archive a completed change.

**When to use**:
- All tasks are complete
- Code is validated
- Ready to close the change

**Steps**:

1. **Verify Completion**
   ```
   Tool: read_change_artifacts
   Check: All tasks marked complete?
   Check: Validation evidence present?
   ```

2. **Validate Implementation**
   ```
   Tool: bash (final validation)
   Compare: Implementation vs design.md
   ```

3. **Archive**
   ```
   Tool: archive_change
   Arguments: {"change_name": "<active>"}
   ```

**Example Session**:

```
User: /opsx:archive

Agent: I'll archive the completed change.

[Tool] read_change_artifacts("add-user-authentication")

Verifying completion...
✓ All tasks complete
✓ Validation evidence present
✓ Implementation matches design

[Tool] bash(command="cargo test")
All tests passed.

[Tool] archive_change("add-user-authentication")

Change archived to: openspec/changes/add-user-authentication-archived-20241228/
```

## Available Tools

### Repository Tools

| Tool | Description | Requires Approval |
|------|-------------|------------------|
| `read_file` | Read file contents | No |
| `write_file` | Write content to file | No |
| `list_dir` | List directory contents | No |
| `grep_code` | Search for patterns in files | No |
| `bash` | Execute shell commands | Yes |

### OpenSpec Workflow Tools

| Tool | Description |
|------|-------------|
| `detect_openspec_project` | Verify OpenSpec structure |
| `list_changes` | Show active/archived changes |
| `create_change_scaffold` | Set up change directory |
| `read_change_artifacts` | Load proposal/design/tasks |
| `archive_change` | Complete and archive change |

### Instruction Tools

| Tool | Description |
|------|-------------|
| `read_agents_rules` | Load AGENTS.md policy |
| `list_skills` | Discover available skills |
| `load_skill` | Load SKILL.md content |

## Security Features

### Path Sandboxing

All file operations are sandboxed to the repository root:

```rust
// ✅ Allowed
read_file(path="src/main.rs")
write_file(path="openspec/changes/my-feature/proposal.md")

// ❌ Blocked
read_file(path="/etc/passwd")
write_file(path="../other-repo/file.rs")
```

### Command Safety

**Banned Commands** (always blocked):
- `rm -rf /` or `rm -rf /*`
- Fork bombs: `:(){ :|:& };:`
- Disk operations: `mkfs`, `format`

**Dangerous Commands** (require approval in DenyDangerous mode):
- `rm -rf` recursive deletes
- `git push --force`
- Direct writes to `/dev/*`

### Secret Protection

Access to these files is blocked:
- `.env` files
- SSH keys (`~/.ssh/*`)
- AWS credentials (`~/.aws/*`)
- Kubernetes configs (`~/.kube/*`)

### Execution Limits

- **Timeout**: 60 seconds per command
- **Output**: 10KB cap (truncated if exceeded)
- **Retries**: Limited for failed commands

## Best Practices

### 1. Always Start with Context

```
Good:
[Tool] read_agents_rules
[Tool] load_skill("openspec-apply")
[Tool] detect_openspec_project

Then: Start working
```

### 2. Inspect Before Mutating

```
Good:
[Tool] read_file(path="src/models/mod.rs")
[Tool] edit_file(...)  // Now I know the structure

Bad:
[Tool] write_file(path="src/models/mod.rs", content="...")  // Overwrites!
```

### 3. Validate Before Completing

```
Good:
[Tool] edit_file(path="src/lib.rs", ...)
[Tool] bash(command="cargo check")
"Task complete - cargo check passed"

Bad:
[Tool] edit_file(path="src/lib.rs", ...)
"Task complete"  // Did it compile?
```

### 4. Update Checklists Accurately

```
Good:
- [x] Implement User struct (cargo check passed)

Bad:
- [x] Implement User struct  // No validation mentioned
```

## Common Tasks

### Task: Add a New Module

```
[Tool] read_file(path="src/lib.rs")  // See current structure
[Tool] write_file(path="src/new_module.rs", content="...")
[Tool] edit_file(path="src/lib.rs", old="...", new="...")  // Add mod statement
[Tool] bash(command="cargo check")
```

### Task: Refactor a Function

```
[Tool] read_file(path="src/utils.rs")  // Understand current code
[Tool] grep_code(pattern="old_function_name")  // Find all usages
[Tool] edit_file(path="src/utils.rs", old="...", new="...")
[Tool] bash(command="cargo test")  // Ensure tests still pass
```

### Task: Update Documentation

```
[Tool] read_file(path="README.md")  // Check current docs
[Tool] edit_file(path="README.md", old="...", new="...")
[Tool] bash(command="cargo doc")  // Verify docs build
```

## Troubleshooting

### Issue: "Path outside repository root"

**Cause**: Attempting to access files outside the sandbox.

**Fix**: Use relative paths from repository root:
```
❌ read_file(path="/absolute/path/to/file.rs")
✅ read_file(path="src/file.rs")
```

### Issue: "Command is banned for security reasons"

**Cause**: Using a dangerous or banned command.

**Fix**: Use safer alternatives:
```
❌ rm -rf src/
✅ rm src/old_file.rs  // Safer, specific target
```

### Issue: "Command timed out after 60 seconds"

**Cause**: Long-running command exceeded timeout.

**Fix**: Break into smaller steps or run manually.

### Issue: "Tool requires approval but mode is DenyDangerous"

**Cause**: Attempting bash command in restrictive mode.

**Fix**: Change approval mode or request approval.

## Instruction Hierarchy

The agent follows this precedence order:

1. **Runtime Safety** - Cannot be overridden
2. **AGENTS.md** - Repository policy
3. **SKILL.md** - Workflow guidance
4. **OpenSpec Artifacts** - Change specifications
5. **User Request** - Your instructions

**Example**: If a user asks to skip validation but AGENTS.md requires it, validation is still required.

## Advanced Usage

### Creating Custom Skills

Add new skills to `.opencode/skills/<skill-name>/SKILL.md`:

```markdown
# Custom Skill

## Purpose

Describe what this skill does.

## When to Use

Trigger conditions.

## Workflow Steps

1. Step 1
2. Step 2

## Tool Sequence

```
1. tool_name(args)
2. other_tool(args)
```
```

### Modifying AGENTS.md

Update `.opencode/AGENTS.md` to change repository policy:

```markdown
## Custom Rules

- Always run `cargo fmt` after editing Rust files
- Require tests for all new modules
- Prefer `edit_file` over `write_file`
```

### Working with Specs

Before implementing, always check `openspec/specs/`:

```
[Tool] list_dir(path="openspec/specs/")
[Tool] read_file(path="openspec/specs/api-contract.md")
[Tool] read_file(path="openspec/specs/data-model.md")
```

## Summary

The OpenSpec Execution Agent provides:

✓ **Structured Workflows**: Propose → Apply → Archive  
✓ **Safety**: Sandboxed execution, banned commands  
✓ **Verification**: Tools required for all claims  
✓ **Documentation**: AGENTS.md and SKILL.md guidance  
✓ **Brownfield**: Incremental edits, spec-driven  

Remember: **Specs first, tools always, validate before completing.**
