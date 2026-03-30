//! Templates
//!
//! Askama templates for CLI code generation.
//! Templates are defined inline in codegen.rs for simplicity.
//! This module provides template utilities and helpers.

use crate::cli_generator::model::{CliArg, CliArgType};

/// Escape string for use in generated code
pub fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Convert a Rust type name to CLI-friendly name
pub fn to_cli_name(name: &str) -> String {
    name.to_lowercase().replace('_', "-").replace("::", "-")
}

/// Generate doc comment for generated code
pub fn generate_doc_comment(description: &str) -> String {
    description
        .lines()
        .map(|line| format!("/// {}", line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate clap argument attributes
pub fn generate_arg_attributes(arg: &CliArg) -> String {
    let mut attrs = Vec::new();

    // Short flag
    attrs.push(format!(
        "short = '{}'",
        arg.name.chars().next().unwrap_or('a')
    ));

    // Long flag
    attrs.push(format!("long"));

    // Help text
    attrs.push(format!("help = \"{}\"", escape_string(&arg.description)));

    // Value hint for paths
    if matches!(arg.arg_type, CliArgType::Path) {
        attrs.push("value_hint = ValueHint::FilePath".to_string());
    }

    // Default value (for optional args)
    if !arg.required {
        attrs.push("default_value = \"\"".to_string());
    }

    format!("#[arg({})]", attrs.join(", "))
}

/// Generate example usage for a command
pub fn generate_example(command_name: &str, args: &[CliArg]) -> String {
    let arg_examples: Vec<String> = args
        .iter()
        .map(|arg| format!("--{} <value>", arg.name))
        .collect();

    if arg_examples.is_empty() {
        format!("cli-anything-<software> {}", command_name)
    } else {
        format!(
            "cli-anything-<software> {} {}",
            command_name,
            arg_examples.join(" ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_string("say \"hi\""), "say \\\"hi\\\"");
    }

    #[test]
    fn test_to_cli_name() {
        assert_eq!(to_cli_name("MyStruct"), "mystruct");
        assert_eq!(to_cli_name("my_function"), "my-function");
        assert_eq!(to_cli_name("a::b::c"), "a-b-c");
    }

    #[test]
    fn test_generate_doc_comment() {
        let comment = generate_doc_comment("This is\na multi-line\ndescription");
        assert!(comment.contains("/// This is"));
        assert!(comment.contains("/// a multi-line"));
        assert!(comment.contains("/// description"));
    }
}
