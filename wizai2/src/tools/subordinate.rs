use crate::agent::core::{AgentEvent, AgentId, AgentRegistry, AgentRole, AgentStatus, Message, MessageRole};
use crate::agent::executor::TaskRequest;
use crate::agent::executor::AgentExecutor;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubordinateRequest {
    pub superior_id: AgentId,
    pub task: String,
    pub role: Option<AgentRole>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubordinateResponse {
    pub success: bool,
    pub subordinate_id: Option<AgentId>,
    pub result: String,
}

pub async fn create_subordinate(
    registry: &mut AgentRegistry,
    request: SubordinateRequest,
) -> Result<SubordinateResponse> {
    debug!(
        "Creating subordinate for superior {:?} with task: {}",
        request.superior_id, request.task
    );

    // Get superior agent
    let superior = registry
        .get_agent(request.superior_id)
        .ok_or_else(|| anyhow!("Superior agent not found"))?;

    let superior_guard = superior.read().await;
    let superior_role = superior_guard.role.clone();
    let superior_id = superior_guard.id;
    drop(superior_guard);

    // Determine subordinate role
    let subordinate_role = request.role.unwrap_or_else(|| {
        // Default to one level below superior
        match superior_role {
            AgentRole::CEO | AgentRole::CTO | AgentRole::CFO | AgentRole::ChiefAI | AgentRole::ChiefProduct => {
                AgentRole::VP
            }
            AgentRole::VP | AgentRole::Director => AgentRole::Manager,
            AgentRole::Manager => AgentRole::Lead,
            AgentRole::Lead => AgentRole::Specialist,
            _ => AgentRole::Specialist,
        }
    });

    // Check if superior can create this role
    if !superior_role.can_create(&subordinate_role) {
        return Ok(SubordinateResponse {
            success: false,
            subordinate_id: None,
            result: format!("{:?} cannot create {:?} agents", superior_role, subordinate_role),
        });
    }

    // Generate name if not provided
    let name = request.name.unwrap_or_else(|| {
        format!("{} {}'s {}", subordinate_role.name(), superior_id.0.to_string()[..8].to_string(), subordinate_role.name())
    });

    // Generate system prompt based on role
    let system_prompt = generate_role_prompt(&subordinate_role, &name);

    // Create subordinate agent
    let subordinate_id = registry
        .create_agent(name, subordinate_role.clone(), Some(superior_id), system_prompt)
        .await?;

    info!("Created subordinate agent {:?} with role {:?}", subordinate_id, subordinate_role);

    // Execute the task
    let executor = AgentExecutor::new(subordinate_id, Arc::new(RwLock::new(registry.clone()))).await?;
    
    let task_request = TaskRequest {
        task: request.task.clone(),
        context: Some(format!("You report to {:?} ({}). Complete this task independently.", superior_id, superior_role.name())),
    };

    match executor.execute_task(task_request).await {
        Ok(result) => {
            info!("Subordinate {:?} completed task successfully", subordinate_id);
            Ok(SubordinateResponse {
                success: true,
                subordinate_id: Some(subordinate_id),
                result: result.response,
            })
        }
        Err(e) => {
            error!("Subordinate {:?} failed to complete task: {}", subordinate_id, e);
            Ok(SubordinateResponse {
                success: false,
                subordinate_id: Some(subordinate_id),
                result: format!("Task failed: {}", e),
            })
        }
    }
}

fn generate_role_prompt(role: &AgentRole, name: &str) -> String {
    match role {
        AgentRole::CEO => format!(
            "You are {}, the Chief Executive Officer of the organization. \
            Your role is to set strategic direction, make high-level decisions, \
            and delegate tasks to other C-level executives and managers. \
            Think about the big picture and long-term goals.",
            name
        ),
        AgentRole::CTO => format!(
            "You are {}, the Chief Technology Officer. \
            You oversee all technical aspects of the organization, \
            make architecture decisions, and guide the engineering teams. \
            Focus on technical excellence and innovation.",
            name
        ),
        AgentRole::CFO => format!(
            "You are {}, the Chief Financial Officer. \
            You manage the organization's finances, budgeting, and financial strategy. \
            Ensure fiscal responsibility and provide financial insights.",
            name
        ),
        AgentRole::ChiefAI => format!(
            "You are {}, the Chief AI Officer. \
            You oversee AI/ML strategy, model selection, and AI ethics. \
            Guide the organization in AI adoption and implementation.",
            name
        ),
        AgentRole::Manager => format!(
            "You are {}, a Manager. \
            You coordinate team efforts, track progress, and ensure deliverables meet quality standards. \
            Support your team and escalate blockers when needed.",
            name
        ),
        AgentRole::Lead => format!(
            "You are {}, a Team Lead. \
            You provide technical guidance to your team, review work, and help with complex problems. \
            Mentor junior team members and ensure best practices.",
            name
        ),
        AgentRole::Specialist => format!(
            "You are {}, a Specialist. \
            You are an expert in your domain and execute tasks with precision. \
            Focus on delivering high-quality work efficiently.",
            name
        ),
        _ => format!(
            "You are {}, a {} in the organization. \
            Complete tasks assigned to you efficiently and report back with results.",
            name,
            role.name()
        ),
    }
}

pub async fn get_subordinate_info(
    registry: &AgentRegistry,
    agent_id: AgentId,
) -> Result<Vec<(AgentId, AgentRole, String)>> {
    let agent = registry
        .get_agent(agent_id)
        .ok_or_else(|| anyhow!("Agent not found"))?;

    let agent_guard = agent.read().await;
    let mut subordinates = Vec::new();

    for sub_id in &agent_guard.subordinates {
        if let Some(sub) = registry.get_agent(*sub_id) {
            let sub_guard = sub.read().await;
            subordinates.push((
                *sub_id,
                sub_guard.role.clone(),
                sub_guard.name.clone(),
            ));
        }
    }

    Ok(subordinates)
}

pub async fn delegate_task(
    registry: &AgentRegistry,
    superior_id: AgentId,
    subordinate_id: AgentId,
    task: &str,
) -> Result<String> {
    debug!("Delegating task from {:?} to {:?}", superior_id, subordinate_id);

    // Verify relationship
    let superior = registry
        .get_agent(superior_id)
        .ok_or_else(|| anyhow!("Superior not found"))?;

    let superior_guard = superior.read().await;
    if !superior_guard.subordinates.contains(&subordinate_id) {
        return Err(anyhow!("Agent {:?} is not a subordinate of {:?}", subordinate_id, superior_id));
    }
    drop(superior_guard);

    // Create executor for subordinate
    let executor = AgentExecutor::new(
        subordinate_id,
        Arc::new(RwLock::new(registry.clone())),
    )
    .await?;

    let task_request = TaskRequest {
        task: task.to_string(),
        context: Some(format!(
            "This task was delegated to you by {:?}",
            superior_id
        )),
    };

    match executor.execute_task(task_request).await {
        Ok(result) => {
            info!("Task delegated successfully to {:?}", subordinate_id);
            Ok(result.response)
        }
        Err(e) => {
            error!("Failed to execute delegated task: {}", e);
            Err(e)
        }
    }
}