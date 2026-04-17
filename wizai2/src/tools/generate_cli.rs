//! Generate CLI Tool
//!
//! Tool for generating CLIs from software source code.

use super::{ToolCall, ToolContext, ToolResult};
use crate::cli_generator::{generate_cli, GeneratedCli};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Execute the generate_cli tool
pub async fn execute(ctx: &ToolContext, call: &ToolCall) -> Result<ToolResult> {
    debug!("Executing generate_cli tool");

    // Extract arguments
    let software_path = call
        .arguments
        .get("software_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required argument: software_path"))?;

    let change_name = call
        .arguments
        .get("change_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required argument: change_name"))?;

    info!(
        "Generating CLI for {} in change {}",
        software_path, change_name
    );

    // Resolve path relative to repo root
    let full_path = ctx.repo_root.join(software_path);

    // Validate path exists
    if !full_path.exists() {
        return Ok(ToolResult {
            tool_call_id: call.id.clone(),
            success: false,
            output: format!("Software path does not exist: {}", full_path.display()),
        });
    }

    // Generate CLI
    match generate_cli(&full_path, change_name).await {
        Ok(cli) => {
            // Write CLI files to change directory
            let cli_dir = ctx
                .repo_root
                .join("openspec/changes")
                .join(change_name)
                .join("cli");

            if let Err(e) = write_cli_files(&cli, &cli_dir) {
                return Ok(ToolResult {
                    tool_call_id: call.id.clone(),
                    success: false,
                    output: format!("Failed to write CLI files: {}", e),
                });
            }

            let output = format!(
                "Successfully generated CLI for {}\n\n\
                CLI Name: {}\n\
                Location: {}\n\
                Commands: {}\n\n\
                Files created:\n\
                - Cargo.toml\n\
                - src/main.rs\n\
                - src/commands/*.rs ({} modules)\n\
                - SKILL.md\n\n\
                To build and use:\n\
                cd {}\n\
                cargo build --release\n\
                ./target/release/{}",
                software_path,
                cli.name,
                cli_dir.display(),
                cli.commands.len(),
                cli.commands.len(),
                cli_dir.display(),
                cli.name
            );

            Ok(ToolResult {
                tool_call_id: call.id.clone(),
                success: true,
                output,
            })
        }
        Err(e) => Ok(ToolResult {
            tool_call_id: call.id.clone(),
            success: false,
            output: format!("Failed to generate CLI: {}", e),
        }),
    }
}

/// Write generated CLI files to disk
fn write_cli_files(cli: &GeneratedCli, cli_dir: &Path) -> Result<()> {
    // Create directory structure
    fs::create_dir_all(cli_dir.join("src/commands"))?;

    // Write Cargo.toml
    fs::write(cli_dir.join("Cargo.toml"), &cli.cargo_toml)?;

    // Write main.rs
    fs::write(cli_dir.join("src/main.rs"), &cli.main_rs)?;

    // Write command modules
    for cmd in &cli.commands {
        let cmd_path = cli_dir.join(format!("src/commands/{}.rs", cmd.module));
        fs::write(&cmd_path, &cmd.code)?;
    }

    // Write SKILL.md
    fs::write(cli_dir.join("SKILL.md"), &cli.skill_md)?;

    info!("Wrote {} files to {}", cli.commands.len() + 3, cli_dir.display());

    Ok(())
}

/// Generate JSON schema for the tool
pub fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "software_path": {
                "type": "string",
                "description": "Path to the software source code (relative to repo root)"
            },
            "change_name": {
                "type": "string",
                "description": "Name of the OpenSpec change where CLI will be generated"
            }
        },
        "required": ["software_path", "change_name"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_cli_tool() {
        // This is a placeholder test
        // Real tests would require a mock file system
        assert!(true);
    }
}
