use crate::agent::core::AgentId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub id: String,
    pub agent_id: AgentId,
    pub task_description: String,
    pub solution_content: String,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub success_count: u32,
}

#[derive(Debug, Default)]
pub struct SolutionCache {
    solutions: HashMap<String, Solution>,
}

impl SolutionCache {
    pub fn new() -> Self {
        Self {
            solutions: HashMap::new(),
        }
    }

    pub fn add_solution(&mut self, solution: Solution) {
        self.solutions.insert(solution.id.clone(), solution);
    }

    pub fn find_relevant(&self, task: &str) -> Vec<&Solution> {
        // Simple keyword matching for now
        let task_lower = task.to_lowercase();
        let keywords: Vec<&str> = task_lower.split_whitespace().collect();

        self.solutions
            .values()
            .filter(|sol| {
                let sol_text = format!(
                    "{} {}",
                    sol.task_description.to_lowercase(),
                    sol.tags.join(" ").to_lowercase()
                );
                keywords.iter().any(|kw| sol_text.contains(kw))
            })
            .take(3)
            .collect()
    }

    pub fn increment_success(&mut self, solution_id: &str) -> Result<()> {
        if let Some(sol) = self.solutions.get_mut(solution_id) {
            sol.success_count += 1;
        }
        Ok(())
    }
}
