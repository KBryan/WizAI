# OpenSpec Execution Agent - Quick Reference

## Workflow Commands

| Command | Mode | Description |
|---------|------|-------------|
| `/opsx:propose <name>` | Propose | Create planning artifacts |
| `/opsx:apply` | Apply | Implement tasks from active change |
| `/opsx:archive` | Archive | Complete and archive change |

## Tool Categories

### File Operations
```
read_file(path="src/main.rs")
write_file(path="src/new.rs", content="...")
list_dir(path="src/", recursive=false)
grep_code(pattern="fn main", path="src/")
```

### Shell Execution
```
bash(command="cargo test")
bash(command="cargo check", working_dir="spree/")
```

### OpenSpec Workflow
```
detect_openspec_project()
list_changes(include_archived=false)
create_change_scaffold(change_name="my-feature")
read_change_artifacts(change_name="my-feature")
archive_change(change_name="my-feature")
```

### Instructions
```
read_agents_rules()
list_skills()
load_skill(skill_name="openspec-apply")
```

## Safety Checklist

✅ **Before File Operations:**
- Path is relative to repo root
- Not accessing sensitive files (.env, .ssh)
- Using `edit_file` over `write_file` when possible

✅ **Before Shell Commands:**
- Not using banned commands (rm -rf /, fork bombs)
- Command will complete within 60 seconds
- Output won't exceed 10KB

✅ **Before Marking Complete:**
- Validated with appropriate command
- Checklist updated with evidence
- Tool output supports claim

## Change Directory Structure

```
openspec/changes/my-feature/
├── proposal.md          # Problem & goals
├── design.md            # Technical approach
├── tasks.md             # Implementation checklist
└── delta/               # Spec deltas
```

## Instruction Hierarchy

```
1. Runtime Safety (cannot override)
2. AGENTS.md (repository policy)
3. SKILL.md (workflow guidance)
4. OpenSpec artifacts (specs & tasks)
5. User request
```

## Common Patterns

### Add New Module
```
read_file("src/lib.rs")
write_file("src/new_module.rs", content)
edit_file("src/lib.rs", old="...", new="...")
bash("cargo check")
```

### Update Existing Code
```
read_file("src/old.rs")
grep_code("old_function", "src/")
edit_file("src/old.rs", old="...", new="...")
bash("cargo test")
```

### Create Change
```
detect_openspec_project()
create_change_scaffold("feature-name")
write_file("openspec/changes/feature-name/proposal.md", content)
write_file("openspec/changes/feature-name/design.md", content)
write_file("openspec/changes/feature-name/tasks.md", content)
```

## Task Checklist Format

```markdown
- [ ] Task description (pending)
- [x] Task description (cargo check passed)
```

## Security Boundaries

🚫 **Always Blocked:**
- Paths outside repo root
- Sensitive files (.env, .ssh, credentials)
- Banned commands (rm -rf /, fork bombs)

⚠️ **Requires Approval (DenyDangerous mode):**
- `rm -rf` recursive deletes
- `git push --force`
- Direct device writes

## Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| Path outside repository | Absolute path or traversal | Use relative path |
| Command banned | Dangerous command | Use safer alternative |
| Command timed out | >60 seconds | Break into smaller steps |
| Tool requires approval | In DenyDangerous mode | Change mode or request approval |

## Remember

**Specs First → Tools Always → Validate Before Completing**

1. Read specs before implementing
2. Use tools for all actions
3. Validate with build/test
4. Update checklists accurately
