//! Research Lead Agent Module
//!
//! Manager-level role responsible for:
//! - Conducting comprehensive research (market, user, technical, competitive)
//! - Creating Product Requirement Documents (PRDs)
//! - Managing research specialists (RealEstateResearcher, Data Analysts, etc.)
//! - Collaborating with all development teams
//! - Ensuring human approval before development handoff
//! - Reviewing development output against PRD requirements
//!
//! Research Flow:
//! 1. Research Lead conducts research → generates findings
//! 2. Creates PRD from findings
//! 3. Submits PRD for human approval
//! 4. Upon approval, hands off to development team
//! 5. Reviews development output
//!
//! Org Hierarchy:
//! - ResearchLead is at Manager level
//! - Can create Specialists and RealEstateResearchers
//! - Reports to Director/VP
//! - Collaborates with all teams (Software, RealEstate, Data, Marketing)

pub mod agent;
pub mod types;

// Re-export main types
pub use agent::ResearchLeadAgent;
pub use types::{
    ResearchType, ResearchRequest, ResearchFindings, ResearchResult,
    PrdStatus, PrdConfig, ProductRequirementDoc, Requirement, RequirementCategory,
    Priority, Timeline, Milestone, Risk, Recommendation,
    DevelopmentTeam, HandoffPackage, HandoffResult,
    ApprovalRequest, ResearchTask, TaskDelegation, DelegationStatus,
    DevelopmentOutput, TestResults, ReviewResult,
};
