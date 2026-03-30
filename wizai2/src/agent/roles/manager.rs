use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Manager Agent - Team management and execution
pub struct ManagerAgent {
    executor: AgentExecutor,
}

impl ManagerAgent {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self { executor })
    }

    pub async fn assign_task(&self, task_description: &str, assignee: Option<&str>) -> Result<TaskResult> {
        let context = if let Some(person) = assignee {
            format!("Assign this task to team member: {}. Ensure clarity and provide necessary context.", person)
        } else {
            "Assign this task to an appropriate team member.".to_string()
        };

        let task = TaskRequest {
            task: format!("Assign and manage task: {}", task_description),
            context: Some(context),
        };

        info!("Manager assigning task");
        self.executor.execute_task(task).await
    }

    pub async fn review_work(&self, work_output: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: format!("Review this work output: {}", work_output),
            context: Some(
                "Check for quality, completeness, and alignment with requirements. Provide constructive feedback."
                    .to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }

    pub async fn process_request(&self, request: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: request.to_string(),
            context: Some(
                "As Manager, ensure tasks are completed on time, within scope, and to quality standards."
                    .to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }
}
