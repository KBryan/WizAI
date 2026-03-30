use crate::skills::{Skill, SkillCategory, SkillExecutionInput, SkillExecutionResult, SkillContext};
use crate::tools::registry::{ToolContext, ToolCall, ToolResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execute a skill by name or ID
pub async fn execute_skill(
    skill_storage: Arc<crate::skills::SkillStorage>,
    skill_executor: Arc<crate::skills::SkillExecutor>,
    skill_identifier: &str,
    input: serde_json::Value,
    agent_id: &str,
) -> Result<ToolResult> {
    // Try to find skill by ID first, then by name
    let skill = if let Some(skill) = skill_storage.get_skill(skill_identifier).await? {
        skill
    } else if let Some(skill) = skill_storage.get_skill_by_name(skill_identifier).await? {
        skill
    } else {
        return Ok(ToolResult {
            tool_call_id: "execute_skill".to_string(),
            success: false,
            output: format!("Skill '{}' not found", skill_identifier),
        });
    };

    // Execute the skill
    let context = SkillContext {
        agent_id: agent_id.to_string(),
        conversation_id: None,
        user_preferences: std::collections::HashMap::new(),
        relevant_memories: Vec::new(),
    };

    let result = skill_executor.execute(&skill.id, input, context).await?;

    Ok(ToolResult {
        tool_call_id: skill_identifier.to_string(),
        success: result.success,
        output: if result.success {
            serde_json::to_string_pretty(&result.output)?
        } else {
            format!("Skill execution failed: {:?}", result.output)
        },
    })
}

/// List available skills, optionally filtered by category
pub async fn list_skills(
    skill_storage: Arc<crate::skills::SkillStorage>,
    category: Option<&str>,
) -> Result<ToolResult> {
    let skills = skill_storage.list_skills(category).await?;
    
    let skill_list: Vec<serde_json::Value> = skills.iter()
        .map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "description": s.description,
            "category": s.category.to_string(),
            "version": s.version,
            "usage_count": s.usage_count,
            "success_rate": s.success_rate,
        }))
        .collect();

    Ok(ToolResult {
        tool_call_id: "list_skills".to_string(),
        success: true,
        output: serde_json::to_string_pretty(&skill_list)?,
    })
}

/// Search for skills by query
pub async fn search_skills(
    skill_storage: Arc<crate::skills::SkillStorage>,
    query: &str,
) -> Result<ToolResult> {
    let skills = skill_storage.search_skills(query).await?;
    
    let skill_list: Vec<serde_json::Value> = skills.iter()
        .map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "description": s.description,
            "category": s.category.to_string(),
            "triggers": s.triggers,
        }))
        .collect();

    Ok(ToolResult {
        tool_call_id: "search_skills".to_string(),
        success: true,
        output: if skill_list.is_empty() {
            format!("No skills found matching '{}'", query)
        } else {
            serde_json::to_string_pretty(&skill_list)?
        },
    })
}

/// Get detailed information about a skill
pub async fn get_skill_info(
    skill_storage: Arc<crate::skills::SkillStorage>,
    skill_id: &str,
) -> Result<ToolResult> {
    match skill_storage.get_skill(skill_id).await? {
        Some(skill) => {
            let info = serde_json::json!({
                "id": skill.id,
                "name": skill.name,
                "description": skill.description,
                "version": skill.version,
                "author": skill.author,
                "created_at": skill.created_at,
                "updated_at": skill.updated_at,
                "usage_count": skill.usage_count,
                "success_rate": skill.success_rate,
                "average_execution_time_ms": skill.average_execution_time_ms,
                "category": skill.category.to_string(),
                "tags": skill.tags,
                "triggers": skill.triggers,
                "input_schema": skill.implementation.input_schema,
                "output_schema": skill.implementation.output_schema,
                "required_tools": skill.implementation.required_tools,
                "examples": skill.implementation.examples.iter().map(|ex| serde_json::json!({
                    "description": ex.description,
                    "input": ex.input,
                    "output": ex.output,
                })).collect::<Vec<_>>(),
            });

            Ok(ToolResult {
                tool_call_id: skill_id.to_string(),
                success: true,
                output: serde_json::to_string_pretty(&info)?,
            })
        }
        None => Ok(ToolResult {
            tool_call_id: skill_id.to_string(),
            success: false,
            output: format!("Skill '{}' not found", skill_id),
        }),
    }
}

/// Create a new skill manually
pub async fn create_skill(
    skill_storage: Arc<crate::skills::SkillStorage>,
    name: &str,
    description: &str,
    prompt_template: &str,
    author: &str,
    category: Option<&str>,
) -> Result<ToolResult> {
    let category = match category {
        Some("code_generation") => SkillCategory::CodeGeneration,
        Some("code_analysis") => SkillCategory::CodeAnalysis,
        Some("file_manipulation") => SkillCategory::FileManipulation,
        Some("web_research") => SkillCategory::WebResearch,
        Some("communication") => SkillCategory::Communication,
        Some("data_processing") => SkillCategory::DataProcessing,
        Some("workflow_automation") => SkillCategory::WorkflowAutomation,
        _ => SkillCategory::Custom(category.unwrap_or("general").to_string()),
    };

    let mut skill = Skill::default();
    skill.name = name.to_string();
    skill.description = description.to_string();
    skill.author = author.to_string();
    skill.category = category;
    skill.implementation.prompt_template = prompt_template.to_string();
    
    // Generate triggers from name
    skill.triggers = name.to_lowercase()
        .split_whitespace()
        .filter(|s| s.len() > 3)
        .map(|s| s.to_string())
        .collect();

    skill_storage.store_skill(&skill).await?;

    Ok(ToolResult {
        tool_call_id: "create_skill".to_string(),
        success: true,
        output: format!("Created skill '{}' ({}) successfully", skill.name, skill.id),
    })
}

/// Delete a skill
pub async fn delete_skill(
    skill_storage: Arc<crate::skills::SkillStorage>,
    skill_id: &str,
) -> Result<ToolResult> {
    match skill_storage.get_skill(skill_id).await? {
        Some(skill) => {
            skill_storage.delete_skill(skill_id).await?;
            Ok(ToolResult {
                tool_call_id: skill_id.to_string(),
                success: true,
                output: format!("Deleted skill '{}'", skill.name),
            })
        }
        None => Ok(ToolResult {
            tool_call_id: skill_id.to_string(),
            success: false,
            output: format!("Skill '{}' not found", skill_id),
        }),
    }
}

/// Export skills to agentskills.io format
pub async fn export_skills(
    skill_storage: Arc<crate::skills::SkillStorage>,
    skill_ids: Option<Vec<String>>,
) -> Result<ToolResult> {
    let export = skill_storage.export_to_agentskills(skill_ids).await?;
    let json = export.to_json()?;

    Ok(ToolResult {
        tool_call_id: "export_skills".to_string(),
        success: true,
        output: json,
    })
}

/// Get skill execution statistics
pub async fn get_skill_stats(
    skill_storage: Arc<crate::skills::SkillStorage>,
) -> Result<ToolResult> {
    let skills = skill_storage.list_skills(None).await?;
    
    let total_skills = skills.len() as i64;
    let total_executions: i64 = skills.iter().map(|s| s.usage_count).sum();
    let avg_success_rate = if total_skills > 0 {
        skills.iter().map(|s| s.success_rate).sum::<f64>() / total_skills as f64
    } else {
        0.0
    };

    let stats = serde_json::json!({
        "total_skills": total_skills,
        "total_executions": total_executions,
        "average_success_rate": avg_success_rate,
        "top_skills": skills.iter().take(10).map(|s| serde_json::json!({
            "name": s.name,
            "executions": s.usage_count,
            "success_rate": s.success_rate,
        })).collect::<Vec<_>>(),
    });

    Ok(ToolResult {
        tool_call_id: "get_skill_stats".to_string(),
        success: true,
        output: serde_json::to_string_pretty(&stats)?,
    })
}

/// Find and suggest skills for a given task
pub async fn suggest_skills(
    skill_executor: Arc<crate::skills::SkillExecutor>,
    task_description: &str,
) -> Result<ToolResult> {
    let skills = skill_executor.find_relevant_skills(task_description).await?;
    
    if skills.is_empty() {
        return Ok(ToolResult {
            tool_call_id: "suggest_skills".to_string(),
            success: true,
            output: "No relevant skills found for this task".to_string(),
        });
    }

    let suggestions: Vec<serde_json::Value> = skills.iter()
        .map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "description": s.description,
            "relevance": "high", // Simplified - could calculate actual relevance
        }))
        .collect();

    Ok(ToolResult {
        tool_call_id: "suggest_skills".to_string(),
        success: true,
        output: serde_json::to_string_pretty(&serde_json::json!({
            "task": task_description,
            "suggested_skills": suggestions,
        }))?,
    })
}
