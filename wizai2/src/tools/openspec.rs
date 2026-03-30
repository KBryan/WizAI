//! OpenSpec workflow tools for managing changes and specifications

use super::{ToolContext, ToolResult};
use anyhow::{anyhow, Result};
use std::path::Path;
use tracing::{debug, info, warn};

/// Detect if this is an OpenSpec project and return project info
pub fn detect_openspec_project(repo_root: &Path) -> Result<ToolResult> {
    debug!("Detecting OpenSpec project in {:?}", repo_root);

    let openspec_dir = repo_root.join("openspec");
    let specs_dir = openspec_dir.join("specs");
    let changes_dir = openspec_dir.join("changes");
    let config_file = openspec_dir.join("config.yaml");

    let is_openspec = openspec_dir.exists() && openspec_dir.is_dir();
    let has_specs = specs_dir.exists() && specs_dir.is_dir();
    let has_changes = changes_dir.exists() && changes_dir.is_dir();
    let has_config = config_file.exists();

    if !is_openspec {
        return Ok(ToolResult {
            tool_call_id: "detect_openspec_project".to_string(),
            success: false,
            output: "Not an OpenSpec project (openspec/ directory not found)".to_string(),
        });
    }

    // Count specs and changes
    let spec_count = if has_specs {
        std::fs::read_dir(&specs_dir)?.count()
    } else {
        0
    };

    let change_count = if has_changes {
        std::fs::read_dir(&changes_dir)?
            .filter(|e| {
                if let Ok(entry) = e {
                    entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                } else {
                    false
                }
            })
            .count()
    } else {
        0
    };

    let output = format!(
        "OpenSpec Project Detected\n\n\
        Repository Root: {}\n\
        OpenSpec Directory: {}\n\
        Specs Directory: {} ({} specs)\n\
        Changes Directory: {} ({} changes)\n\
        Config File: {}",
        repo_root.display(),
        if openspec_dir.exists() { "✓" } else { "✗" },
        if has_specs { "✓" } else { "✗" },
        spec_count,
        if has_changes { "✓" } else { "✗" },
        change_count,
        if has_config { "✓" } else { "✗" }
    );

    Ok(ToolResult {
        tool_call_id: "detect_openspec_project".to_string(),
        success: true,
        output,
    })
}

/// List all changes in the openspec/changes/ directory
pub fn list_changes(repo_root: &Path, include_archived: bool) -> Result<ToolResult> {
    debug!("Listing changes in {:?}", repo_root);

    let changes_dir = repo_root.join("openspec/changes");

    if !changes_dir.exists() {
        return Ok(ToolResult {
            tool_call_id: "list_changes".to_string(),
            success: false,
            output: "Changes directory not found. Run detect_openspec_project first.".to_string(),
        });
    }

    let mut active_changes = Vec::new();
    let mut archived_changes = Vec::new();

    for entry in std::fs::read_dir(&changes_dir)? {
        if let Ok(entry) = entry {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            if path.is_dir() {
                if name.starts_with("archived") || name.ends_with("-archived") {
                    if include_archived {
                        archived_changes.push(name);
                    }
                } else {
                    // Check if there's a tasks.md
                    let has_tasks = path.join("tasks.md").exists();
                    active_changes.push(format!(
                        "{} (tasks: {})",
                        name,
                        if has_tasks { "✓" } else { "✗" }
                    ));
                }
            }
        }
    }

    let mut output = String::new();
    output.push_str(&format!("Active Changes ({}):\n", active_changes.len()));
    if active_changes.is_empty() {
        output.push_str("  (none)\n");
    } else {
        for change in active_changes {
            output.push_str(&format!("  - {}\n", change));
        }
    }

    if include_archived {
        output.push_str(&format!(
            "\nArchived Changes ({}):\n",
            archived_changes.len()
        ));
        if archived_changes.is_empty() {
            output.push_str("  (none)\n");
        } else {
            for change in archived_changes {
                output.push_str(&format!("  - {}\n", change));
            }
        }
    }

    Ok(ToolResult {
        tool_call_id: "list_changes".to_string(),
        success: true,
        output,
    })
}

/// Create a new change scaffold
pub fn create_change_scaffold(change_name: &str, repo_root: &Path) -> Result<ToolResult> {
    debug!("Creating change scaffold for '{}'", change_name);

    let changes_dir = repo_root.join("openspec/changes");
    let change_dir = changes_dir.join(change_name);
    let delta_dir = change_dir.join("delta");

    // Validate change name (no path traversal)
    if change_name.contains("..") || change_name.contains('/') || change_name.contains('\\') {
        return Ok(ToolResult {
            tool_call_id: "create_change_scaffold".to_string(),
            success: false,
            output: "Invalid change name: cannot contain path separators or '..'".to_string(),
        });
    }

    // Check if already exists
    if change_dir.exists() {
        return Ok(ToolResult {
            tool_call_id: "create_change_scaffold".to_string(),
            success: false,
            output: format!("Change '{}' already exists", change_name),
        });
    }

    // Create directories
    std::fs::create_dir_all(&change_dir)?;
    std::fs::create_dir_all(&delta_dir)?;

    info!("Created change scaffold at {:?}", change_dir);

    Ok(ToolResult {
        tool_call_id: "create_change_scaffold".to_string(),
        success: true,
        output: format!(
            "Created change scaffold for '{}'\n\n\
            Directories created:\n\
            - openspec/changes/{}/\n\
            - openspec/changes/{}/delta/\n\n\
            Next steps:\n\
            1. Write proposal.md\n\
            2. Write design.md\n\
            3. Write tasks.md",
            change_name, change_name, change_name
        ),
    })
}

/// Read artifacts from a change directory
pub fn read_change_artifacts(change_name: &str, repo_root: &Path) -> Result<ToolResult> {
    debug!("Reading artifacts for change '{}'", change_name);

    let change_dir = repo_root.join("openspec/changes").join(change_name);

    if !change_dir.exists() {
        return Ok(ToolResult {
            tool_call_id: "read_change_artifacts".to_string(),
            success: false,
            output: format!("Change '{}' not found", change_name),
        });
    }

    let proposal_path = change_dir.join("proposal.md");
    let design_path = change_dir.join("design.md");
    let tasks_path = change_dir.join("tasks.md");

    let mut output = format!("Artifacts for change '{}':\n\n", change_name);

    if proposal_path.exists() {
        let content = std::fs::read_to_string(&proposal_path)?;
        output.push_str("## proposal.md\n");
        output.push_str(&content);
        output.push_str("\n\n");
    } else {
        output.push_str("## proposal.md\n(not found)\n\n");
    }

    if design_path.exists() {
        let content = std::fs::read_to_string(&design_path)?;
        output.push_str("## design.md\n");
        output.push_str(&content);
        output.push_str("\n\n");
    } else {
        output.push_str("## design.md\n(not found)\n\n");
    }

    if tasks_path.exists() {
        let content = std::fs::read_to_string(&tasks_path)?;
        output.push_str("## tasks.md\n");
        output.push_str(&content);
    } else {
        output.push_str("## tasks.md\n(not found)");
    }

    Ok(ToolResult {
        tool_call_id: "read_change_artifacts".to_string(),
        success: true,
        output,
    })
}

/// Archive a completed change
pub fn archive_change(change_name: &str, repo_root: &Path) -> Result<ToolResult> {
    debug!("Archiving change '{}'", change_name);

    let changes_dir = repo_root.join("openspec/changes");
    let change_dir = changes_dir.join(change_name);

    if !change_dir.exists() {
        return Ok(ToolResult {
            tool_call_id: "archive_change".to_string(),
            success: false,
            output: format!("Change '{}' not found", change_name),
        });
    }

    // Verify tasks are complete
    let tasks_path = change_dir.join("tasks.md");
    if tasks_path.exists() {
        let tasks_content = std::fs::read_to_string(&tasks_path)?;
        let incomplete_tasks = tasks_content
            .lines()
            .filter(|l| l.starts_with("- [ ]"))
            .count();

        if incomplete_tasks > 0 {
            return Ok(ToolResult {
                tool_call_id: "archive_change".to_string(),
                success: false,
                output: format!(
                    "Cannot archive: {} incomplete tasks in tasks.md",
                    incomplete_tasks
                ),
            });
        }
    }

    // Create archive destination with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d");
    let archive_name = format!("{}-archived-{}", change_name, timestamp);
    let archive_dir = changes_dir.join(&archive_name);

    // Move the change directory
    std::fs::rename(&change_dir, &archive_dir)?;

    info!("Archived change '{}' to {:?}", change_name, archive_dir);

    Ok(ToolResult {
        tool_call_id: "archive_change".to_string(),
        success: true,
        output: format!(
            "Successfully archived change '{}'\n\
            Archived to: openspec/changes/{}/",
            change_name, archive_name
        ),
    })
}

/// Get the active change (most recent unarchived)
pub fn get_active_change(repo_root: &Path) -> Result<Option<String>> {
    let changes_dir = repo_root.join("openspec/changes");

    if !changes_dir.exists() {
        return Ok(None);
    }

    let mut active_changes: Vec<_> = std::fs::read_dir(&changes_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            e.path().is_dir() && !name.starts_with("archived") && !name.ends_with("-archived")
        })
        .map(|e| {
            let path = e.path();
            let modified = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            (e.file_name().to_string_lossy().to_string(), modified)
        })
        .collect();

    // Sort by modification time, newest first
    active_changes.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(active_changes.first().map(|(name, _)| name.clone()))
}
