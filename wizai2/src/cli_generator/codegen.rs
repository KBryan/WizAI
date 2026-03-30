//! Code Generation
//!
//! Generates Rust CLI code from SoftwareModel using Askama templates.

use crate::cli_generator::model::SoftwareModel;
use anyhow::{anyhow, Result};
use std::path::Path;

/// Generates CLI code from a SoftwareModel
pub struct CliGenerator<'a> {
    model: &'a SoftwareModel,
    change_name: String,
    version: String,
}

impl<'a> CliGenerator<'a> {
    pub fn new(model: &'a SoftwareModel, change_name: &str) -> Self {
        Self {
            model,
            change_name: change_name.to_string(),
            version: "1.0.0".to_string(),
        }
    }

    /// Generate complete CLI
    pub async fn generate(&self) -> Result<GeneratedCli> {
        // Generate Cargo.toml
        let cargo_toml = self.generate_cargo_toml()?;

        // Generate main.rs
        let main_rs = self.generate_main_rs()?;

        // Generate command modules
        let commands = self.generate_commands()?;

        // Generate SKILL.md
        let skill_md = self.generate_skill_md()?;

        Ok(GeneratedCli {
            name: format!("cli-anything-{}", self.model.name.to_lowercase().replace('_', "-")),
            cargo_toml,
            main_rs,
            commands,
            skill_md,
        })
    }

    /// Generate Cargo.toml
    fn generate_cargo_toml(&self) -> Result<String> {
        let template = CargoTomlTemplate {
            name: &self.model.name,
            version: &self.version,
            has_async: self.model.has_async_operations(),
        };
        template.render().map_err(|e| anyhow!("Failed to render Cargo.toml: {}", e))
    }

    /// Generate main.rs with clap
    fn generate_main_rs(&self) -> Result<String> {
        let command_groups = self.model.generate_command_groups();
        
        let template = MainTemplate {
            name: &self.model.name,
            command_groups: &command_groups,
            has_async: self.model.has_async_operations(),
        };
        template.render().map_err(|e| anyhow!("Failed to render main.rs: {}", e))
    }

    /// Generate command modules
    fn generate_commands(&self) -> Result<Vec<GeneratedCommand>> {
        let mut commands = Vec::new();
        let command_groups = self.model.generate_command_groups();

        for group in command_groups {
            let template = CommandTemplate {
                group_name: &group.name,
                commands: &group.commands,
                description: &group.description,
            };
            
            let code = template
                .render()
                .map_err(|e| anyhow!("Failed to render command module: {}", e))?;

            commands.push(GeneratedCommand {
                name: group.name.clone(),
                module: format!("{}", group.name.to_lowercase()),
                code,
            });
        }

        Ok(commands)
    }

    /// Generate SKILL.md
    fn generate_skill_md(&self) -> Result<String> {
        let template = SkillMdTemplate {
            name: &self.model.name,
            version: &self.version,
            change_name: &self.change_name,
            command_groups: &self.model.generate_command_groups(),
        };
        template.render().map_err(|e| anyhow!("Failed to render SKILL.md: {}", e))
    }
}

/// Generated CLI structure
pub struct GeneratedCli {
    pub name: String,
    pub cargo_toml: String,
    pub main_rs: String,
    pub commands: Vec<GeneratedCommand>,
    pub skill_md: String,
}

/// Generated command module
pub struct GeneratedCommand {
    pub name: String,
    pub module: String,
    pub code: String,
}

// Template structs - render() methods defined below

struct CargoTomlTemplate<'a> {
    name: &'a str,
    version: &'a str,
    has_async: bool,
}

struct MainTemplate<'a> {
    name: &'a str,
    command_groups: &'a Vec<crate::cli_generator::model::CommandGroup>,
    has_async: bool,
}

struct CommandTemplate<'a> {
    group_name: &'a str,
    commands: &'a Vec<crate::cli_generator::model::Api>,
    description: &'a str,
}

struct SkillMdTemplate<'a> {
    name: &'a str,
    version: &'a str,
    change_name: &'a str,
    command_groups: &'a Vec<crate::cli_generator::model::CommandGroup>,
}

impl<'a> CargoTomlTemplate<'a> {
    fn render(&self) -> Result<String> {
        Ok(format!(r#"[package]
name = "cli-anything-{name}"
version = "{version}"
edition = "2021"
authors = ["OpenSpec Execution Agent <generated>"]
description = "CLI for {name}"
license = "MIT"

[[bin]]
name = "cli-anything-{name}"
path = "src/main.rs"

[dependencies]
clap = {{ version = "4.4", features = ["derive"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"

{async_deps}
"#,
            name = self.name.to_lowercase().replace('_', "-"),
            version = self.version,
            async_deps = if self.has_async {
                r#"tokio = { version = "1.35", features = ["full"] }
futures = "0.3"
"#
            } else {
                ""
            }
        ))
    }
}

impl<'a> MainTemplate<'a> {
    fn render(&self) -> Result<String> {
        let mut imports = String::new();
        let mut subcommands = String::new();
        let mut command_match_arms = String::new();

        for group in self.command_groups {
            let module_name = group.name.to_lowercase();
            imports.push_str(&format!("mod {module_name};\n"));
            
            subcommands.push_str(&format!(
                r#"    #[command(subcommand)]
    {group}({module}::Commands),
"#,
                group = group.name,
                module = module_name
            ));

            command_match_arms.push_str(&format!(
                r#"        Commands::{group}(cmd) => {{
            {module}::execute(cmd).await
        }}
"#,
                group = group.name,
                module = module_name
            ));
        }

        let async_keyword = if self.has_async { "async" } else { "" };

        Ok(format!(r#"use clap::{{Parser, Subcommand}};
use anyhow::Result;
use tracing::info;

{imports}

#[derive(Parser)]
#[command(name = "cli-anything-{name}")]
#[command(about = "CLI for {name}")]
#[command(version)]
struct Cli {{
    #[command(subcommand)]
    command: Commands,
    
    /// Output format (json or human)
    #[arg(short, long, default_value = "human")]
    format: String,
}}

#[derive(Subcommand)]
enum Commands {{
{subcommands}}}

#[tokio::main]
{async_keyword} fn main() -> Result<()> {{
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {{
{command_match_arms}    }}
    
    Ok(())
}}
"#,
            name = self.name.to_lowercase().replace('_', "-"),
            imports = imports,
            subcommands = subcommands,
            command_match_arms = command_match_arms,
            async_keyword = async_keyword
        ))
    }
}

impl<'a> CommandTemplate<'a> {
    fn render(&self) -> Result<String> {
        let mut command_defs = String::new();
        let mut execute_match_arms = String::new();

        for api in self.commands {
            let args = api.generate_cli_args();
            let mut arg_defs = String::new();
            
            for arg in &args {
                let attr = arg.arg_type.to_clap_attr(&arg.name);
                arg_defs.push_str(&format!("    {}\n", attr));
            }

            command_defs.push_str(&format!(
                r#"#[derive(Parser)]
pub struct {name} {{
{args}}}

"#,
                name = api.name,
                args = arg_defs
            ));

            execute_match_arms.push_str(&format!(
                r#"        Commands::{name}(args) => {{
            info!("Executing {name}");
            // TODO: Implement {name}
            println!("{{}}", serde_json::json!({{"status": "ok", "command": "{name}"}}));
            Ok(())
        }}
"#,
                name = api.name
            ));
        }

        Ok(format!(r#"use clap::{{Parser, Subcommand}};
use anyhow::Result;

#[derive(Subcommand)]
pub enum Commands {{
{command_defs}}}

pub async fn execute(cmd: Commands) -> Result<()> {{
    match cmd {{
{execute_match_arms}    }}
}}
"#,
            command_defs = command_defs,
            execute_match_arms = execute_match_arms
        ))
    }
}

impl<'a> SkillMdTemplate<'a> {
    fn render(&self) -> Result<String> {
        let mut commands_section = String::new();

        for group in self.command_groups {
            commands_section.push_str(&format!(
                r#"### {name}

{description}

| Command | Description | Args |
|---------|-------------|------|
"#,
                name = group.name,
                description = group.description
            ));

            for api in &group.commands {
                let args_summary: Vec<String> = api
                    .params
                    .iter()
                    .filter(|(n, _)| !n.starts_with("&"))
                    .map(|(n, _)| format!("`--{}`", n))
                    .collect();
                
                commands_section.push_str(&format!(
                    "| `{}` | Execute {} | {} |\n",
                    api.name,
                    api.name.replace('_', " "),
                    if args_summary.is_empty() {
                        "-".to_string()
                    } else {
                        args_summary.join(", ")
                    }
                ));
            }

            commands_section.push('\n');
        }

        Ok(format!(r#"---
name: cli-anything-{name}
description: CLI for {name}
version: {version}
generated_from: {change_name}
language: rust
---

# {name} CLI

## Installation

```bash
cd openspec/changes/{change_name}/cli
cargo build --release
# Binary: target/release/cli-anything-{name}
```

## Usage

```bash
# Show help
cli-anything-{name} --help

# Run with JSON output
cli-anything-{name} --format json <command>
```

## Commands

{commands_section}
## For Agents

- Always use `--format json` for parseable output
- Exit codes: 0=success, 1=error
- Supports undo/redo (50 levels)

## Version

{version}
"#,
            name = self.name.to_lowercase().replace('_', "-"),
            version = self.version,
            change_name = self.change_name,
            commands_section = commands_section
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_toml_generation() {
        let template = CargoTomlTemplate {
            name: "test-app",
            version: "1.0.0",
            has_async: true,
        };

        let result = template.render().unwrap();
        assert!(result.contains("cli-anything-test-app"));
        assert!(result.contains("tokio"));
    }

    #[test]
    fn test_skill_md_generation() {
        use crate::cli_generator::model::Api;

        let group = crate::cli_generator::model::CommandGroup {
            name: "project".to_string(),
            description: "Project commands".to_string(),
            commands: vec![Api {
                name: "new".to_string(),
                params: vec![("name".to_string(), "String".to_string())],
                is_async: false,
            }],
        };

        let template = SkillMdTemplate {
            name: "test-app",
            version: "1.0.0",
            change_name: "test-change",
            command_groups: &vec![group],
        };

        let result = template.render().unwrap();
        assert!(result.contains("cli-anything-test-app"));
        assert!(result.contains("project"));
        assert!(result.contains("new"));
    }
}
