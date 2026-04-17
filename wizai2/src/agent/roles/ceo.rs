use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// CEO Agent - Top of the organizational hierarchy
pub struct CEOAgent {
    executor: AgentExecutor,
}

impl CEOAgent {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self { executor })
    }

    pub async fn process_request(&self, request: &str) -> Result<TaskResult> {
        info!("CEO processing request: {}", request);

        let task = TaskRequest {
            task: request.to_string(),
            context: Some(
                "As CEO, consider the strategic implications and delegate to appropriate executives if needed."
                    .to_string(),
            ),
        };

        self.executor.execute_task(task).await
    }

    pub async fn make_strategic_decision(&self, options: &[String]) -> Result<String> {
        let options_str = options.join(", ");
        let task = TaskRequest {
            task: format!(
                "Make a strategic decision between these options: {}",
                options_str
            ),
            context: Some(
                "Consider organizational goals, resources, risks, and long-term impact."
                    .to_string(),
            ),
        };

        let result = self.executor.execute_task(task).await?;
        Ok(result.response)
    }
}
