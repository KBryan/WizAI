use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Chief AI Officer Agent - AI/ML strategy
pub struct ChiefAIOfficer {
    executor: AgentExecutor,
}

impl ChiefAIOfficer {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self { executor })
    }

    pub async fn evaluate_model(&self, model_name: &str, task: &str) -> Result<TaskResult> {
        let task_req = TaskRequest {
            task: format!("Evaluate model '{}' for task: {}", model_name, task),
            context: Some(
                "Consider accuracy, latency, cost, ethical implications, and data requirements."
                    .to_string(),
            ),
        };

        info!("Chief AI Officer evaluating model");
        self.executor.execute_task(task_req).await
    }

    pub async fn ai_strategy(&self, business_goal: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: format!("Develop AI strategy for: {}", business_goal),
            context: Some(
                "Consider current AI capabilities, roadmap, and ethical considerations.".to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }

    pub async fn process_request(&self, request: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: request.to_string(),
            context: Some(
                "As Chief AI Officer, ensure AI solutions are ethical, effective, and aligned with organizational goals."
                    .to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }
}
