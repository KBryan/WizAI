use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use serde::{Serialize, Deserialize};

/// Research Agent - For autonomous ML experimentation
pub struct ResearchAgent {
    executor: AgentExecutor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub name: String,
    pub model_depth: i32,
    pub batch_size: i32,
    pub learning_rate: f64,
    pub max_tokens: i64,
    pub budget_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub experiment_id: String,
    pub config: ExperimentConfig,
    pub status: ExperimentStatus,
    pub val_bpb: Option<f64>,
    pub cost_usd: f64,
    pub duration_secs: i64,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

impl ResearchAgent {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self { executor })
    }

    pub async fn design_experiment(&self, hypothesis: &str) -> Result<ExperimentConfig> {
        let task = TaskRequest {
            task: format!(
                "Design an ML experiment to test this hypothesis: {}",
                hypothesis
            ),
            context: Some(
                "You are an ML researcher. Design an experiment following autoresearch principles:\n\
                1. Model architecture based on depth parameter\n\
                2. Fixed 5-minute time budget per experiment\n\
                3. Track validation bits-per-byte (val_bpb)\n\
                4. Optimize for lower val_bpb (better model)\n\
                Return a JSON object with: name, model_depth, batch_size, learning_rate".to_string()
            ),
        };

        let result = self.executor.execute_task(task).await?;
        
        // Parse the response to extract experiment config
        // In a real implementation, we'd parse the LLM response
        let config = ExperimentConfig {
            name: format!("exp_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
            model_depth: 8,
            batch_size: 64,
            learning_rate: 0.001,
            max_tokens: 524288,
            budget_usd: 0.05,
        };

        info!("Designed experiment: {:?}", config);
        Ok(config)
    }

    pub async fn run_experiment(&self, config: ExperimentConfig) -> Result<ExperimentResult> {
        let experiment_id = uuid::Uuid::new_v4().to_string();
        
        info!("Starting experiment {}: {:?}", experiment_id, config);

        let task = TaskRequest {
            task: format!(
                "Execute ML experiment '{}' with config: {:?}",
                config.name, config
            ),
            context: Some(format!(
                "Execute this experiment following autoresearch patterns:\n\
                1. Setup: Prepare data and tokenizer\n\
                2. Train: Run for fixed 5-minute budget\n\
                3. Evaluate: Calculate val_bpb\n\
                4. Report: Return results as JSON\n\
                Budget: ${}",
                config.budget_usd
            )),
        };

        let start_time = chrono::Utc::now();
        
        match self.executor.execute_task(task).await {
            Ok(result) => {
                let duration = chrono::Utc::now().signed_duration_since(start_time);
                
                Ok(ExperimentResult {
                    experiment_id,
                    config,
                    status: ExperimentStatus::Completed,
                    val_bpb: Some(3.5), // Mock result
                    cost_usd: 0.05,
                    duration_secs: duration.num_seconds(),
                    artifacts: vec!["model.bin".to_string(), "results.json".to_string()],
                })
            }
            Err(e) => {
                Ok(ExperimentResult {
                    experiment_id,
                    config,
                    status: ExperimentStatus::Failed(e.to_string()),
                    val_bpb: None,
                    cost_usd: 0.02,
                    duration_secs: 0,
                    artifacts: vec![],
                })
            }
        }
    }

    pub async fn analyze_results(&self, results: &[ExperimentResult]) -> Result<String> {
        let task = TaskRequest {
            task: format!("Analyze {} experiment results and identify best configuration", results.len()),
            context: Some(
                "Compare experiments based on:\n\
                1. val_bpb (lower is better)\n\
                2. Cost efficiency (val_bpb per $)\n\
                3. Training stability\n\
                Provide recommendations for next experiments.".to_string()
            ),
        };

        let result = self.executor.execute_task(task).await?;
        Ok(result.response)
    }

    pub async fn process_request(&self, request: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: request.to_string(),
            context: Some(
                "As a Research Agent, focus on systematic experimentation and evidence-based conclusions.".to_string()
            ),
        };

        self.executor.execute_task(task).await
    }
}
