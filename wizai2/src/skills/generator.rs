use super::models::*;
use super::storage::SkillStorage;
use crate::llm::VeniceClient;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct SkillGenerator {
    storage: Arc<SkillStorage>,
    llm: Arc<VeniceClient>,
}

impl std::fmt::Debug for SkillGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillGenerator").finish()
    }
}

impl SkillGenerator {
    pub fn new(storage: Arc<SkillStorage>, llm: Arc<VeniceClient>) -> Self {
        Self { storage, llm }
    }

    /// Generate a new skill from a successful task completion
    pub async fn generate_from_task(&self, request: SkillGenerationRequest) -> Result<Option<Skill>> {
        // Only generate skills for successful tasks
        let quality_threshold = 0.7;
        let should_generate = match &request.outcome {
            TaskOutcome::Success { quality_score, .. } => *quality_score >= quality_threshold,
            TaskOutcome::PartialSuccess { .. } => false, // Don't generate from partial successes
            TaskOutcome::Failure { .. } => false,
        };

        if !should_generate {
            info!("Skipping skill generation - task did not meet quality threshold");
            return Ok(None);
        }

        info!("Generating skill from task: {}", request.task_description);

        // Check if a similar skill already exists
        if self.similar_skill_exists(&request.task_description).await? {
            info!("Similar skill already exists, skipping generation");
            return Ok(None);
        }

        // Use LLM to generate the skill
        let skill = self.llm_generate_skill(&request).await?;

        // Store the generation record
        let generation_id = self.storage.record_generation(&request, Some(&skill.id)).await?;
        debug!("Recorded skill generation: {}", generation_id);

        // Store the skill
        self.storage.store_skill(&skill).await?;

        info!("Generated new skill '{}' ({}) from task", skill.name, skill.id);
        
        Ok(Some(skill))
    }

    async fn llm_generate_skill(&self, request: &SkillGenerationRequest) -> Result<Skill> {
        let system_prompt = r#"You are an expert at creating reusable skills for AI agents. 
Your task is to analyze a completed task and create a skill that can be reused for similar tasks in the future.

The skill should:
1. Have a clear, specific name
2. Include a detailed description of what it does
3. Define input/output schemas
4. Include a prompt template with placeholders
5. List required tools
6. Include example inputs/outputs
7. Define trigger patterns

Return the skill as a JSON object matching the Skill format."#;

        let prompt = format!(
            r#"Generate a skill from this task completion:

Task Description: {}
Task Context: {}

Execution Trace:
{}

Outcome: {}

Create a skill that captures the pattern used to complete this task. The skill should be reusable for similar tasks."#,
            request.task_description,
            request.task_context,
            self.format_execution_trace(&request.execution_trace),
            self.format_outcome(&request.outcome)
        );

        let messages = vec![
            crate::llm::client::ChatMessage {
                role: "system".to_string(),
                content: Some(system_prompt.to_string()),
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
        
        // Parse the response to extract the skill
        let content = response.choices.first().map(|c| c.message.content.clone()).flatten().unwrap_or_default();
        let skill = self.parse_skill_from_response(&content, &request.agent_id).await?;
        
        Ok(skill)
    }

    async fn parse_skill_from_response(&self, response: &str, author: &str) -> Result<Skill> {
        // Try to extract JSON from the response
        let json_str = if response.contains("```json") {
            response.split("```json").nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else if response.contains("```") {
            response.split("```").nth(1)
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        // Try to parse the JSON response
        let skill_data: serde_json::Value = serde_json::from_str(json_str)?;
        
        let mut skill = Skill::default();
        skill.name = skill_data.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed_skill")
            .to_string();
        skill.description = skill_data.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();
        skill.author = author.to_string();
        
        // Parse category
        if let Some(cat) = skill_data.get("category").and_then(|v| v.as_str()) {
            skill.category = SkillCategory::from(cat.to_string());
        }
        
        // Parse tags
        if let Some(tags) = skill_data.get("tags").and_then(|v| v.as_array()) {
            skill.tags = tags.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        
        // Parse triggers
        if let Some(triggers) = skill_data.get("triggers").and_then(|v| v.as_array()) {
            skill.triggers = triggers.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        
        // Parse implementation
        if let Some(imp) = skill_data.get("implementation").or_else(|| skill_data.get("prompt_template")) {
            if let Some(template) = imp.as_str() {
                skill.implementation.prompt_template = template.to_string();
            } else if let Some(obj) = imp.as_object() {
                if let Some(template) = obj.get("prompt_template").and_then(|v| v.as_str()) {
                    skill.implementation.prompt_template = template.to_string();
                }
                if let Some(sys_prompt) = obj.get("system_prompt").and_then(|v| v.as_str()) {
                    skill.implementation.system_prompt = Some(sys_prompt.to_string());
                }
                if let Some(tools) = obj.get("required_tools").and_then(|v| v.as_array()) {
                    skill.implementation.required_tools = tools.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
                if let Some(skill_type) = obj.get("skill_type").and_then(|v| v.as_str()) {
                    skill.implementation.skill_type = match skill_type {
                        "generator" => SkillType::Generator,
                        "workflow" => SkillType::Workflow,
                        "analyzer" => SkillType::Analyzer,
                        "composite" => SkillType::Composite(Vec::new()),
                        _ => SkillType::Generator,
                    };
                }
            }
        }
        
        // Parse examples
        if let Some(examples) = skill_data.get("examples").and_then(|v| v.as_array()) {
            skill.implementation.examples = examples.iter()
                .filter_map(|ex| {
                    Some(SkillExample {
                        input: ex.get("input")?.clone(),
                        output: ex.get("output")?.clone(),
                        description: ex.get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Example")
                            .to_string(),
                    })
                })
                .collect();
        }
        
        // Parse input/output schemas
        if let Some(input_schema) = skill_data.get("input_schema") {
            skill.implementation.input_schema = input_schema.clone();
        }
        if let Some(output_schema) = skill_data.get("output_schema") {
            skill.implementation.output_schema = output_schema.clone();
        }
        
        // Parse constraints
        if let Some(constraints) = skill_data.get("constraints").and_then(|v| v.as_array()) {
            skill.implementation.constraints = constraints.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        
        Ok(skill)
    }

    fn format_execution_trace(&self, trace: &[ExecutionStep]) -> String {
        let mut formatted = String::new();
        for step in trace {
            formatted.push_str(&format!(
                "Step {}: {}\n  Input: {:?}\n  Output: {:?}\n  Duration: {}ms\n\n",
                step.step_number,
                step.action,
                step.input,
                step.output,
                step.duration_ms
            ));
        }
        formatted
    }

    fn format_outcome(&self, outcome: &TaskOutcome) -> String {
        match outcome {
            TaskOutcome::Success { quality_score, user_feedback } => {
                let mut s = format!("Success (quality: {})", quality_score);
                if let Some(feedback) = user_feedback {
                    s.push_str(&format!(", feedback: {}", feedback));
                }
                s
            }
            TaskOutcome::PartialSuccess { issues, user_feedback } => {
                let mut s = format!("Partial Success, issues: {:?}", issues);
                if let Some(feedback) = user_feedback {
                    s.push_str(&format!(", feedback: {}", feedback));
                }
                s
            }
            TaskOutcome::Failure { reason } => {
                format!("Failure: {}", reason)
            }
        }
    }

    async fn similar_skill_exists(&self, task_description: &str) -> Result<bool> {
        // Simple check: look for skills with similar keywords
        let words: Vec<&str> = task_description.split_whitespace().collect();
        
        for word in words.iter().filter(|w| w.len() > 4) {
            let existing = self.storage.search_skills(word).await?;
            if !existing.is_empty() {
                // Check if any existing skill has high usage and good success rate
                for skill in existing.iter().take(3) {
                    if skill.usage_count > 10 && skill.success_rate > 0.8 {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }

    /// Manually create a skill from user input
    pub async fn create_skill_manually(
        &self,
        name: &str,
        description: &str,
        prompt_template: &str,
        author: &str,
        category: SkillCategory,
    ) -> Result<Skill> {
        let mut skill = Skill::default();
        skill.name = name.to_string();
        skill.description = description.to_string();
        skill.author = author.to_string();
        skill.category = category;
        skill.implementation.prompt_template = prompt_template.to_string();
        skill.implementation.skill_type = SkillType::Generator;
        
        // Generate triggers from name and description
        skill.triggers = name.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        
        self.storage.store_skill(&skill).await?;
        
        info!("Manually created skill '{}' ({}) by {}", skill.name, skill.id, author);
        
        Ok(skill)
    }
}
