# OpenSpec Propose Skill

## Purpose

Guide the agent through creating a new OpenSpec change with planning artifacts.

## When to Use

Triggered by `/opsx:propose <change-name>` command.

## Workflow Steps

### 1. Inspect Repository

- Use `detect_openspec_project` to confirm OpenSpec structure
- List specs directory to understand current specifications
- Read relevant source files to understand context

### 2. Create Change Scaffold

- Create directory: `openspec/changes/<change-name>/`
- Create subdirectories: `delta/`

### 3. Generate Planning Artifacts

Create these files in the change directory:

#### proposal.md
- **Status**: Draft
- **Summary**: What problem does this solve?
- **Motivation**: Why is this needed?
- **Goals**: What will be achieved
- **Non-goals**: What's explicitly out of scope

#### design.md
- **Overview**: High-level approach
- **Changes**: What will be modified
- **Risks**: Potential issues
- **Alternatives**: Other approaches considered

#### tasks.md
- Task checklist with checkboxes: `- [ ] Task description`
- Break work into 2-hour chunks
- Include validation steps
- Order by dependency

### 4. Restrictions

- **NEVER** modify production source code
- **NEVER** mark tasks as complete
- Only create planning artifacts

## Tool Sequence Example

```
1. detect_openspec_project
2. list_dir(path="openspec/specs/")
3. read_file(path="openspec/config.yaml")
4. Create change scaffold
5. write_file(path="openspec/changes/<name>/proposal.md", content="...")
6. write_file(path="openspec/changes/<name>/design.md", content="...")
7. write_file(path="openspec/changes/<name>/tasks.md", content="...")
```

## Success Criteria

- Change directory exists with proper structure
- All three planning artifacts are present
- No production code was modified
