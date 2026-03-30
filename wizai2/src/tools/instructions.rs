//! Instruction loading tools for AGENTS.md and SKILL.md files

use super::ToolResult;
use anyhow::Result;
use std::path::Path;
use tracing::{debug, info, warn};

/// Read AGENTS.md from repository root
pub fn read_agents_rules(repo_root: &Path) -> Result<ToolResult> {
    debug!("Reading AGENTS.md from {:?}", repo_root);

    let agents_md_path = repo_root.join(".opencode/AGENTS.md");

    if !agents_md_path.exists() {
        return Ok(ToolResult {
            tool_call_id: "read_agents_rules".to_string(),
            success: false,
            output: "AGENTS.md not found at .opencode/AGENTS.md".to_string(),
        });
    }

    match std::fs::read_to_string(&agents_md_path) {
        Ok(content) => {
            info!("Successfully loaded AGENTS.md ({} bytes)", content.len());
            Ok(ToolResult {
                tool_call_id: "read_agents_rules".to_string(),
                success: true,
                output: content,
            })
        }
        Err(e) => Ok(ToolResult {
            tool_call_id: "read_agents_rules".to_string(),
            success: false,
            output: format!("Failed to read AGENTS.md: {}", e),
        }),
    }
}

/// List available skills in .opencode/skills/
pub fn list_skills(repo_root: &Path) -> Result<ToolResult> {
    debug!("Listing skills from {:?}", repo_root);

    let skills_dir = repo_root.join(".opencode/skills");

    if !skills_dir.exists() {
        return Ok(ToolResult {
            tool_call_id: "list_skills".to_string(),
            success: false,
            output: "Skills directory not found at .opencode/skills/".to_string(),
        });
    }

    let mut skills = Vec::new();

    for entry in std::fs::read_dir(&skills_dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                let skill_name = entry.file_name().to_string_lossy().to_string();
                let skill_md = path.join("SKILL.md");

                if skill_md.exists() {
                    skills.push(skill_name);
                }
            }
        }
    }

    skills.sort();

    let output = if skills.is_empty() {
        "No skills found in .opencode/skills/".to_string()
    } else {
        format!(
            "Available Skills ({}):\n{}",
            skills.len(),
            skills
                .iter()
                .map(|s| format!("  - {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    Ok(ToolResult {
        tool_call_id: "list_skills".to_string(),
        success: true,
        output,
    })
}

/// Load a specific skill's SKILL.md content
pub fn load_skill(skill_name: &str, repo_root: &Path) -> Result<ToolResult> {
    debug!("Loading skill '{}'", skill_name);

    // Validate skill name (prevent path traversal)
    if skill_name.contains("..") || skill_name.contains('/') || skill_name.contains('\\') {
        return Ok(ToolResult {
            tool_call_id: "load_skill".to_string(),
            success: false,
            output: "Invalid skill name: cannot contain path separators or '..'".to_string(),
        });
    }

    let skill_path = repo_root
        .join(".opencode/skills")
        .join(skill_name)
        .join("SKILL.md");

    if !skill_path.exists() {
        return Ok(ToolResult {
            tool_call_id: "load_skill".to_string(),
            success: false,
            output: format!(
                "Skill '{}' not found at .opencode/skills/{}/SKILL.md",
                skill_name, skill_name
            ),
        });
    }

    match std::fs::read_to_string(&skill_path) {
        Ok(content) => {
            info!(
                "Successfully loaded skill '{}' ({} bytes)",
                skill_name,
                content.len()
            );
            Ok(ToolResult {
                tool_call_id: "load_skill".to_string(),
                success: true,
                output: content,
            })
        }
        Err(e) => Ok(ToolResult {
            tool_call_id: "load_skill".to_string(),
            success: false,
            output: format!("Failed to read skill '{}': {}", skill_name, e),
        }),
    }
}

/// Get recommended skills for an OpenSpec mode
pub fn get_recommended_skills(mode: &str) -> Vec<&'static str> {
    match mode {
        "propose" => vec!["openspec-propose"],
        "apply" => vec!["openspec-apply"],
        "archive" => vec!["openspec-archive"],
        _ => vec![],
    }
}
