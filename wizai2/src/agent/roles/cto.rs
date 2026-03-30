use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// CTO Agent - Technical leadership
pub struct CTOAgent {
    executor: AgentExecutor,
}

impl CTOAgent {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self { executor })
    }

    pub async fn review_architecture(&self, design: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: format!("Review this architecture/design: {}", design),
            context: Some(
                "Focus on scalability, security, maintainability, and technical feasibility."
                    .to_string(),
            ),
        };

        info!("CTO reviewing architecture");
        self.executor.execute_task(task).await
    }

    pub async fn evaluate_tech_stack(&self, requirements: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: format!("Evaluate technology stack for: {}", requirements),
            context: Some(
                "Consider performance, ecosystem, team expertise, and long-term viability."
                    .to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }

    pub async fn process_request(&self, request: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: request.to_string(),
            context: Some(
                "As CTO, ensure technical excellence and proper engineering practices."
                    .to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }
}
