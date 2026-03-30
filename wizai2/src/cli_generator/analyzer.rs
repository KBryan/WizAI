//! Software Analyzer
//! 
//! Uses tree-sitter to parse source code and extract CLI-relevant information.

use crate::cli_generator::model::{Api, EntryPoint, Language, Module, Operation, SoftwareModel};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tree_sitter::{Node, Parser, Query, QueryCursor, Tree};
use walkdir::WalkDir;

/// Analyzes software to extract CLI-relevant structures
pub struct SoftwareAnalyzer {
    parsers: HashMap<Language, Parser>,
}

impl SoftwareAnalyzer {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        
        // Initialize Rust parser
        let mut parser = Parser::new();
        if parser.set_language(tree_sitter_rust::language()).is_ok() {
            parsers.insert(Language::Rust, parser);
        }
        
        // Initialize Python parser
        let mut parser = Parser::new();
        if parser.set_language(tree_sitter_python::language()).is_ok() {
            parsers.insert(Language::Python, parser);
        }
        
        // Initialize JavaScript parser
        let mut parser = Parser::new();
        if parser.set_language(tree_sitter_javascript::language()).is_ok() {
            parsers.insert(Language::JavaScript, parser);
        }
        
        Self { parsers }
    }
    
    /// Analyze software at the given path
    pub async fn analyze(&self, path: &Path) -> Result<SoftwareModel> {
        // Detect primary language
        let language = self.detect_language(path)?;
        
        // Find all source files
        let files = self.find_source_files(path, language)?;
        
        // Parse each file and extract information
        let mut modules = Vec::new();
        let mut public_apis = Vec::new();
        let mut stateful_ops = Vec::new();
        let mut entry_points = Vec::new();
        
        for file_path in files {
            if let Some(file_info) = self.analyze_file(&file_path, language).await? {
                modules.push(file_info.module);
                public_apis.extend(file_info.public_apis);
                stateful_ops.extend(file_info.stateful_ops);
                entry_points.extend(file_info.entry_points);
            }
        }
        
        Ok(SoftwareModel {
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            language,
            modules,
            public_apis,
            stateful_ops,
            entry_points,
        })
    }
    
    /// Detect the primary language of the codebase
    fn detect_language(&self, path: &Path) -> Result<Language> {
        let mut counts: HashMap<Language, usize> = HashMap::new();
        
        for entry in WalkDir::new(path).max_depth(3) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        let ext = ext.to_str().unwrap_or("");
                        match ext {
                            "rs" => *counts.entry(Language::Rust).or_insert(0) += 1,
                            "py" => *counts.entry(Language::Python).or_insert(0) += 1,
                            "js" | "ts" => *counts.entry(Language::JavaScript).or_insert(0) += 1,
                            _ => {}
                        }
                    }
                }
            }
        }
        
        counts.into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(lang, _)| lang)
            .ok_or_else(|| anyhow!("Could not detect language"))
    }
    
    /// Find all source files for the given language
    fn find_source_files(&self, path: &Path, language: Language) -> Result<Vec<PathBuf>> {
        let ext = match language {
            Language::Rust => "rs",
            Language::Python => "py",
            Language::JavaScript => "js",
        };
        
        let mut files = Vec::new();
        
        for entry in WalkDir::new(path) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Some(file_ext) = entry.path().extension() {
                        if file_ext == ext {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }
        
        Ok(files)
    }
    
    /// Analyze a single file
    async fn analyze_file(&self, path: &Path, language: Language) -> Result<Option<FileInfo>> {
        let source = tokio::fs::read_to_string(path).await?;
        
        // Create parser on demand to avoid mutable borrow issues
        let mut parser = Parser::new();
        let language_fn = match language {
            Language::Rust => tree_sitter_rust::language(),
            Language::Python => tree_sitter_python::language(),
            Language::JavaScript => tree_sitter_javascript::language(),
        };
        
        parser.set_language(language_fn)
            .map_err(|e| anyhow!("Failed to set language: {:?}", e))?;
        
        let tree = parser.parse(&source, None)
            .ok_or_else(|| anyhow!("Failed to parse file: {:?}", path))?;
        
        match language {
            Language::Rust => self.analyze_rust_file(path, &source, &tree).await,
            Language::Python => self.analyze_python_file(path, &source, &tree).await,
            Language::JavaScript => self.analyze_javascript_file(path, &source, &tree).await,
        }
    }
    
    /// Analyze a Rust file
    async fn analyze_rust_file(&self, path: &Path, source: &str, tree: &Tree) -> Result<Option<FileInfo>> {
        let mut public_apis = Vec::new();
        let mut stateful_ops = Vec::new();
        let mut entry_points = Vec::new();
        
        let root = tree.root_node();
        
        // Walk the AST and extract functions, structs, impl blocks
        self.walk_rust_nodes(&root, source, &mut public_apis, &mut stateful_ops, &mut entry_points)?;
        
        let module = Module {
            name: path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            path: path.to_path_buf(),
        };
        
        Ok(Some(FileInfo {
            module,
            public_apis,
            stateful_ops,
            entry_points,
        }))
    }
    
    /// Walk Rust AST nodes
    fn walk_rust_nodes(&self, node: &Node, source: &str, 
                      public_apis: &mut Vec<Api>,
                      stateful_ops: &mut Vec<Operation>,
                      entry_points: &mut Vec<EntryPoint>) -> Result<()> {
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            
            match child.kind() {
                "function_item" => {
                    if self.is_public(&child, source) {
                        let name = self.extract_function_name(&child, source)?;
                        let params = self.extract_params(&child, source)?;
                        
                        public_apis.push(Api {
                            name: name.clone(),
                            params,
                            is_async: self.is_async_function(&child, source),
                        });
                        
                        // Check if this is an entry point (like main or a tool handler)
                        if name == "main" || name.ends_with("_handler") {
                            entry_points.push(EntryPoint {
                                name: name.clone(),
                                kind: "function".to_string(),
                            });
                        }
                        
                        // Check if it modifies state
                        if self.modifies_state(&child, source) {
                            stateful_ops.push(Operation {
                                name,
                                kind: "stateful".to_string(),
                            });
                        }
                    }
                }
                "struct_item" => {
                    if self.is_public(&child, source) {
                        let name = self.extract_struct_name(&child, source)?;
                        entry_points.push(EntryPoint {
                            name,
                            kind: "struct".to_string(),
                        });
                    }
                }
                "impl_item" => {
                    // Extract methods from impl blocks
                    self.extract_impl_methods(&child, source, public_apis)?;
                }
                _ => {}
            }
            
            // Recursively walk children
            self.walk_rust_nodes(&child, source, public_apis, stateful_ops, entry_points)?;
        }
        
        Ok(())
    }
    
    /// Check if a node is public
    fn is_public(&self, node: &Node, source: &str) -> bool {
        // Check for `pub` visibility modifier
        let text = self.node_text(node, source);
        text.starts_with("pub ") || text.contains("\npub ")
    }
    
    /// Extract function name from AST
    fn extract_function_name(&self, node: &Node, source: &str) -> Result<String> {
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            if child.kind() == "identifier" {
                return Ok(self.node_text(&child, source).to_string());
            }
        }
        Err(anyhow!("Could not extract function name"))
    }
    
    /// Extract struct name from AST
    fn extract_struct_name(&self, node: &Node, source: &str) -> Result<String> {
        self.extract_function_name(node, source) // Same logic
    }
    
    /// Extract function parameters
    fn extract_params(&self, node: &Node, source: &str) -> Result<Vec<(String, String)>> {
        let mut params = Vec::new();
        
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            if child.kind() == "parameters" {
                for j in 0..child.child_count() {
                    let param = child.child(j).unwrap();
                    if param.kind() == "parameter" {
                        let param_text = self.node_text(&param, source);
                        // Parse "name: Type" format
                        if let Some(colon_pos) = param_text.find(':') {
                            let name = param_text[..colon_pos].trim().to_string();
                            let ty = param_text[colon_pos + 1..].trim().to_string();
                            params.push((name, ty));
                        }
                    }
                }
            }
        }
        
        Ok(params)
    }
    
    /// Check if function is async
    fn is_async_function(&self, node: &Node, source: &str) -> bool {
        let text = self.node_text(node, source);
        text.contains("async fn")
    }
    
    /// Check if function modifies state
    fn modifies_state(&self, node: &Node, source: &str) -> bool {
        let text = self.node_text(node, source);
        // Heuristic: look for mutable references or state changes
        text.contains("&mut") || 
        text.contains("self.") || 
        text.contains("write") ||
        text.contains("save")
    }
    
    /// Extract methods from impl blocks
    fn extract_impl_methods(&self, node: &Node, source: &str, 
                           public_apis: &mut Vec<Api>) -> Result<()> {
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            if child.kind() == "function_item" && self.is_public(&child, source) {
                if let Ok(name) = self.extract_function_name(&child, source) {
                    let params = self.extract_params(&child, source)?;
                    public_apis.push(Api {
                        name,
                        params,
                        is_async: self.is_async_function(&child, source),
                    });
                }
            }
        }
        Ok(())
    }
    
    /// Get text of a node
    fn node_text(&self, node: &Node, source: &str) -> String {
        source[node.start_byte()..node.end_byte()].to_string()
    }
    
    /// Analyze a Python file (placeholder)
    async fn analyze_python_file(&self, path: &Path, source: &str, tree: &Tree) -> Result<Option<FileInfo>> {
        // TODO: Implement Python analysis
        Ok(None)
    }
    
    /// Analyze a JavaScript file (placeholder)
    async fn analyze_javascript_file(&self, path: &Path, source: &str, tree: &Tree) -> Result<Option<FileInfo>> {
        // TODO: Implement JavaScript analysis
        Ok(None)
    }
}

/// Information extracted from a single file
struct FileInfo {
    module: Module,
    public_apis: Vec<Api>,
    stateful_ops: Vec<Operation>,
    entry_points: Vec<EntryPoint>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_analyzer_creation() {
        let analyzer = SoftwareAnalyzer::new();
        // Just verify it compiles
    }
}