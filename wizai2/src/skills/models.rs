use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A skill is a reusable capability extracted from successful task completions
/// Following the agentskills.io open standard format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_count: i64,
    pub success_rate: f64,
    pub average_execution_time_ms: i64,
    pub tags: Vec<String>,
    pub category: SkillCategory,
    pub triggers: Vec<String>,
    pub implementation: SkillImplementation,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillCategory {
    CodeGeneration,
    CodeAnalysis,
    FileManipulation,
    WebResearch,
    Communication,
    DataProcessing,
    WorkflowAutomation,
    Custom(String),
}

impl SkillCategory {
    pub fn to_string(&self) -> String {
        match self {
            SkillCategory::CodeGeneration => "code_generation".to_string(),
            SkillCategory::CodeAnalysis => "code_analysis".to_string(),
            SkillCategory::FileManipulation => "file_manipulation".to_string(),
            SkillCategory::WebResearch => "web_research".to_string(),
            SkillCategory::Communication => "communication".to_string(),
            SkillCategory::DataProcessing => "data_processing".to_string(),
            SkillCategory::WorkflowAutomation => "workflow_automation".to_string(),
            SkillCategory::Custom(s) => s.clone(),
        }
    }
}

impl From<String> for SkillCategory {
    fn from(s: String) -> Self {
        match s.as_str() {
            "code_generation" => SkillCategory::CodeGeneration,
            "code_analysis" => SkillCategory::CodeAnalysis,
            "file_manipulation" => SkillCategory::FileManipulation,
            "web_research" => SkillCategory::WebResearch,
            "communication" => SkillCategory::Communication,
            "data_processing" => SkillCategory::DataProcessing,
            "workflow_automation" => SkillCategory::WorkflowAutomation,
            _ => SkillCategory::Custom(s),
        }
    }
}

/// The implementation details of a skill
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillImplementation {
    pub skill_type: SkillType,
    pub prompt_template: String,
    pub system_prompt: Option<String>,
    pub required_tools: Vec<String>,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub examples: Vec<SkillExample>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SkillType {
    /// Single-turn skill that generates content
    Generator,
    /// Multi-step skill that executes tools
    Workflow,
    /// Analysis skill that processes input
    Analyzer,
    /// Composite skill that combines other skills
    Composite(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SkillExample {
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub description: String,
}

/// Input for executing a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionInput {
    pub skill_id: String,
    pub input: serde_json::Value,
    pub context: Option<SkillContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContext {
    pub agent_id: String,
    pub conversation_id: Option<String>,
    pub user_preferences: HashMap<String, String>,
    pub relevant_memories: Vec<String>,
}

/// Result of skill execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    pub skill_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub execution_time_ms: i64,
    pub tool_calls: Vec<SkillToolCall>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolCall {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub execution_time_ms: i64,
}

/// Request to generate a new skill from task completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillGenerationRequest {
    pub task_description: String,
    pub task_context: String,
    pub execution_trace: Vec<ExecutionStep>,
    pub outcome: TaskOutcome,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_number: i32,
    pub action: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutcome {
    Success {
        quality_score: f64,
        user_feedback: Option<String>,
    },
    PartialSuccess {
        issues: Vec<String>,
        user_feedback: Option<String>,
    },
    Failure {
        reason: String,
    },
}

/// A skill invocation trigger - patterns that suggest using a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    pub skill_id: String,
    pub trigger_type: TriggerType,
    pub pattern: String,
    pub confidence_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Keyword,
    Intent,
    Pattern,
    Semantic,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            description: String::new(),
            version: "1.0.0".to_string(),
            author: String::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            usage_count: 0,
            success_rate: 1.0,
            average_execution_time_ms: 0,
            tags: Vec::new(),
            category: SkillCategory::Custom("general".to_string()),
            triggers: Vec::new(),
            implementation: SkillImplementation {
                skill_type: SkillType::Generator,
                prompt_template: String::new(),
                system_prompt: None,
                required_tools: Vec::new(),
                input_schema: serde_json::json!({"type": "object"}),
                output_schema: serde_json::json!({"type": "object"}),
                examples: Vec::new(),
                constraints: Vec::new(),
            },
            metadata: HashMap::new(),
        }
    }
}

/// Agentskills.io compatibility - export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentskillsExport {
    pub spec_version: String,
    pub skills: Vec<Skill>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentskillsExport {
    pub fn new(skills: Vec<Skill>) -> Self {
        Self {
            spec_version: "1.0".to_string(),
            skills,
            metadata: HashMap::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
