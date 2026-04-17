use super::{ToolCall, ToolResult};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Validate that a path is within the repository root
fn validate_path(path: &str, repo_root: &Path) -> Result<PathBuf> {
    let path = Path::new(path);

    // Convert to absolute path
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    };

    // Check for directory traversal
    let canonical = absolute.canonicalize().unwrap_or(absolute.clone());
    let canonical_root = repo_root.canonicalize().unwrap_or(repo_root.to_path_buf());

    if !canonical.starts_with(&canonical_root) {
        return Err(anyhow!(
            "Path '{}' is outside repository root",
            path.display()
        ));
    }

    Ok(absolute)
}

/// Check if path is sensitive
fn is_sensitive_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    let sensitive = [
        ".env",
        ".ssh",
        ".aws",
        ".docker",
        ".kube",
        "id_rsa",
        "id_ed25519",
        "id_dsa",
        ".htpasswd",
        "passwd",
        "shadow",
    ];

    sensitive.iter().any(|s| path_str.contains(s))
}

pub fn read_file(path: &str, repo_root: &Path) -> Result<ToolResult> {
    debug!("Reading file: {}", path);

    // Validate path is within repo
    let validated_path = match validate_path(path, repo_root) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ToolResult {
                tool_call_id: "read_file".to_string(),
                success: false,
                output: format!("Sandbox error: {}", e),
            });
        }
    };

    // Security check - prevent reading sensitive files
    if is_sensitive_path(&validated_path) {
        return Ok(ToolResult {
            tool_call_id: "read_file".to_string(),
            success: false,
            output: "Access denied: cannot read sensitive file".to_string(),
        });
    }

    match std::fs::read_to_string(&validated_path) {
        Ok(content) => Ok(ToolResult {
            tool_call_id: "read_file".to_string(),
            success: true,
            output: content,
        }),
        Err(e) => Ok(ToolResult {
            tool_call_id: "read_file".to_string(),
            success: false,
            output: format!("Failed to read file '{}': {}", path, e),
        }),
    }
}

pub fn write_file(path: &str, content: &str, append: bool, repo_root: &Path) -> Result<ToolResult> {
    debug!("Writing file: {} (append: {})", path, append);

    // Validate path is within repo
    let validated_path = match validate_path(path, repo_root) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ToolResult {
                tool_call_id: "write_file".to_string(),
                success: false,
                output: format!("Sandbox error: {}", e),
            });
        }
    };

    // Security check - prevent writing to sensitive paths
    if is_sensitive_path(&validated_path) {
        return Ok(ToolResult {
            tool_call_id: "write_file".to_string(),
            success: false,
            output: "Access denied: cannot write to sensitive path".to_string(),
        });
    }

    let result = if append {
        std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&validated_path)
            .and_then(|mut file| {
                use std::io::Write;
                file.write_all(content.as_bytes())
            })
    } else {
        std::fs::write(&validated_path, content)
    };

    match result {
        Ok(_) => Ok(ToolResult {
            tool_call_id: "write_file".to_string(),
            success: true,
            output: format!("File written successfully: {}", path),
        }),
        Err(e) => Ok(ToolResult {
            tool_call_id: "write_file".to_string(),
            success: false,
            output: format!("Failed to write file '{}': {}", path, e),
        }),
    }
}

pub fn list_directory(path: &str, recursive: bool, repo_root: &Path) -> Result<ToolResult> {
    debug!("Listing directory: {} (recursive: {})", path, recursive);

    // Validate path is within repo
    let validated_path = match validate_path(path, repo_root) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ToolResult {
                tool_call_id: "list_dir".to_string(),
                success: false,
                output: format!("Sandbox error: {}", e),
            });
        }
    };

    if !validated_path.exists() {
        return Ok(ToolResult {
            tool_call_id: "list_dir".to_string(),
            success: false,
            output: format!("Directory '{}' does not exist", path),
        });
    }

    let mut output = String::new();

    if recursive {
        list_recursive(&validated_path, &mut output, "", repo_root)?;
    } else {
        for entry in std::fs::read_dir(&validated_path)? {
            if let Ok(entry) = entry {
                let metadata = entry.metadata()?;
                let name = entry.file_name();
                let prefix = if metadata.is_dir() { "[DIR]" } else { "[FILE]" };
                output.push_str(&format!("{} {}\n", prefix, name.to_string_lossy()));
            }
        }
    }

    Ok(ToolResult {
        tool_call_id: "list_dir".to_string(),
        success: true,
        output,
    })
}

fn list_recursive(path: &Path, output: &mut String, prefix: &str, repo_root: &Path) -> Result<()> {
    // Validate we're still within repo
    if !path.starts_with(repo_root) {
        return Ok(());
    }

    for entry in std::fs::read_dir(path)? {
        if let Ok(entry) = entry {
            let metadata = entry.metadata()?;
            let name = entry.file_name();
            let full_path = entry.path();

            if metadata.is_dir() {
                output.push_str(&format!(
                    "{}[DIR]  {}{}\n",
                    prefix,
                    name.to_string_lossy(),
                    "/"
                ));
                let new_prefix = format!("{}  ", prefix);
                list_recursive(&full_path, output, &new_prefix, repo_root)?;
            } else {
                let size = metadata.len();
                output.push_str(&format!(
                    "{}[FILE] {} ({} bytes)\n",
                    prefix,
                    name.to_string_lossy(),
                    size
                ));
            }
        }
    }
    Ok(())
}

pub fn search_files(pattern: &str, path: &str, repo_root: &Path) -> Result<ToolResult> {
    debug!("Searching for '{}' in {}", pattern, path);

    // Validate path is within repo
    let validated_path = match validate_path(path, repo_root) {
        Ok(p) => p,
        Err(e) => {
            return Ok(ToolResult {
                tool_call_id: "search_files".to_string(),
                success: false,
                output: format!("Sandbox error: {}", e),
            });
        }
    };

    let mut results = Vec::new();

    for entry in walkdir::WalkDir::new(&validated_path) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() {
                let file_path = entry.path();

                // Skip sensitive files
                if is_sensitive_path(file_path) {
                    continue;
                }

                // Validate path is still within repo
                if !file_path.starts_with(repo_root) {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(file_path) {
                    if content.contains(pattern) {
                        results.push(file_path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    let output = if results.is_empty() {
        format!("No files found containing '{}'", pattern)
    } else {
        format!("Found {} files:\n{}", results.len(), results.join("\n"))
    };

    Ok(ToolResult {
        tool_call_id: "search_files".to_string(),
        success: true,
        output,
    })
}
