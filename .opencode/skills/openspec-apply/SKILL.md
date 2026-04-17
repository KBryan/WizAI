# OpenSpec Apply Skill

## Purpose

Guide the agent through implementing tasks from an active OpenSpec change.

## When to Use

Triggered by `/opsx:apply` command.

## Workflow Steps

### 1. Resolve Active Change

- Use `detect_openspec_project` to confirm structure
- Use `list_changes` to find unarchived changes
- Select most recent unarchived change as active
- Read `tasks.md` from the active change

### 2. Process Tasks in Order

For each incomplete task:

1. **Understand**: Read relevant specs and source files
2. **Plan**: Determine what changes are needed
3. **Implement**: Edit or create files
4. **Validate**: Run build/test commands
5. **Update**: Mark task complete in tasks.md

### 3. Implementation Guidelines

- Read spec files before modifying code
- Prefer incremental edits (`edit_file` over `write_file`)
- Group related changes together
- Validate after each logical unit of work

### 4. Task Checklist Updates

Update task state in `tasks.md`:
- `- [ ]` incomplete
- `- [x]` complete
- Include evidence: built, tested, verified

### 5. Validation Commands

Always run appropriate validation:
- Rust: `cargo check`, `cargo test`
- TypeScript: `tsc`, `npm test`
- Python: `pytest`, `mypy`

Do not mark tasks complete without validation.

## Tool Sequence Example

```
1. detect_openspec_project
2. list_changes
3. read_file(path="openspec/changes/<active>/tasks.md")
4. For each task:
   - read_file (relevant specs)
   - read_file (source files)
   - edit_file or write_file
   - run_bash (validation)
   - update_tasks_checklist
```

## Success Criteria

- All tasks from tasks.md are addressed
- Each task has validation evidence
- Checklist is updated accurately
- No tasks remain incomplete without explanation
