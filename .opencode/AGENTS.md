# Spree OpenSpec Execution Agent

## Repository Policy

This repository uses OpenSpec as its workflow contract. All agents must respect OpenSpec workflows and repository safety policies.

## Core Principles

1. **Specs First**: Always inspect specs in `openspec/specs/` before implementation
2. **Brownfield Bias**: Prefer incremental edits over broad rewrites
3. **Tool Verification**: Only claim actions verified by tool output
4. **Safety First**: Respect sandbox boundaries and approval policies

## Workflow Commands

- `/opsx:propose <change-name>` - Create a new change with planning artifacts
- `/opsx:apply` - Implement the active change's tasks
- `/opsx:archive` - Archive a completed change

## Operating Modes

### Propose Mode
- Inspect repository structure and specs
- Create change scaffold in `openspec/changes/<change-name>/`
- Generate `proposal.md`, `design.md`, `tasks.md`
- **NEVER** modify production code in this mode

### Apply Mode
- Read `tasks.md` from active change
- Implement tasks in order
- Validate via build/test commands
- Update task checklist state

### Archive Mode
- Verify all tasks complete
- Check implementation matches artifacts
- Safely archive the change

## Safety Rules

1. All file operations must stay within repository root
2. Shell commands require approval based on policy
3. Secrets from `.env` and config files must not be exposed
4. Never claim work completed without tool evidence

## Instruction Hierarchy (Highest to Lowest)

1. Runtime safety and sandbox rules
2. This AGENTS.md file
3. Loaded SKILL.md files
4. OpenSpec artifacts
5. Current user request

## Code Standards

- Rust 2021 edition
- Use `anyhow` for error handling
- Prefer `tracing` for logging
- Async/await with tokio
- Follow existing code patterns in `spree/src/`

## Testing Requirements

- Run `cargo check` before claiming Rust code is valid
- Run `cargo test` when tests exist
- Validate changes compile before marking complete

## OpenSpec Conventions

- Changes live in `openspec/changes/<change-name>/`
- Active change determined by most recent unarchived change
- Spec deltas in `delta/` folder within change
- Tasks checklist tracks implementation progress
