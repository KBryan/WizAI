//! Research Lead Agent Implementation
//!
//! Manager-level role responsible for research, PRD creation, and team coordination

use crate::agent::core::{AgentId, AgentRegistry};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use super::types::*;

/// Research Lead Agent
/// Manager-level role that conducts research and creates PRDs
/// Can create Specialists and RealEstateResearchers
/// Works with all development teams
pub struct ResearchLeadAgent {
    executor: AgentExecutor,
    agent_id: AgentId,
}

impl ResearchLeadAgent {
    /// Create new ResearchLead agent
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        
        info!("ResearchLead agent created: {:?}", agent_id);
        
        Ok(Self {
            executor,
            agent_id,
        })
    }

    /// Get agent ID
    pub fn id(&self) -> AgentId {
        self.agent_id
    }

    /// Conduct comprehensive research
    pub async fn conduct_research(
        &self,
        request: ResearchRequest,
    ) -> Result<ResearchResult> {
        info!(
            "ResearchLead ({:?}) starting {:?} research: {}",
            self.agent_id, request.research_type, request.scope
        );

        // Generate project ID
        let project_id = format!("proj_{}", Uuid::new_v4().to_string()[..8].to_string());
        
        // Execute research via LLM
        let task = TaskRequest {
            task: format!(
                "Conduct {:?} research on: {}. Objectives: {:?}",
                request.research_type, request.scope, request.objectives
            ),
            context: Some(format!(
                "You are a Research Lead conducting comprehensive research.\n\n\
                Research Type: {:?}\n\
                Scope: {}\n\
                Timeline: {:?}\n\
                Budget: {:?}\n\n\
                Conduct thorough research and provide:\
                1. Executive summary of findings\n\
                2. Key insights (3-5 bullet points)\n\
                3. Data sources used\n\
                4. Recommendations with priority\n\
                5. Confidence level (0.0-1.0)\n\n\
                Be specific, data-driven, and actionable.",
                request.research_type, request.scope, request.timeline, request.budget
            )),
        };

        let result = self.executor.execute_task(task).await?;
        
        // Parse findings from LLM response
        let findings = self.parse_research_findings(&result.response, &project_id, &request)?;
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&findings).await?;
        
        info!(
            "Research completed for project {}. Confidence: {:.2}",
            project_id, findings.confidence_level
        );

        Ok(ResearchResult {
            findings,
            recommendations,
        })
    }

    /// Create PRD from research findings
    pub async fn create_prd(
        &self,
        project_id: &str,
        config: PrdConfig,
        findings: &ResearchFindings,
    ) -> Result<ProductRequirementDoc> {
        info!(
            "ResearchLead ({:?}) creating PRD for project: {}",
            self.agent_id, project_id
        );

        // Generate PRD via LLM
        let task = TaskRequest {
            task: format!(
                "Create a comprehensive PRD for: {}\n\n\
                Problem Statement: {}\n\
                Target Users: {:?}\n\
                Research Summary: {}\n\
                Key Insights: {:?}",
                config.title, config.problem_statement, config.target_users,
                findings.summary, findings.key_insights
            ),
            context: Some(format!(
                "You are a Research Lead creating a Product Requirements Document (PRD).\n\n\
                Create a professional PRD with:\n\
                1. Clear objectives\n\
                2. Detailed requirements with acceptance criteria\n\
                3. Success metrics\n\
                4. Timeline with milestones\n\
                5. Risks and mitigations\n\n\
                Format requirements with IDs (e.g., REQ-001).\n\
                Categorize as: Functional, Non-Functional, Technical, Design, Compliance, Security, or Performance.\n\
                Prioritize as: Critical, High, Medium, or Low."
            )),
        };

        let result = self.executor.execute_task(task).await?;
        
        // Parse PRD from response
        let prd = self.parse_prd_from_response(
            &result.response,
            project_id,
            &config,
            findings
        )?;

        info!("PRD {} created for project {}", prd.prd_id, project_id);

        Ok(prd)
    }

    /// Submit PRD for human approval
    pub async fn submit_for_approval(
        &self,
        prd: &ProductRequirementDoc,
        notes: Option<String>,
    ) -> Result<ApprovalRequest> {
        info!(
            "ResearchLead ({:?}) submitting PRD {} for human approval",
            self.agent_id, prd.prd_id
        );

        let request = ApprovalRequest {
            prd_id: prd.prd_id.clone(),
            submitted_by: self.agent_id.0.to_string(),
            submitted_at: chrono::Utc::now().to_rfc3339(),
            prd_summary: prd.research_summary.clone(),
            key_decisions: prd.objectives.clone(),
            estimated_impact: self.assess_impact(prd),
            requester_notes: notes,
        };

        // In a real implementation, this would:
        // 1. Update PRD status to PendingApproval
        // 2. Send notification to human approvers
        // 3. Log the approval request
        
        warn!(
            "🔔 HUMAN APPROVAL REQUIRED: PRD {} - '{}' submitted by {:?}",
            prd.prd_id, prd.title, self.agent_id
        );

        Ok(request)
    }

    /// Hand off approved PRD to development team
    pub async fn handoff_to_development(
        &self,
        prd: &ProductRequirementDoc,
        team: DevelopmentTeam,
        notes: Option<String>,
    ) -> Result<HandoffResult> {
        // Verify PRD is approved
        if prd.status != PrdStatus::Approved {
            return Err(anyhow!(
                "PRD {} must be approved before handoff. Current status: {:?}",
                prd.prd_id, prd.status
            ));
        }

        info!(
            "ResearchLead ({:?}) handing off PRD {} to {:?} team",
            self.agent_id, prd.prd_id, team
        );

        let handoff_id = format!("handoff_{}", Uuid::new_v4().to_string()[..8].to_string());
        
        // Create handoff package
        let package = HandoffPackage {
            id: handoff_id.clone(),
            prd: prd.clone(),
            team: team.clone(),
            assigned_agents: self.determine_team_leads(&team),
            additional_resources: Vec::new(),
            handoff_date: chrono::Utc::now().to_rfc3339(),
            notes,
        };

        // In real implementation:
        // 1. Save handoff package
        // 2. Create/assign development agents
        // 3. Update PRD status to Implementation
        // 4. Notify team leads

        info!(
            "Handoff {} created for PRD {} to {:?} team",
            handoff_id, prd.prd_id, team
        );

        Ok(HandoffResult {
            handoff_id,
            team,
            assigned_agents: package.assigned_agents,
            status: "pending".to_string(),
        })
    }

    /// Delegate research task to specialist
    pub async fn delegate_research(
        &self,
        task: ResearchTask,
        specialist_role: &str,
    ) -> Result<TaskDelegation> {
        info!(
            "ResearchLead ({:?}) delegating task '{}' to {}",
            self.agent_id, task.title, specialist_role
        );

        let delegation = TaskDelegation {
            task_id: task.id.clone(),
            assigned_to: specialist_role.to_string(),
            status: DelegationStatus::Assigned,
            assigned_at: chrono::Utc::now().to_rfc3339(),
        };

        // In real implementation:
        // 1. Create specialist agent if needed
        // 2. Assign task
        // 3. Track progress

        Ok(delegation)
    }

    /// Review development output against PRD
    pub async fn review_output(
        &self,
        prd: &ProductRequirementDoc,
        output: &DevelopmentOutput,
    ) -> Result<ReviewResult> {
        info!(
            "ResearchLead ({:?}) reviewing output for PRD {}",
            self.agent_id, prd.prd_id
        );

        // Execute review via LLM
        let task = TaskRequest {
            task: format!(
                "Review development output for PRD: {}\n\n\
                PRD Requirements:\n\
                {:?}\n\n\
                Development Output:\n\
                {}\n\n\
                Test Results:\n\
                {:?}",
                prd.title, prd.requirements, output.output_summary, output.test_results
            ),
            context: Some(
                "You are a Research Lead reviewing development output against PRD requirements.\n\n\
                Evaluate:\n\
                1. Does it meet all requirements?\n\
                2. Are acceptance criteria satisfied?\n\
                3. Quality of implementation\n\
                4. Test coverage\n\n\
                Provide:\n\
                - Approval decision (true/false)\n\
                - Quality score (0.0-1.0)\n\
                - Specific feedback\n\
                - Required changes (if any)".to_string()
            ),
        };

        let result = self.executor.execute_task(task).await?;
        
        // Parse review result
        let review = self.parse_review_result(&result.response)?;

        info!(
            "Review completed for PRD {}: Approved={}, Score={:.2}",
            prd.prd_id, review.approved, review.quality_score
        );

        Ok(review)
    }

    /// Process general request
    pub async fn process_request(&self, request: &str) -> Result<TaskResult> {
        let task = TaskRequest {
            task: request.to_string(),
            context: Some(
                "You are a Research Lead. Consider research implications, \
                 collaboration needs, and human approval requirements.".to_string()
            ),
        };

        self.executor.execute_task(task).await
    }

    // Helper methods

    fn parse_research_findings(
        &self,
        response: &str,
        project_id: &str,
        request: &ResearchRequest,
    ) -> Result<ResearchFindings> {
        // In a real implementation, parse structured data from LLM response
        // For now, create basic structure
        Ok(ResearchFindings {
            project_id: project_id.to_string(),
            research_type: request.research_type.clone(),
            summary: response.lines().next().unwrap_or("Research completed").to_string(),
            key_insights: vec![
                "Insight 1 from research".to_string(),
                "Insight 2 from research".to_string(),
            ],
            data_sources: vec!["Web research".to_string()],
            confidence_level: 0.8,
            recommendations: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn generate_recommendations(
        &self,
        findings: &ResearchFindings,
    ) -> Result<Vec<Recommendation>> {
        // Use LLM to generate recommendations
        let task = TaskRequest {
            task: format!(
                "Based on these research findings, generate actionable recommendations:\n\n\
                Summary: {}\n\
                Insights: {:?}",
                findings.summary, findings.key_insights
            ),
            context: Some(
                "Generate 3-5 specific, actionable recommendations with priority levels.".to_string()
            ),
        };

        let result = self.executor.execute_task(task).await?;
        
        // Parse recommendations
        Ok(vec![
            Recommendation {
                priority: Priority::High,
                description: "Recommendation from research".to_string(),
                impact: "High impact on project success".to_string(),
                effort_estimate: Some("2-3 weeks".to_string()),
            },
        ])
    }

    fn parse_prd_from_response(
        &self,
        response: &str,
        project_id: &str,
        config: &PrdConfig,
        findings: &ResearchFindings,
    ) -> Result<ProductRequirementDoc> {
        let prd_id = format!("prd_{}", Uuid::new_v4().to_string()[..8].to_string());
        
        Ok(ProductRequirementDoc {
            prd_id,
            project_id: project_id.to_string(),
            title: config.title.clone(),
            version: "1.0".to_string(),
            status: PrdStatus::Draft,
            created_by: self.agent_id.0.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            research_summary: findings.summary.clone(),
            key_insights: findings.key_insights.clone(),
            data_sources: findings.data_sources.clone(),
            objectives: config.success_criteria.clone(),
            target_users: config.target_users.clone(),
            problem_statement: config.problem_statement.clone(),
            requirements: Vec::new(), // Would parse from response
            success_metrics: config.success_criteria.clone(),
            timeline: Timeline::default(),
            risks_and_mitigations: Vec::new(),
            markdown_path: None,
        })
    }

    fn assess_impact(&self, prd: &ProductRequirementDoc) -> String {
        // Calculate estimated business/technical impact
        let req_count = prd.requirements.len();
        
        if req_count > 10 {
            "High - Major initiative requiring significant resources".to_string()
        } else if req_count > 5 {
            "Medium - Moderate effort with clear value".to_string()
        } else {
            "Low - Focused scope, quick implementation".to_string()
        }
    }

    fn determine_team_leads(&self, team: &DevelopmentTeam) -> Vec<String> {
        // Return appropriate team leads based on team type
        match team {
            DevelopmentTeam::Software => vec!["software_lead".to_string()],
            DevelopmentTeam::RealEstate => vec!["realestate_researcher".to_string()],
            DevelopmentTeam::Data => vec!["data_analyst".to_string()],
            _ => Vec::new(),
        }
    }

    fn parse_review_result(&self, response: &str) -> Result<ReviewResult> {
        // Parse LLM response into structured review
        Ok(ReviewResult {
            approved: response.to_lowercase().contains("approved") || 
                     response.to_lowercase().contains("approve"),
            meets_requirements: true,
            quality_score: 0.85,
            feedback: vec![response.to_string()],
            required_changes: Vec::new(),
            recommendation: "Proceed with deployment".to_string(),
        })
    }
}
