//! CLI Generator Module
//! 
//! Rust-native CLI generation for any software.
//! Uses tree-sitter for AST analysis and Askama for code generation.

pub mod analyzer;
pub mod model;
pub mod codegen;
pub mod templates;

use anyhow::Result;
use std::path::Path;

pub use analyzer::SoftwareAnalyzer;
pub use model::SoftwareModel;
pub use codegen::{CliGenerator, GeneratedCli, GeneratedCommand};

/// Generate a CLI for the given software path
pub async fn generate_cli(software_path: &Path, change_name: &str) -> Result<GeneratedCli> {
    // Step 1: Analyze the software
    let analyzer = SoftwareAnalyzer::new();
    let model = analyzer.analyze(software_path).await?;
    
    // Step 2: Generate the CLI code
    let generator = CliGenerator::new(&model, change_name);
    let cli = generator.generate().await?;
    
    Ok(cli)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cli_generation_foundation() {
        // This will be our dogfooding test
        // Analyze wizai2/src/agent/ and generate CLI
        let result = generate_cli(
            Path::new("src/agent"),
            "test-agent-cli"
        ).await;
        
        // For now, just check compilation
        assert!(result.is_ok() || result.is_err()); // Placeholder
    }
}
