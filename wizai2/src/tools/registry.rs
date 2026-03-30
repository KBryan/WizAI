use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::future::Future;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Tool execution context with repository and session information
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub repo_root: PathBuf,
    pub cwd: PathBuf,
    pub session_id: String,
    pub active_change: Option<String>,
    pub loaded_agents_md: Option<String>,
    pub loaded_skills: Vec<String>,
    pub approval_mode: ApprovalMode,
}

impl ToolContext {
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            cwd: std::env::current_dir().unwrap_or_default(),
            session_id: Uuid::new_v4().to_string(),
            active_change: None,
            loaded_agents_md: None,
            loaded_skills: Vec::new(),
            approval_mode: ApprovalMode::Ask,
        }
    }

    pub fn with_approval_mode(mut self, mode: ApprovalMode) -> Self {
        self.approval_mode = mode;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ApprovalMode {
    Auto,
    Ask,
    DenyDangerous,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub success: bool,
    pub output: String,
}

// Type alias for async tool handlers
pub type ToolHandler = Box<
    dyn Fn(&ToolContext, &ToolCall) -> Pin<Box<dyn Future<Output = Result<ToolResult>> + Send>>
        + Send
        + Sync,
>;

pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub handler: ToolHandler,
    pub requires_approval: bool,
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("parameters", &self.parameters)
            .finish()
    }
}

impl Clone for Tool {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
            requires_approval: self.requires_approval,
            handler: Box::new(|_, _| Box::pin(async move {
                Err(anyhow!("Cloned tool handler not implemented"))
            })),
        }
    }
}

#[derive(Debug)]
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        
        // Register default tools
        registry.register_default_tools();
        
        registry
    }
    
    fn register_default_tools(&mut self) {
        use crate::tools::file_system;
        use crate::tools::code_exec;

        // Read file tool - uses sandboxed file_system module
        self.register(Tool {
            name: "read_file".to_string(),
            description: "Read the contents of a file within the repository".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read (relative to repo root)"
                    }
                },
                "required": ["path"]
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let path = call.arguments["path"].as_str()
                    .ok_or_else(|| anyhow!("Missing path argument"))
                    .unwrap_or("")
                    .to_string();
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match file_system::read_file(&path, &repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error reading file: {}", e),
                        }),
                    }
                })
            }),
        });
        
        // Write file tool - uses sandboxed file_system module
        self.register(Tool {
            name: "write_file".to_string(),
            description: "Write content to a file within the repository".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file (relative to repo root)"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let path = call.arguments["path"].as_str()
                    .ok_or_else(|| anyhow!("Missing path argument"))
                    .unwrap_or("")
                    .to_string();
                let content = call.arguments["content"].as_str()
                    .ok_or_else(|| anyhow!("Missing content argument"))
                    .unwrap_or("")
                    .to_string();
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match file_system::write_file(&path, &content, false, &repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error writing file: {}", e),
                        }),
                    }
                })
            }),
        });
        
        // List directory tool - uses sandboxed file_system module
        self.register(Tool {
            name: "list_dir".to_string(),
            description: "List contents of a directory within the repository".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path (relative to repo root)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "List recursively",
                        "default": false
                    }
                },
                "required": ["path"]
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let path = call.arguments["path"].as_str()
                    .ok_or_else(|| anyhow!("Missing path argument"))
                    .unwrap_or("")
                    .to_string();
                let recursive = call.arguments.get("recursive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match file_system::list_directory(&path, recursive, &repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error listing directory: {}", e),
                        }),
                    }
                })
            }),
        });

        // Search files tool - uses sandboxed file_system module  
        self.register(Tool {
            name: "grep_code".to_string(),
            description: "Search for a pattern in files within the repository".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Directory to search in (relative to repo root)",
                        "default": "."
                    }
                },
                "required": ["pattern"]
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let pattern = call.arguments["pattern"].as_str()
                    .ok_or_else(|| anyhow!("Missing pattern argument"))
                    .unwrap_or("")
                    .to_string();
                let path = call.arguments.get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".")
                    .to_string();
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match file_system::search_files(&pattern, &path, &repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error searching files: {}", e),
                        }),
                    }
                })
            }),
        });
        
        // Execute bash command - requires approval
        self.register(Tool {
            name: "bash".to_string(),
            description: "Execute a bash command in the repository".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Bash command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Working directory (relative to repo root, optional)"
                    }
                },
                "required": ["command"]
            }),
            requires_approval: true,
            handler: Box::new(|ctx, call| {
                let command = call.arguments["command"].as_str()
                    .ok_or_else(|| anyhow!("Missing command argument"))
                    .unwrap_or("")
                    .to_string();
                let working_dir = call.arguments.get("working_dir")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match code_exec::run_bash(&command, working_dir.as_deref(), &repo_root).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error executing command: {}", e),
                        }),
                    }
                })
            }),
        });
        
        // Search memory tool
        self.register(Tool {
            name: "search_memory".to_string(),
            description: "Search agent's memory for relevant information".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
            requires_approval: false,
            handler: Box::new(|_ctx, call| {
                let query = call.arguments["query"].as_str()
                    .ok_or_else(|| anyhow!("Missing query argument"))
                    .unwrap_or("")
                    .to_string();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    // This is a placeholder - actual implementation needs access to memory store
                    Ok(ToolResult {
                        tool_call_id: call_id,
                        success: true,
                        output: format!("Memory search for '{}' would return results here", query),
                    })
                })
            }),
        });
        
        // Delegate to subordinate tool
        self.register(Tool {
            name: "delegate".to_string(),
            description: "Delegate a task to a subordinate agent".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task": {
                        "type": "string",
                        "description": "Task to delegate"
                    },
                    "subordinate_role": {
                        "type": "string",
                        "description": "Role of subordinate to use (optional)"
                    }
                },
                "required": ["task"]
            }),
            requires_approval: false,
            handler: Box::new(|_ctx, call| {
                let task = call.arguments["task"].as_str()
                    .ok_or_else(|| anyhow!("Missing task argument"))
                    .unwrap_or("")
                    .to_string();
                let role = call.arguments.get("subordinate_role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Specialist")
                    .to_string();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    // This is a placeholder - actual implementation needs access to agent registry
                    Ok(ToolResult {
                        tool_call_id: call_id,
                        success: true,
                        output: format!("Delegating task to {}: {}", role, task),
                    })
                })
            }),
        });
        
        // CLI Generation tool
        self.register(Tool {
            name: "generate_cli".to_string(),
            description: "Generate a CLI from software source code".to_string(),
            parameters: crate::tools::generate_cli::schema(),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let ctx = ctx.clone();
                let call = call.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match crate::tools::generate_cli::execute(&ctx, &call).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error generating CLI: {}", e),
                        }),
                    }
                })
            }),
        });

        // OpenSpec workflow tools
        self.register(Tool {
            name: "detect_openspec_project".to_string(),
            description: "Detect if this is an OpenSpec project and return project info".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match crate::tools::openspec::detect_openspec_project(&repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error detecting OpenSpec project: {}", e),
                        }),
                    }
                })
            }),
        });

        self.register(Tool {
            name: "list_changes".to_string(),
            description: "List all changes in the openspec/changes/ directory".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_archived": {
                        "type": "boolean",
                        "description": "Include archived changes",
                        "default": false
                    }
                },
                "required": []
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let include_archived = call.arguments.get("include_archived")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match crate::tools::openspec::list_changes(&repo_root, include_archived) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error listing changes: {}", e),
                        }),
                    }
                })
            }),
        });

        self.register(Tool {
            name: "create_change_scaffold".to_string(),
            description: "Create a new change scaffold with directories and initial structure".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "change_name": {
                        "type": "string",
                        "description": "Name of the change to create"
                    }
                },
                "required": ["change_name"]
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let change_name = call.arguments["change_name"].as_str()
                    .ok_or_else(|| anyhow!("Missing change_name argument"))
                    .unwrap_or("")
                    .to_string();
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match crate::tools::openspec::create_change_scaffold(&change_name, &repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error creating change scaffold: {}", e),
                        }),
                    }
                })
            }),
        });

        self.register(Tool {
            name: "read_change_artifacts".to_string(),
            description: "Read artifacts from a change directory (proposal.md, design.md, tasks.md)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "change_name": {
                        "type": "string",
                        "description": "Name of the change to read"
                    }
                },
                "required": ["change_name"]
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let change_name = call.arguments["change_name"].as_str()
                    .ok_or_else(|| anyhow!("Missing change_name argument"))
                    .unwrap_or("")
                    .to_string();
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match crate::tools::openspec::read_change_artifacts(&change_name, &repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error reading change artifacts: {}", e),
                        }),
                    }
                })
            }),
        });

        self.register(Tool {
            name: "archive_change".to_string(),
            description: "Safely archive a completed change".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "change_name": {
                        "type": "string",
                        "description": "Name of the change to archive"
                    }
                },
                "required": ["change_name"]
            }),
            requires_approval: false,
            handler: Box::new(|ctx, call| {
                let change_name = call.arguments["change_name"].as_str()
                    .ok_or_else(|| anyhow!("Missing change_name argument"))
                    .unwrap_or("")
                    .to_string();
                let repo_root = ctx.repo_root.clone();
                let call_id = call.id.clone();
                
                Box::pin(async move {
                    match crate::tools::openspec::archive_change(&change_name, &repo_root) {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error archiving change: {}", e),
                        }),
                    }
                })
            }),
        });
        
        info!("Registered {} default tools", self.tools.len());
    }
    
    pub fn register(&mut self, tool: Tool) {
        debug!("Registering tool: {}", tool.name);
        self.tools.insert(tool.name.clone(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }
    
    pub fn list(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }
    
    pub fn get_definitions(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.parameters
                })
            })
            .collect()
    }
    
    pub async fn execute(&self, ctx: &ToolContext, tool_call: &ToolCall) -> Result<ToolResult> {
        let tool = self.tools.get(&tool_call.name)
            .ok_or_else(|| anyhow!("Tool '{}' not found", tool_call.name))?;
        
        // Check approval requirements
        if tool.requires_approval && ctx.approval_mode == ApprovalMode::DenyDangerous {
            return Ok(ToolResult {
                tool_call_id: tool_call.id.clone(),
                success: false,
                output: format!("Tool '{}' requires approval but mode is DenyDangerous", tool_call.name),
            });
        }
        
        debug!("Executing tool: {}", tool_call.name);
        
        // Execute async handler
        let result = (tool.handler)(ctx, tool_call).await;
        
        match &result {
            Ok(res) => {
                if res.success {
                    info!("Tool {} executed successfully", tool_call.name);
                } else {
                    warn!("Tool {} failed: {}", tool_call.name, res.output);
                }
            }
            Err(e) => {
                error!("Tool {} error: {}", tool_call.name, e);
            }
        }
        
        result
    }
    
    pub fn parse_tool_calls(content: &str) -> Vec<ToolCall> {
        // Parse tool calls from LLM response
        // Expected format: ```tool\n{"name": "...", "arguments": {...}}\n```
        let mut tool_calls = Vec::new();
        
        if let Some(start) = content.find("```tool") {
            if let Some(end) = content[start..].find("```") {
                let tool_block = &content[start + 7..start + end].trim();
                
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(tool_block) {
                    if let Some(name) = json["name"].as_str() {
                        tool_calls.push(ToolCall {
                            id: Uuid::new_v4().to_string(),
                            name: name.to_string(),
                            arguments: json["arguments"].clone(),
                        });
                    }
                }
            }
        }
        
        tool_calls
    }
    
    /// Register skill management tools
    pub fn register_skill_tools(
        &mut self,
        skill_storage: std::sync::Arc<crate::skills::SkillStorage>,
        skill_executor: std::sync::Arc<crate::skills::SkillExecutor>,
    ) {
        use crate::tools::skills;

        // Execute skill tool
        let skill_storage_exec = skill_storage.clone();
        let skill_executor_exec = skill_executor.clone();
        self.register(Tool {
            name: "execute_skill".to_string(),
            description: "Execute a skill by name or ID with the given input".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_identifier": {
                        "type": "string",
                        "description": "Name or ID of the skill to execute"
                    },
                    "input": {
                        "type": "object",
                        "description": "Input parameters for the skill"
                    },
                    "agent_id": {
                        "type": "string",
                        "description": "ID of the agent executing the skill"
                    }
                },
                "required": ["skill_identifier", "input", "agent_id"]
            }),
            requires_approval: false,
            handler: Box::new(move |_ctx, call| {
                let skill_storage = skill_storage_exec.clone();
                let skill_executor = skill_executor_exec.clone();
                let skill_identifier = call.arguments["skill_identifier"].as_str()
                    .unwrap_or("")
                    .to_string();
                let input = call.arguments["input"].clone();
                let agent_id = call.arguments["agent_id"].as_str()
                    .unwrap_or("")
                    .to_string();
                let call_id = call.id.clone();

                Box::pin(async move {
                    match skills::execute_skill(skill_storage, skill_executor, &skill_identifier, input, &agent_id).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error executing skill: {}", e),
                        }),
                    }
                })
            }),
        });

        // List skills tool
        let skill_storage_list = skill_storage.clone();
        self.register(Tool {
            name: "list_skills".to_string(),
            description: "List available skills, optionally filtered by category".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "description": "Filter by category (optional)"
                    }
                },
                "required": []
            }),
            requires_approval: false,
            handler: Box::new(move |_ctx, call| {
                let skill_storage = skill_storage_list.clone();
                let category = call.arguments.get("category")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let call_id = call.id.clone();

                Box::pin(async move {
                    let category_ref = category.as_deref();
                    match skills::list_skills(skill_storage, category_ref).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error listing skills: {}", e),
                        }),
                    }
                })
            }),
        });

        // Search skills tool
        let skill_storage_search = skill_storage.clone();
        self.register(Tool {
            name: "search_skills".to_string(),
            description: "Search for skills by query string".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    }
                },
                "required": ["query"]
            }),
            requires_approval: false,
            handler: Box::new(move |_ctx, call| {
                let skill_storage = skill_storage_search.clone();
                let query = call.arguments["query"].as_str()
                    .unwrap_or("")
                    .to_string();
                let call_id = call.id.clone();

                Box::pin(async move {
                    match skills::search_skills(skill_storage, &query).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error searching skills: {}", e),
                        }),
                    }
                })
            }),
        });

        // Get skill info tool
        let skill_storage_info = skill_storage.clone();
        self.register(Tool {
            name: "get_skill_info".to_string(),
            description: "Get detailed information about a specific skill".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_id": {
                        "type": "string",
                        "description": "ID or name of the skill"
                    }
                },
                "required": ["skill_id"]
            }),
            requires_approval: false,
            handler: Box::new(move |_ctx, call| {
                let skill_storage = skill_storage_info.clone();
                let skill_id = call.arguments["skill_id"].as_str()
                    .unwrap_or("")
                    .to_string();
                let call_id = call.id.clone();

                Box::pin(async move {
                    match skills::get_skill_info(skill_storage, &skill_id).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error getting skill info: {}", e),
                        }),
                    }
                })
            }),
        });

        // Create skill tool
        let skill_storage_create = skill_storage.clone();
        self.register(Tool {
            name: "create_skill".to_string(),
            description: "Create a new skill manually".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the skill"
                    },
                    "description": {
                        "type": "string",
                        "description": "Description of what the skill does"
                    },
                    "prompt_template": {
                        "type": "string",
                        "description": "The prompt template with {{input}} placeholders"
                    },
                    "author": {
                        "type": "string",
                        "description": "Author of the skill"
                    },
                    "category": {
                        "type": "string",
                        "description": "Category of the skill (optional)"
                    }
                },
                "required": ["name", "description", "prompt_template", "author"]
            }),
            requires_approval: false,
            handler: Box::new(move |_ctx, call| {
                let skill_storage = skill_storage_create.clone();
                let name = call.arguments["name"].as_str().unwrap_or("").to_string();
                let description = call.arguments["description"].as_str().unwrap_or("").to_string();
                let prompt_template = call.arguments["prompt_template"].as_str().unwrap_or("").to_string();
                let author = call.arguments["author"].as_str().unwrap_or("").to_string();
                let category = call.arguments.get("category").and_then(|v| v.as_str()).map(|s| s.to_string());
                let call_id = call.id.clone();

                Box::pin(async move {
                    let category_ref = category.as_deref();
                    match skills::create_skill(skill_storage, &name, &description, &prompt_template, &author, category_ref).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error creating skill: {}", e),
                        }),
                    }
                })
            }),
        });

        // Suggest skills tool
        let skill_executor_suggest = skill_executor.clone();
        self.register(Tool {
            name: "suggest_skills".to_string(),
            description: "Find and suggest skills for a given task".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_description": {
                        "type": "string",
                        "description": "Description of the task to find skills for"
                    }
                },
                "required": ["task_description"]
            }),
            requires_approval: false,
            handler: Box::new(move |_ctx, call| {
                let skill_executor = skill_executor_suggest.clone();
                let task_description = call.arguments["task_description"].as_str()
                    .unwrap_or("")
                    .to_string();
                let call_id = call.id.clone();

                Box::pin(async move {
                    match skills::suggest_skills(skill_executor, &task_description).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error suggesting skills: {}", e),
                        }),
                    }
                })
            }),
        });

        // Get skill stats tool
        let skill_storage_stats = skill_storage.clone();
        self.register(Tool {
            name: "get_skill_stats".to_string(),
            description: "Get statistics about skill usage across the system".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            requires_approval: false,
            handler: Box::new(move |_ctx, call| {
                let skill_storage = skill_storage_stats.clone();
                let call_id = call.id.clone();

                Box::pin(async move {
                    match skills::get_skill_stats(skill_storage).await {
                        Ok(result) => Ok(result),
                        Err(e) => Ok(ToolResult {
                            tool_call_id: call_id,
                            success: false,
                            output: format!("Error getting skill stats: {}", e),
                        }),
                    }
                })
            }),
        });

        info!("Registered skill management tools");
    }

    /// Load MCP tools from MCP client
    pub async fn load_mcp_tools(&mut self, mcp_client: &crate::mcp::MCPClient) -> Result<()> {
        use crate::mcp::protocol::MCPContent;
        
        let servers = mcp_client.list_servers().await;
        
        for server_name in servers {
            let tools = mcp_client.get_server_tools(&server_name).await?;
            
            for mcp_tool in tools {
                // Convert MCP tool to Spree tool
                let tool_name = format!("{}.{}", server_name, mcp_tool.name);
                let tool_name_for_handler = tool_name.clone();
                
                let spree_tool = Tool {
                    name: tool_name.clone(),
                    description: mcp_tool.description.clone(),
                    parameters: mcp_tool.input_schema.clone(),
                    requires_approval: false, // Could be configured per tool
                    handler: Box::new(move |_ctx, call| {
                        let tool_name = tool_name_for_handler.clone();
                        let call_id = call.id.clone();
                        let args = call.arguments.clone();
                        
                        Box::pin(async move {
                            // For now, return a placeholder - actual MCP calls should go through the client
                            Ok(ToolResult {
                                tool_call_id: call_id,
                                success: true,
                                output: format!("MCP tool {} called with {:?}", tool_name, args),
                            })
                        })
                    }),
                };
                
                self.register(spree_tool);
                info!("Registered MCP tool: {}", tool_name);
            }
        }
        
        Ok(())
    }
}