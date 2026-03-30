use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// CFO Agent - Financial leadership
pub struct CFOAgent {
    executor: AgentExecutor,
}

impl CFOAgent {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self { executor })
    }

    pub async fn analyze_cost(&self, proposal: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: format!("Analyze the cost implications of: {}", proposal),
            context: Some(
                "Provide detailed cost breakdown, ROI analysis, and budget impact."
                    .to_string(),
            ),
        };

        info!("CFO analyzing costs");
        self.executor.execute_task(task).await
    }

    pub async fn budget_review(&self) -> Result<TaskResult> {
        let task = TaskRequest {
            task: "Review current budget status and provide recommendations".to_string(),
            context: Some(
                "Focus on fiscal responsibility and optimization opportunities.".to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }

    pub async fn process_request(&self, request: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: request.to_string(),
            context: Some(
                "As CFO, always consider financial implications and provide cost-benefit analysis."
                    .to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }
}
