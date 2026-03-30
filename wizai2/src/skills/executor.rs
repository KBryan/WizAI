use super::models::*;
use super::storage::SkillStorage;
use crate::llm::VeniceClient;
use crate::tools::{ToolRegistry, ToolContext, ToolCall, ToolResult};
use anyhow::{anyhow, Result};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct SkillExecutor {
    storage: Arc<SkillStorage>,
    llm: Arc<VeniceClient>,
    tools: Arc<RwLock<ToolRegistry>>,
}

impl std::fmt::Debug for SkillExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillExecutor").finish()
    }
}

impl SkillExecutor {
    pub fn new(
        storage: Arc<SkillStorage>,
        llm: Arc<VeniceClient>,
        tools: Arc<RwLock<ToolRegistry>>,
    ) -> Self {
        Self {
            storage,
            llm,
            tools,
        }
    }

    /// Execute a skill with the given input
    pub async fn execute(&self, skill_id: &str, input: serde_json::Value, context: SkillContext) -> Result<SkillExecutionResult> {
        let start_time = std::time::Instant::now();
        
        // Load the skill
        let skill = match self.storage.get_skill(skill_id).await? {
            Some(s) => s,
            None => return Err(anyhow!("Skill '{}' not found", skill_id)),
        };

        info!("Executing skill '{}' ({})", skill.name, skill_id);
        debug!("Skill input: {:?}", input);

        // Execute based on skill type
        let result = match &skill.implementation.skill_type {
            SkillType::Generator => self.execute_generator_skill(&skill, &input, &context).await,
            SkillType::Workflow => self.execute_workflow_skill(&skill, &input, &context).await,
            SkillType::Analyzer => self.execute_analyzer_skill(&skill, &input, &context).await,
            SkillType::Composite(sub_skill_ids) => self.execute_composite_skill(&skill, sub_skill_ids, &input, &context).await,
        };

        let execution_time_ms = start_time.elapsed().as_millis() as i64;

        // Record the execution
        let success = result.is_ok();
        let output = result.as_ref().map_err(|e| {
            error!("Skill execution failed: {}", e);
            e
        }).ok().map(|r| r.clone());

        self.storage.record_execution(
            skill_id,
            &context.agent_id,
            &input,
            output.as_ref(),
            success,
            execution_time_ms,
        ).await?;

        match result {
            Ok(output) => {
                info!("Skill '{}' executed successfully in {}ms", skill.name, execution_time_ms);
                Ok(SkillExecutionResult {
                    skill_id: skill_id.to_string(),
                    success: true,
                    output,
                    execution_time_ms,
                    tool_calls: Vec::new(), // TODO: Track tool calls
                    metadata: std::collections::HashMap::new(),
                })
            }
            Err(e) => {
                warn!("Skill '{}' execution failed: {}", skill.name, e);
                Ok(SkillExecutionResult {
                    skill_id: skill_id.to_string(),
                    success: false,
                    output: serde_json::json!({"error": e.to_string()}),
                    execution_time_ms,
                    tool_calls: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                })
            }
        }
    }

    async fn execute_generator_skill(
        &self,
        skill: &Skill,
        input: &serde_json::Value,
        context: &SkillContext,
    ) -> Result<serde_json::Value> {
        // Prepare the prompt
        let prompt = self.prepare_prompt(&skill.implementation.prompt_template, input)?;
        
        // Call LLM
        let messages = vec![
            crate::llm::client::ChatMessage {
                role: "system".to_string(),
                content: Some(skill.implementation.system_prompt.clone().unwrap_or_default()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            crate::llm::client::ChatMessage {
                role: "user".to_string(),
                content: Some(prompt),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];

        let chat_request = crate::llm::client::ChatRequest {
            model: "default".to_string(),
            messages,
            temperature: 0.7,
            stream: None,
            tools: None,
            tool_choice: None,
        };
        let response = self.llm.chat(chat_request).await?;
        
        let content = response.choices.first().map(|c| c.message.content.clone()).flatten().unwrap_or_default();
        Ok(serde_json::json!({
            "output": content
        }))
    }

    async fn execute_workflow_skill(
        &self,
        skill: &Skill,
        input: &serde_json::Value,
        context: &SkillContext,
    ) -> Result<serde_json::Value> {
        // Workflow skills execute a sequence of tool calls
        // Parse the prompt template to determine the workflow
        let prompt = self.prepare_prompt(&skill.implementation.prompt_template, input)?;
        
        // For workflow skills, the prompt template should describe the steps
        // We'll use the LLM to interpret and execute them
        let system_prompt = format!(
            "You are executing a workflow skill: {}. {}",
            skill.name,
            skill.implementation.system_prompt.clone().unwrap_or_default()
        );

        let messages = vec![
            crate::llm::client::ChatMessage {
                role: "system".to_string(),
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            crate::llm::client::ChatMessage {
                role: "user".to_string(),
                content: Some(prompt),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];

        let request = crate::llm::client::ChatRequest {
            model: "default".to_string(),
            messages,
            temperature: 0.7,
            stream: None,
            tools: None,
            tool_choice: None,
        };
        let response = self.llm.chat(request).await?;
        
        let content = response.choices.first().map(|c| c.message.content.clone()).flatten().unwrap_or_default();
        Ok(serde_json::json!({
            "output": content,
            "workflow_completed": true
        }))
    }

    async fn execute_analyzer_skill(
        &self,
        skill: &Skill,
        input: &serde_json::Value,
        context: &SkillContext,
    ) -> Result<serde_json::Value> {
        // Analyzer skills process input and return structured analysis
        let prompt = self.prepare_prompt(&skill.implementation.prompt_template, input)?;
        
        let system_prompt = format!(
            "You are an analyzer skill: {}. {}",
            skill.name,
            skill.implementation.system_prompt.clone().unwrap_or_default()
        );

        let messages = vec![
            crate::llm::client::ChatMessage {
                role: "system".to_string(),
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            crate::llm::client::ChatMessage {
                role: "user".to_string(),
                content: Some(prompt),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];

        let request = crate::llm::client::ChatRequest {
            model: "default".to_string(),
            messages,
            temperature: 0.7,
            stream: None,
            tools: None,
            tool_choice: None,
        };
        let response = self.llm.chat(request).await?;
        
        // Try to parse as JSON, fallback to text
        let content = response.choices.first().map(|c| c.message.content.clone()).flatten().unwrap_or_default();
        let output = match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => json,
            Err(_) => serde_json::json!({
                "analysis": content
            }),
        };
        
        Ok(output)
    }

    fn execute_composite_skill<'a>(
        &'a self,
        _skill: &'a Skill,
        sub_skill_ids: &'a [String],
        input: &'a serde_json::Value,
        context: &'a SkillContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send + 'a>> {
        Box::pin(async move {
            // Composite skills execute multiple sub-skills and combine results
            let mut results = Vec::new();
            
            for sub_skill_id in sub_skill_ids {
                match self.execute(sub_skill_id, input.clone(), context.clone()).await {
                    Ok(result) => {
                        if result.success {
                            results.push(result.output);
                        }
                    }
                    Err(e) => {
                        warn!("Sub-skill {} failed: {}", sub_skill_id, e);
                    }
                }
            }
            
            Ok(serde_json::json!({
                "sub_results": results,
                "combined_count": results.len()
            }))
        })
    }

    fn prepare_prompt(&self, template: &str, input: &serde_json::Value) -> Result<String> {
        // Simple template replacement
        let mut prompt = template.to_string();
        
        // Replace {{input}} with the full input
        let input_str = serde_json::to_string_pretty(input)?;
        prompt = prompt.replace("{{input}}", &input_str);
        
        // Replace {{input.<field>}} with specific fields
        if let serde_json::Value::Object(map) = input {
            for (key, value) in map {
                let placeholder = format!("{{{{input.{}}}}}", key);
                let value_str = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                prompt = prompt.replace(&placeholder, &value_str);
            }
        }
        
        Ok(prompt)
    }

    /// Find skills that might be relevant for a given task
    pub async fn find_relevant_skills(&self, task_description: &str) -> Result<Vec<Skill>> {
        // Search by keywords in task description
        let keywords: Vec<&str> = task_description.split_whitespace().collect();
        
        let mut all_skills = Vec::new();
        for keyword in keywords {
            if keyword.len() > 3 { // Only search for meaningful keywords
                let skills = self.storage.search_skills(keyword).await?;
                all_skills.extend(skills);
            }
        }
        
        // Deduplicate and sort by relevance (usage count as proxy)
        let mut unique_skills: Vec<Skill> = all_skills
            .into_iter()
            .fold(Vec::new(), |mut acc, skill| {
                if !acc.iter().any(|s| s.id == skill.id) {
                    acc.push(skill);
                }
                acc
            });
        
        unique_skills.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        unique_skills.truncate(5); // Return top 5
        
        Ok(unique_skills)
    }

    /// Check if a skill should be triggered based on task description
    pub async fn should_trigger_skill(&self, skill: &Skill, task_description: &str) -> (bool, f64) {
        let task_lower = task_description.to_lowercase();
        
        for trigger in &skill.triggers {
            let trigger_lower = trigger.to_lowercase();
            
            if task_lower.contains(&trigger_lower) {
                let confidence = 0.8; // Simple keyword match
                return (true, confidence);
            }
        }
        
        (false, 0.0)
    }
}
