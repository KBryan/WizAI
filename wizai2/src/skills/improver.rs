use super::models::*;
use super::storage::SkillStorage;
use crate::llm::VeniceClient;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct SkillImprover {
    storage: Arc<SkillStorage>,
    llm: Arc<VeniceClient>,
}

impl SkillImprover {
    pub fn new(storage: Arc<SkillStorage>, llm: Arc<VeniceClient>) -> Self {
        Self { storage, llm }
    }

    /// Analyze skill usage and suggest improvements
    pub async fn analyze_and_improve(&self, skill_id: &str) -> Result<Option<Skill>> {
        let skill = match self.storage.get_skill(skill_id).await? {
            Some(s) => s,
            None => return Err(anyhow!("Skill '{}' not found", skill_id)),
        };

        // Only improve skills with sufficient usage data
        if skill.usage_count < 5 {
            debug!("Skill {} has insufficient usage ({} < 5), skipping improvement", skill_id, skill.usage_count);
            return Ok(None);
        }

        // Check success rate
        if skill.success_rate >= 0.95 {
            debug!("Skill {} has high success rate ({}), no improvement needed", skill_id, skill.success_rate);
            return Ok(None);
        }

        info!("Analyzing skill '{}' for improvement (success rate: {}, usage: {})", 
            skill.name, skill.success_rate, skill.usage_count);

        // Get execution history
        let executions = self.get_recent_executions(skill_id, 20).await?;
        
        // Analyze failures
        let failures: Vec<_> = executions.iter()
            .filter(|e| !e.success)
            .collect();

        if failures.is_empty() {
            debug!("No failures found for skill {}, skipping improvement", skill_id);
            return Ok(None);
        }

        // Generate improved version
        let improved_skill = self.generate_improved_version(&skill, &executions).await?;
        
        // Store the improved version as a new version
        self.storage.store_skill(&improved_skill).await?;

        info!("Created improved version of skill '{}' ({} -> {})", 
            skill.name, skill.version, improved_skill.version);

        Ok(Some(improved_skill))
    }

    async fn get_recent_executions(&self, skill_id: &str, limit: i64) -> Result<Vec<SkillExecutionRecord>> {
        // This would query from skill_executions table
        // For now, return empty - in full implementation, add query method to storage
        Ok(Vec::new())
    }

    async fn generate_improved_version(
        &self,
        skill: &Skill,
        executions: &[SkillExecutionRecord],
    ) -> Result<Skill> {
        let system_prompt = r#"You are an expert at improving AI agent skills.
Analyze the skill and its execution history to suggest improvements.
Focus on:
1. Better prompt templates
2. More accurate triggers
3. Improved examples
4. Better error handling

Return an improved version of the skill with version bumped."#;

        let executions_summary = self.summarize_executions(executions);

        let prompt = format!(
            r#"Improve this skill:

Name: {}
Description: {}
Current Success Rate: {}
Usage Count: {}

Current Prompt Template:
{}

Current Triggers: {:?}

Recent Execution Summary:
{}

Provide an improved version of this skill."#,
            skill.name,
            skill.description,
            skill.success_rate,
            skill.usage_count,
            skill.implementation.prompt_template,
            skill.triggers,
            executions_summary
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
        
        // Parse and create improved skill
        let mut improved = skill.clone();
        
        // Increment version
        improved.version = self.bump_version(&skill.version);
        improved.updated_at = chrono::Utc::now();
        
        let content = response.choices.first().map(|c| c.message.content.clone()).flatten().unwrap_or_default();
        
        // Parse improvements from response
        if let Some(template) = self.extract_section(&content, "prompt_template") {
            improved.implementation.prompt_template = template;
        }
        
        if let Some(description) = self.extract_section(&content, "description") {
            improved.description = description;
        }
        
        // Extract new triggers
        if let Some(triggers_str) = self.extract_section(&content, "triggers") {
            improved.triggers = triggers_str
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        Ok(improved)
    }

    fn summarize_executions(&self, executions: &[SkillExecutionRecord]) -> String {
        let total = executions.len();
        let successful = executions.iter().filter(|e| e.success).count();
        let failed = total - successful;
        
        let mut summary = format!(
            "Total executions: {}\nSuccessful: {}\nFailed: {}\n\n",
            total, successful, failed
        );

        // Add details about failures
        if failed > 0 {
            summary.push_str("Recent failures:\n");
            for exec in executions.iter().filter(|e| !e.success).take(5) {
                summary.push_str(&format!("- Input: {:?}\n", exec.input));
                if let Some(output) = &exec.output {
                    summary.push_str(&format!("  Error: {}\n", output));
                }
            }
        }

        summary
    }

    fn bump_version(&self, current_version: &str) -> String {
        // Parse semantic version and bump patch
        let parts: Vec<&str> = current_version.split('.').collect();
        if parts.len() >= 3 {
            if let Ok(patch) = parts[2].parse::<u32>() {
                return format!("{}.{}.{}", parts[0], parts[1], patch + 1);
            }
        }
        
        // Default bump
        format!("{}-improved", current_version)
    }

    fn extract_section(&self, response: &str, section_name: &str) -> Option<String> {
        // Look for section markers like "## Prompt Template" or "prompt_template:"
        let markers = vec![
            format!("## {}", section_name.replace("_", " ")),
            format!("**{}**", section_name.replace("_", " ")),
            format!("{}:", section_name),
            format!("{}", section_name),
        ];

        for marker in markers {
            if let Some(start) = response.find(&marker) {
                let after_marker = &response[start + marker.len()..];
                let end = after_marker.find("\n\n")
                    .or_else(|| after_marker.find("##"))
                    .unwrap_or(after_marker.len());
                let content = &after_marker[..end].trim();
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }

        None
    }

    /// Run improvement analysis on all skills
    pub async fn improve_all_skills(&self) -> Result<Vec<String>> {
        let skills = self.storage.list_skills(None).await?;
        let mut improved_ids = Vec::new();

        for skill in skills {
            if let Some(improved) = self.analyze_and_improve(&skill.id).await? {
                improved_ids.push(improved.id);
            }
        }

        if !improved_ids.is_empty() {
            info!("Improved {} skills", improved_ids.len());
        }

        Ok(improved_ids)
    }
}

#[derive(Debug, Clone)]
struct SkillExecutionRecord {
    success: bool,
    input: serde_json::Value,
    output: Option<serde_json::Value>,
    execution_time_ms: i64,
}
