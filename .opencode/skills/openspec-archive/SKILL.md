# OpenSpec Archive Skill

## Purpose

Guide the agent through safely archiving a completed OpenSpec change.

## When to Use

Triggered by `/opsx:archive` command.

## Workflow Steps

### 1. Verify Completion

- Read `tasks.md` from active change
- Verify all tasks marked complete (`- [x]`)
- Check for validation evidence on each task
- If incomplete tasks exist, report and stop

### 2. Verify Implementation

- Compare implementation against design.md
- Check that proposal goals are met
- Verify no orphaned changes in source code
- Run final validation suite

### 3. Spec Reconciliation

- Check if spec deltas exist in `delta/` folder
- Apply deltas to main specs if present
- Update spec version numbers if applicable

### 4. Archive Change

- Move change from `openspec/changes/<name>/` to `openspec/changes/archived/<name>/`
- Or rename with timestamp: `<name>-archived-<timestamp>/`
- Preserve all artifacts for historical reference

### 5. Cleanup

- Clear active change reference
- Update any change tracking state

## Safety Checks

Before archiving, confirm:
- All tasks complete: YES/NO
- Validation evidence present: YES/NO
- Implementation matches design: YES/NO
- Specs reconciled (if needed): YES/NO

If any check fails, report the issue and request override.

## Tool Sequence Example

```
1. detect_openspec_project
2. read_file(path="openspec/changes/<active>/tasks.md")
3. Verify all tasks complete
4. read_file(path="openspec/changes/<active>/design.md")
5. Compare with implementation
6. run_bash (final validation)
7. Check for deltas in openspec/changes/<active>/delta/
8. If deltas exist, apply to main specs
9. archive_change
```

## Success Criteria

- Change is archived safely
- All tasks verified complete
- Implementation matches design
- Specs reconciled if deltas exist
- Archive location is correct
