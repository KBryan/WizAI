//! Software Model
//!
//! Data structures representing analyzed software.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The complete model of a software project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareModel {
    pub name: String,
    pub language: Language,
    pub modules: Vec<Module>,
    pub public_apis: Vec<Api>,
    pub stateful_ops: Vec<Operation>,
    pub entry_points: Vec<EntryPoint>,
}

impl SoftwareModel {
    /// Generate CLI command groups from the model
    pub fn generate_command_groups(&self) -> Vec<CommandGroup> {
        let mut groups: std::collections::HashMap<String, Vec<Api>> =
            std::collections::HashMap::new();

        // Group APIs by module
        for api in &self.public_apis {
            let module_name = self
                .modules
                .iter()
                .find(|m| api.name.to_lowercase().starts_with(&m.name.to_lowercase()))
                .map(|m| m.name.clone())
                .unwrap_or_else(|| "general".to_string());

            groups
                .entry(module_name)
                .or_insert_with(Vec::new)
                .push(api.clone());
        }

        // Convert to command groups
        groups
            .into_iter()
            .map(|(name, apis)| CommandGroup {
                description: format!("Commands for {}", name),
                name: name.clone(),
                commands: apis,
            })
            .collect()
    }

    /// Get the main entry point (if any)
    pub fn main_entry_point(&self) -> Option<&EntryPoint> {
        self.entry_points.iter().find(|ep| ep.name == "main")
    }

    /// Check if the software has async operations
    pub fn has_async_operations(&self) -> bool {
        self.public_apis.iter().any(|api| api.is_async)
    }
}

/// Programming language of the software
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
}

/// A module in the software
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub path: PathBuf,
}

/// A public API (function or method)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api {
    pub name: String,
    pub params: Vec<(String, String)>, // (name, type)
    pub is_async: bool,
}

impl Api {
    /// Generate CLI argument definitions for this API
    pub fn generate_cli_args(&self) -> Vec<CliArg> {
        self.params
            .iter()
            .filter(|(name, _)| name != "&self" && name != "&mut self")
            .map(|(name, ty)| CliArg {
                name: name.clone(),
                arg_type: CliArgType::from_rust_type(ty),
                description: format!("The {}", name.replace('_', " ")),
                required: !ty.contains("Option"),
            })
            .collect()
    }
}

/// A stateful operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub kind: String,
}

/// An entry point (main, tool handler, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub name: String,
    pub kind: String,
}

/// A group of related CLI commands
#[derive(Debug, Clone)]
pub struct CommandGroup {
    pub name: String,
    pub description: String,
    pub commands: Vec<Api>,
}

/// CLI argument definition
#[derive(Debug, Clone)]
pub struct CliArg {
    pub name: String,
    pub arg_type: CliArgType,
    pub description: String,
    pub required: bool,
}

/// CLI argument type
#[derive(Debug, Clone)]
pub enum CliArgType {
    String,
    Integer,
    Float,
    Boolean,
    Path,
    Json,
}

impl CliArgType {
    /// Convert from Rust type to CLI arg type
    pub fn from_rust_type(rust_type: &str) -> Self {
        match rust_type {
            t if t.contains("String") || t.contains("str") => CliArgType::String,
            t if t.contains("i32") || t.contains("i64") || t.contains("usize") => {
                CliArgType::Integer
            }
            t if t.contains("f32") || t.contains("f64") => CliArgType::Float,
            t if t.contains("bool") => CliArgType::Boolean,
            t if t.contains("Path") => CliArgType::Path,
            t if t.contains("Value") || t.contains("Json") => CliArgType::Json,
            _ => CliArgType::String, // Default to string
        }
    }

    /// Get the Rust type for the generated code
    pub fn to_rust_type(&self) -> &'static str {
        match self {
            CliArgType::String => "String",
            CliArgType::Integer => "i64",
            CliArgType::Float => "f64",
            CliArgType::Boolean => "bool",
            CliArgType::Path => "PathBuf",
            CliArgType::Json => "serde_json::Value",
        }
    }

    /// Get the clap attribute for this type
    pub fn to_clap_attr(&self, name: &str) -> String {
        match self {
            CliArgType::String => format!("#[arg(short, long)]\n    pub {}: String", name),
            CliArgType::Integer => {
                format!("#[arg(short, long)]\n    pub {}: i64", name)
            }
            CliArgType::Float => format!("#[arg(short, long)]\n    pub {}: f64", name),
            CliArgType::Boolean => {
                format!("#[arg(short, long)]\n    pub {}: bool", name)
            }
            CliArgType::Path => {
                format!(
                    "#[arg(short, long, value_hint = ValueHint::FilePath)]\n    pub {}: PathBuf",
                    name
                )
            }
            CliArgType::Json => {
                format!(
                    "#[arg(short, long, value_parser = parse_json)]\n    pub {}: serde_json::Value",
                    name
                )
            }
        }
    }
}

/// Gap analysis for refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageAnalysis {
    pub version: String,
    pub total_apis: usize,
    pub covered_apis: usize,
    pub gaps: Vec<ApiGap>,
    pub suggestions: Vec<CommandSuggestion>,
}

impl CoverageAnalysis {
    pub fn coverage_percent(&self) -> f64 {
        if self.total_apis == 0 {
            0.0
        } else {
            (self.covered_apis as f64 / self.total_apis as f64) * 100.0
        }
    }
}

/// An API gap (missing CLI command)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGap {
    pub module: String,
    pub missing: Vec<String>,
    pub priority: String, // "high", "medium", "low"
}

/// A suggestion for a new command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command_name: String,
    pub module: String,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_arg_type_conversion() {
        assert!(matches!(
            CliArgType::from_rust_type("String"),
            CliArgType::String
        ));
        assert!(matches!(
            CliArgType::from_rust_type("i32"),
            CliArgType::Integer
        ));
        assert!(matches!(
            CliArgType::from_rust_type("bool"),
            CliArgType::Boolean
        ));
    }

    #[test]
    fn test_coverage_analysis() {
        let analysis = CoverageAnalysis {
            version: "1.0.0".to_string(),
            total_apis: 10,
            covered_apis: 7,
            gaps: vec![],
            suggestions: vec![],
        };

        assert_eq!(analysis.coverage_percent(), 70.0);
    }
}
