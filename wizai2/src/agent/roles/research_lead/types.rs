//! Type definitions for Research Lead Agent
//!
//! Research Lead is a Manager-level role responsible for:
//! - Conducting comprehensive research across all domains
//! - Creating PRDs (Product Requirements Documents)
//! - Managing research specialists
//! - Collaborating with all development teams
//! - Ensuring human approval before development handoff

use serde::{Deserialize, Serialize};

/// Research project types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResearchType {
    MarketResearch,      // Market size, trends, competition
    UserResearch,        // User interviews, personas, journey maps
    TechnicalResearch,   // Tech evaluation, architecture research
    CompetitiveAnalysis, // Competitor benchmarking
    FeasibilityStudy,    // Technical/business feasibility
}

/// PRD status with human-in-the-loop
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrdStatus {
    Draft,           // Initial creation
    InReview,        // Internal review by ResearchLead
    PendingApproval, // Submitted for human approval
    Approved,        // Human approved
    Rejected,        // Human rejected with feedback
    Implementation,  // Handed off to dev
    Completed,       // Fully implemented
    Archived,        // No longer relevant
}

/// Requirement categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequirementCategory {
    Functional,
    NonFunctional,
    Technical,
    Design,
    Compliance,
    Security,
    Performance,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

/// Team types for handoff
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DevelopmentTeam {
    Software,       // SoftwareDeveloper agents
    RealEstate,     // RealEstateResearcher agents
    Data,           // Data analysis specialists
    Marketing,      // Marketing specialists
    Operations,     // Operations team
    Custom(String), // Future teams
}

/// Research request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRequest {
    pub research_type: ResearchType,
    pub scope: String,
    pub objectives: Vec<String>,
    pub timeline: Option<String>,
    pub budget: Option<f64>,
}

/// Research findings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchFindings {
    pub project_id: String,
    pub research_type: ResearchType,
    pub summary: String,
    pub key_insights: Vec<String>,
    pub data_sources: Vec<String>,
    pub confidence_level: f32, // 0.0 - 1.0
    pub recommendations: Vec<Recommendation>,
    pub created_at: String,
}

/// Recommendation from research
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub description: String,
    pub impact: String,
    pub effort_estimate: Option<String>,
}

/// PRD Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrdConfig {
    pub title: String,
    pub target_users: Vec<String>,
    pub problem_statement: String,
    pub success_criteria: Vec<String>,
}

/// PRD Structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRequirementDoc {
    pub prd_id: String,
    pub project_id: String,
    pub title: String,
    pub version: String,
    pub status: PrdStatus,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,

    // Research foundation
    pub research_summary: String,
    pub key_insights: Vec<String>,
    pub data_sources: Vec<String>,

    // PRD Content
    pub objectives: Vec<String>,
    pub target_users: Vec<String>,
    pub problem_statement: String,
    pub requirements: Vec<Requirement>,
    pub success_metrics: Vec<String>,
    pub timeline: Timeline,
    pub risks_and_mitigations: Vec<Risk>,

    // File references
    pub markdown_path: Option<String>,
}

/// Requirement in PRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub category: RequirementCategory,
    pub priority: Priority,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub notes: Option<String>,
}

/// Timeline structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub start_date: String,
    pub target_completion: String,
    pub milestones: Vec<Milestone>,
}

/// Milestone in timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub target_date: String,
    pub deliverables: Vec<String>,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub impact: String,      // 'high', 'medium', 'low'
    pub probability: String, // 'high', 'medium', 'low'
    pub mitigation: String,
}

/// Research result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub findings: ResearchFindings,
    pub recommendations: Vec<Recommendation>,
}

/// Human approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub prd_id: String,
    pub submitted_by: String,
    pub submitted_at: String,
    pub prd_summary: String,
    pub key_decisions: Vec<String>,
    pub estimated_impact: String,
    pub requester_notes: Option<String>,
}

/// Handoff package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPackage {
    pub id: String,
    pub prd: ProductRequirementDoc,
    pub team: DevelopmentTeam,
    pub assigned_agents: Vec<String>, // Agent IDs
    pub additional_resources: Vec<String>,
    pub handoff_date: String,
    pub notes: Option<String>,
}

/// Handoff result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffResult {
    pub handoff_id: String,
    pub team: DevelopmentTeam,
    pub assigned_agents: Vec<String>,
    pub status: String,
}

/// Task delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDelegation {
    pub task_id: String,
    pub assigned_to: String, // Agent role name
    pub status: DelegationStatus,
    pub assigned_at: String,
}

/// Delegation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DelegationStatus {
    Assigned,
    InProgress,
    Completed,
    Failed(String),
}

/// Research task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub research_type: ResearchType,
    pub priority: Priority,
    pub deadline: Option<String>,
}

/// Development output for review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentOutput {
    pub handoff_id: String,
    pub output_summary: String,
    pub deliverables: Vec<String>,
    pub test_results: Option<TestResults>,
    pub notes: Option<String>,
}

/// Test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResults {
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub coverage_percentage: Option<f32>,
    pub test_summary: String,
}

/// Review result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub approved: bool,
    pub meets_requirements: bool,
    pub quality_score: f32, // 0.0 - 1.0
    pub feedback: Vec<String>,
    pub required_changes: Vec<String>,
    pub recommendation: String,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            start_date: chrono::Utc::now().to_rfc3339(),
            target_completion: chrono::Utc::now().to_rfc3339(),
            milestones: Vec::new(),
        }
    }
}
