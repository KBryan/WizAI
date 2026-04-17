use super::{ToolCall, ToolResult};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Banned commands that are too dangerous
const BANNED_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    ":(){ :|:& };:", // fork bomb
    "dd if=/dev/zero",
    "mkfs",
    "format",
];

/// Commands that require explicit approval
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf",
    "rm -r /",
    "git push --force",
    "git push -f",
    "> /dev",
];

/// Validate working directory is within repo root
fn validate_working_dir(working_dir: Option<&str>, repo_root: &Path) -> Result<PathBuf> {
    let dir = if let Some(dir) = working_dir {
        repo_root.join(dir)
    } else {
        repo_root.to_path_buf()
    };
    
    let canonical = dir.canonicalize().unwrap_or(dir.clone());
    let canonical_root = repo_root.canonicalize().unwrap_or(repo_root.to_path_buf());
    
    if !canonical.starts_with(&canonical_root) {
        return Err(anyhow!(
            "Working directory '{}' is outside repository root",
            dir.display()
        ));
    }
    
    Ok(canonical)
}

/// Check if command is banned or dangerous
fn check_command_safety(command: &str) -> (bool, bool) {
    let cmd_lower = command.to_lowercase();
    
    // Check banned commands
    if BANNED_COMMANDS.iter().any(|banned| cmd_lower.contains(&banned.to_lowercase())) {
        return (false, true);
    }
    
    // Check dangerous patterns
    let is_dangerous = DANGEROUS_PATTERNS.iter()
        .any(|pattern| cmd_lower.contains(&pattern.to_lowercase()));
    
    (true, is_dangerous)
}

pub async fn execute_code(
    language: &str,
    code: &str,
    working_dir: Option<&str>,
) -> Result<ToolResult> {
    debug!("Executing {} code", language);
    
    let (cmd, args) = match language {
        "python" => ("python3", vec!["-c", code]),
        "bash" | "shell" => ("bash", vec!["-c", code]),
        "javascript" | "node" => ("node", vec!["-e", code]),
        "rust" => return execute_rust(code).await,
        _ => return Err(anyhow!("Unsupported language: {}", language)),
    };
    
    let mut command = Command::new(cmd);
    command.args(&args);
    
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    
    match command.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take().expect("Failed to capture stdout");
            let stderr = child.stderr.take().expect("Failed to capture stderr");
            
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();
            
            let mut stdout_output = String::new();
            let mut stderr_output = String::new();
            
            // Read stdout
            while let Ok(Some(line)) = stdout_reader.next_line().await {
                stdout_output.push_str(&line);
                stdout_output.push('\n');
            }
            
            // Read stderr
            while let Ok(Some(line)) = stderr_reader.next_line().await {
                stderr_output.push_str(&line);
                stderr_output.push('\n');
            }
            
            let status = child.wait().await?;
            
            Ok(ToolResult {
                tool_call_id: "exec".to_string(),
                success: status.success(),
                output: format!(
                    "stdout:\n{}\nstderr:\n{}",
                    stdout_output.trim(),
                    stderr_output.trim()
                ),
            })
        }
        Err(e) => Ok(ToolResult {
            tool_call_id: "exec".to_string(),
            success: false,
            output: format!("Failed to spawn process: {}", e),
        }),
    }
}

async fn execute_rust(code: &str) -> Result<ToolResult> {
    // Create a temporary Rust project
    let temp_dir = std::env::temp_dir().join(format!("spree_rust_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = r#"
[package]
name = "temp"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    std::fs::write(temp_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create src directory
    std::fs::create_dir_all(temp_dir.join("src"))?;
    
    // Create main.rs with the code wrapped in a main function if needed
    let main_content = if code.contains("fn main") {
        code.to_string()
    } else {
        format!("fn main() {{\n{}\n}}", code)
    };
    std::fs::write(temp_dir.join("src/main.rs"), main_content)?;
    
    // Compile and run
    let output = Command::new("cargo")
        .args(&["run", "--release"])
        .current_dir(&temp_dir)
        .output()
        .await?;
    
    // Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    Ok(ToolResult {
        tool_call_id: "exec_rust".to_string(),
        success: output.status.success(),
        output: format!(
            "stdout:\n{}\nstderr:\n{}",
            stdout.trim(),
            stderr.trim()
        ),
    })
}

/// Run a bash command with sandboxing and safety checks
pub async fn run_bash(
    command: &str,
    working_dir: Option<&str>,
    repo_root: &Path,
) -> Result<ToolResult> {
    debug!("Running bash command: {}", command);
    
    // Validate working directory
    let validated_dir = match validate_working_dir(working_dir, repo_root) {
        Ok(d) => d,
        Err(e) => {
            return Ok(ToolResult {
                tool_call_id: "bash".to_string(),
                success: false,
                output: format!("Sandbox error: {}", e),
            });
        }
    };
    
    // Check command safety
    let (is_safe, is_dangerous) = check_command_safety(command);
    
    if !is_safe {
        warn!("Blocked banned command: {}", command);
        return Ok(ToolResult {
            tool_call_id: "bash".to_string(),
            success: false,
            output: "Error: Command is banned for security reasons".to_string(),
        });
    }
    
    if is_dangerous {
        warn!("Command flagged as dangerous: {}", command);
        // We'll still execute but mark it in the output
    }
    
    // Execute with timeout (60 seconds) using tokio
    let child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .current_dir(&validated_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    
    let result = match child {
        Ok(mut child) => {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(60),
                child.wait()
            ).await {
                Ok(Ok(status)) => {
                    // Read stdout
                    let mut stdout = String::new();
                    if let Some(mut out) = child.stdout.take() {
                        use tokio::io::AsyncReadExt;
                        let mut buf = vec![];
                        let _ = tokio::io::AsyncReadExt::read_to_end(&mut out, &mut buf).await;
                        stdout = String::from_utf8_lossy(&buf).to_string();
                    }
                    
                    // Read stderr
                    let mut stderr = String::new();
                    if let Some(mut err) = child.stderr.take() {
                        use tokio::io::AsyncReadExt;
                        let mut buf = vec![];
                        let _ = tokio::io::AsyncReadExt::read_to_end(&mut err, &mut buf).await;
                        stderr = String::from_utf8_lossy(&buf).to_string();
                    }
                    
                    Ok((status.success(), stdout, stderr))
                }
                Ok(Err(e)) => Err(anyhow!("Process error: {}", e)),
                Err(_) => {
                    let _ = child.kill().await;
                    Err(anyhow!("Command timed out after 60 seconds"))
                }
            }
        }
        Err(e) => Err(anyhow!("Failed to spawn process: {}", e)),
    };
    
    match result {
        Ok((success, stdout, stderr)) => {
            // Cap output size (10KB limit)
            let max_len = 10_000;
            let stdout = if stdout.len() > max_len {
                format!("{}\n... (truncated)", &stdout[..max_len])
            } else {
                stdout
            };
            let stderr = if stderr.len() > max_len {
                format!("{}\n... (truncated)", &stderr[..max_len])
            } else {
                stderr
            };
            
            let status_msg = if success { 
                "Command succeeded" 
            } else { 
                "Command failed" 
            };
            let danger_msg = if is_dangerous { " [DANGEROUS COMMAND EXECUTED]" } else { "" };
            
            Ok(ToolResult {
                tool_call_id: "bash".to_string(),
                success,
                output: format!("{}{}\nstdout:\n{}\nstderr:\n{}", 
                    status_msg, danger_msg, stdout, stderr),
            })
        }
        Err(e) => Ok(ToolResult {
            tool_call_id: "bash".to_string(),
            success: false,
            output: format!("Failed to execute command: {}", e),
        }),
    }
}
