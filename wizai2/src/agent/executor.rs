use super::core::*;
use crate::agent::honest_mode::HonestModeConfig;
use crate::llm::{ChatMessage, ChatRequest, VeniceClient};
use crate::tools::{ToolCall, ToolContext, ToolRegistry, ToolResult};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AgentExecutor {
    agent_id: AgentId,
    registry: Arc<RwLock<AgentRegistry>>,
    event_sender: mpsc::Sender<AgentEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task: String,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub response: String,
    pub actions_taken: Vec<String>,
}

impl AgentExecutor {
    pub async fn new(agent_id: AgentId, registry: Arc<RwLock<AgentRegistry>>) -> Result<Self> {
        let (event_sender, _) = mpsc::channel(100);
        
        Ok(Self {
            agent_id,
            registry,
            event_sender,
        })
    }

    pub async fn execute_task(&self, task: TaskRequest) -> Result<TaskResult> {
        info!("Agent {:?} executing task: {}", self.agent_id, task.task);
        
        // Add user message
        let user_message = Message {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: task.task.clone(),
            timestamp: Utc::now(),
            metadata: None,
        };
        
        self.add_message(user_message).await?;
        
        // Update status
        self.set_status(AgentStatus::Thinking).await?;
        
        // Get agent context
        let agent = self.get_agent().await?;
        let agent_guard = agent.read().await;
        let system_prompt = agent_guard.context.system_prompt.clone();
        let messages = agent_guard.context.messages.clone();
        drop(agent_guard);
        
        // Apply honest mode configuration
        let honest_config = HonestModeConfig::default();
        let enhanced_prompt = honest_config.get_full_prompt(&system_prompt);
        
        // Convert to LLM messages
        let mut chat_messages = self.convert_to_chat_messages(&messages)?;
        
        // Prepend system message with enhanced prompt
        let system_message = ChatMessage {
            role: "system".to_string(),
            content: Some(enhanced_prompt),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };
        chat_messages.insert(0, system_message);
        
        // Get LLM client
        let llm = self.registry.read().await.get_llm().await;
        
        // Check if we need to delegate
        let should_delegate = self.should_delegate_to_subordinate(&task.task).await?;
        
        let response = if should_delegate {
            info!("Delegating task to subordinate");
            self.delegate_task(&task.task).await?
        } else {
            // Get tools from registry
            let tools = self.registry.read().await.get_tools().await;
            let tool_defs = tools.read().await.get_definitions();
            let tool_definitions: Vec<crate::llm::ToolDefinition> = tool_defs.into_iter().map(|td| {
                crate::llm::ToolDefinition {
                    name: td.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: td.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    parameters: td.get("parameters").cloned().unwrap_or(serde_json::json!({})),
                }
            }).collect();
            
            // Determine model and temperature based on honest mode
            // For data/research queries, we might want uncensored model
            // For tool-heavy tasks, we need kimi-k2-5 for tool support
            let needs_tools = !tool_definitions.is_empty();
            let is_research_query = task.task.to_lowercase().contains("research") 
                || task.task.to_lowercase().contains("market")
                || task.task.to_lowercase().contains("analysis");
            
            let (model, temperature) = if needs_tools {
                // Use kimi-k2-5 for tool support but with honest mode prompt
                ("kimi-k2-5".to_string(), honest_config.temperature)
            } else if is_research_query && honest_config.enabled {
                // Use uncensored model for research queries when honest mode is on
                (honest_config.model.clone(), honest_config.temperature)
            } else {
                // Default balanced approach
                ("kimi-k2-5".to_string(), 0.7_f32)
            };
            
            info!("Using model: {} with temperature: {} (honest mode: {})", 
                  model, temperature, honest_config.enabled);
            
            // Call LLM with tools (using kimi-k2-5 for tool support)
            let request = ChatRequest {
                model,
                messages: chat_messages,
                temperature,
                stream: Some(true),
                tools: if tool_definitions.is_empty() { None } else { Some(tool_definitions) },
                tool_choice: Some(crate::llm::ToolChoice::Auto),
            };
            
            let mut full_response = String::new();
            let mut stream_error = None;
            
            match llm.chat_stream(request).await {
                Ok(mut stream) => {
                    while let Some(chunk) = stream.recv().await {
                        match chunk {
                            Ok(text) => {
                                full_response.push_str(&text);
                            }
                            Err(e) => {
                                error!("LLM stream error: {}", e);
                                stream_error = Some(e.to_string());
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("LLM error: {}", e);
                    return Err(e);
                }
            }
            
            // If we got a stream error, report it
            if let Some(err) = stream_error {
                return Err(anyhow!("LLM stream error: {}", err));
            }
            
            // If response is empty, report an error
            if full_response.is_empty() {
                return Err(anyhow!("LLM returned empty response"));
            }
            
            full_response
        };
        
        // Make response mutable for tool result appending
        let mut response = response;
        
        // Check for and execute tool calls
        let tool_calls = crate::tools::ToolRegistry::parse_tool_calls(&response);
        let mut tool_results = Vec::new();
        
        if !tool_calls.is_empty() {
            info!("Detected {} tool calls in response", tool_calls.len());
            
            // Create tool context with current directory as repo root
            let repo_root = std::env::current_dir().unwrap_or_default();
            let tool_ctx = ToolContext::new(repo_root);
            
            // Execute each tool call
            let tools = self.registry.read().await.get_tools().await;
            for tool_call in tool_calls {
                info!("Executing tool: {}", tool_call.name);
                self.set_status(AgentStatus::ExecutingTool(tool_call.name.clone())).await?;
                
                match tools.read().await.execute(&tool_ctx, &tool_call).await {
                    Ok(result) => {
                        tool_results.push(format!("{}: {}", tool_call.name, 
                            if result.success { "✓" } else { "✗" }));
                        
                        // Append tool result to response
                        response.push_str(&format!("\n\n[Tool {} result]: {}", 
                            tool_call.name, result.output));
                    }
                    Err(e) => {
                        error!("Tool execution failed: {}", e);
                        response.push_str(&format!("\n\n[Tool {} error]: {}", 
                            tool_call.name, e));
                    }
                }
            }
        }
        
        // Add assistant message
        let assistant_message = Message {
            id: Uuid::new_v4(),
            role: MessageRole::Assistant,
            content: response.clone(),
            timestamp: Utc::now(),
            metadata: None,
        };
        
        self.add_message(assistant_message).await?;
        
        // Update status
        self.set_status(AgentStatus::Idle).await?;
        
        Ok(TaskResult {
            success: true,
            response,
            actions_taken: tool_results,
        })
    }

    async fn should_delegate_to_subordinate(&self, task: &str) -> Result<bool> {
        let agent = self.get_agent().await?;
        let agent_guard = agent.read().await;
        
        // Check if agent has subordinates
        if agent_guard.subordinates.is_empty() {
            return Ok(false);
        }
        
        // Simple heuristic: delegate if task mentions subordinates or is complex
        let lower_task = task.to_lowercase();
        let delegate_keywords = [
            "delegate", "assign", "team", "help with", "assist with",
            "specialist", "expert", "detailed", "complex",
        ];
        
        Ok(delegate_keywords.iter().any(|kw| lower_task.contains(kw)))
    }

    async fn delegate_task(&self, task: &str) -> Result<String> {
        let agent = self.get_agent().await?;
        
        // Get subordinate ID while holding the lock
        let subordinate_id = {
            let agent_guard = agent.read().await;
            agent_guard.subordinates.first().cloned()
                .ok_or_else(|| anyhow!("No subordinates available"))?
        };
        
        // Create delegation message
        let delegation_message = format!(
            "I am delegating this task to you from {}:\n\n{}",
            self.agent_id.0,
            task
        );
        
        // Use boxed future to avoid recursion
        let registry = self.registry.clone();
        let task_request = TaskRequest {
            task: delegation_message,
            context: None,
        };
        
        let result: Result<TaskResult> = Box::pin(async move {
            let sub_executor = AgentExecutor::new(subordinate_id, registry).await?;
            sub_executor.execute_task(task_request).await
        }).await;
        
        match result {
            Ok(result) => {
                info!("Subordinate completed task successfully");
                Ok(format!(
                    "I delegated this task to a team member. Here's their response:\n\n{}",
                    result.response
                ))
            }
            Err(e) => {
                warn!("Subordinate failed: {}", e);
                Ok(format!(
                    "I attempted to delegate this but encountered an issue: {}. I'll handle it myself.",
                    e
                ))
            }
        }
    }

    fn convert_to_chat_messages(&self, messages: &[Message]) -> Result<Vec<ChatMessage>> {
        messages
            .iter()
            .map(|m| {
                let role = match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                    MessageRole::Tool => "tool",
                };
                
                Ok(ChatMessage {
                    role: role.to_string(),
                    content: Some(m.content.clone()),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                })
            })
            .collect()
    }

    async fn get_agent(&self) -> Result<Arc<RwLock<Agent>>> {
        self.registry
            .read()
            .await
            .get_agent(self.agent_id)
            .ok_or_else(|| anyhow!("Agent not found"))
    }

    async fn add_message(&self, message: Message) -> Result<()> {
        let mut registry = self.registry.write().await;
        registry.add_message(self.agent_id, message).await
    }

    async fn set_status(&self, status: AgentStatus) -> Result<()> {
        // First update the agent status
        if let Some(agent) = self.registry.read().await.get_agent(self.agent_id) {
            agent.write().await.status = status.clone();
        }
        
        // Then broadcast the event
        let registry = self.registry.read().await;
        registry
            .broadcast_event(AgentEvent::StatusChanged {
                agent_id: self.agent_id,
                status,
            })
            .await;
        
        Ok(())
    }

    pub async fn execute_tools(&self, tool_calls: Vec<ToolCall>) -> Result<Vec<ToolResult>> {
        let tools = self.registry.read().await.get_tools().await;
        let tools_guard = tools.read().await;
        
        // Create tool context
        let ctx = crate::tools::ToolContext::new(std::env::current_dir()?);
        
        let mut results = Vec::new();
        
        for tool_call in tool_calls {
            self.set_status(AgentStatus::ExecutingTool(tool_call.name.clone())).await?;
            
            match tools_guard.execute(&ctx, &tool_call).await {
                Ok(result) => {
                    results.push(result.clone());
                    
                    let mut registry = self.registry.write().await;
                    registry
                        .broadcast_event(AgentEvent::ToolExecuted {
                            agent_id: self.agent_id,
                            tool_call,
                            result,
                        })
                        .await;
                }
                Err(e) => {
                    error!("Tool execution failed: {}", e);
                    results.push(ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        success: false,
                        output: format!("Error: {}", e),
                    });
                }
            }
        }
        
        Ok(results)
    }
}
