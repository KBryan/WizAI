//! Communication data models for AI Real Estate Team
//!
//! Defines structures for draft generation, approval workflow, and communication tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Risk classification for AI-generated content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low risk: appointment reminders, internal summaries
    Low,
    /// Medium risk: client follow-ups, property descriptions
    Medium,
    /// High risk: pricing, contracts, negotiations, legal advice
    High,
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Medium
    }
}

impl RiskLevel {
    /// Get display name for the risk level
    pub fn display_name(&self) -> &'static str {
        match self {
            RiskLevel::Low => "Low Risk",
            RiskLevel::Medium => "Medium Risk",
            RiskLevel::High => "High Risk - Requires Approval",
        }
    }

    /// Check if this risk level requires approval
    pub fn requires_approval(&self) -> bool {
        matches!(self, RiskLevel::Medium | RiskLevel::High)
    }

    /// Check if this is high risk
    pub fn is_high_risk(&self) -> bool {
        matches!(self, RiskLevel::High)
    }
}

/// Approval status for communication drafts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum ApprovalStatus {
    /// Initial draft, not yet reviewed
    Draft,
    /// Submitted for approval
    Pending,
    /// Auto-approved (low risk)
    AutoApproved,
    /// Human agent approved
    Approved,
    /// Human agent rejected
    Rejected,
    /// Sent to client
    Sent,
    /// Archived
    Archived,
}

impl Default for ApprovalStatus {
    fn default() -> Self {
        ApprovalStatus::Draft
    }
}

/// Communication channel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum CommunicationChannel {
    /// Email communication
    Email,
    /// SMS/Text message
    Sms,
    /// Phone call
    Phone,
    /// In-person meeting
    InPerson,
    /// Video call
    Video,
}

impl Default for CommunicationChannel {
    fn default() -> Self {
        CommunicationChannel::Email
    }
}

/// AI-generated communication draft
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommunicationDraft {
    /// Unique identifier
    pub id: String,
    /// Associated lead ID
    pub lead_id: String,
    /// Human agent who requested the draft
    pub requested_by: String,
    /// AI agent that generated the draft
    pub generated_by: String,
    /// Communication channel
    pub channel: CommunicationChannel,
    /// Draft subject (for emails)
    pub subject: Option<String>,
    /// Draft content/body
    pub content: String,
    /// Risk classification
    pub risk_level: RiskLevel,
    /// Current approval status
    pub status: ApprovalStatus,
    /// AI confidence score (0-100)
    pub confidence: i32,
    /// Reason for risk classification
    pub risk_reason: Option<String>,
    /// Human agent who reviewed
    pub reviewed_by: Option<String>,
    /// Review notes
    pub review_notes: Option<String>,
    /// When reviewed
    pub reviewed_at: Option<DateTime<Utc>>,
    /// Final content after edits
    pub final_content: Option<String>,
    /// When sent to client
    pub sent_at: Option<DateTime<Utc>>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl CommunicationDraft {
    /// Create a new draft
    pub fn new(
        lead_id: impl Into<String>,
        requested_by: impl Into<String>,
        generated_by: impl Into<String>,
        channel: CommunicationChannel,
        subject: Option<impl Into<String>>,
        content: impl Into<String>,
        risk_level: RiskLevel,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            lead_id: lead_id.into(),
            requested_by: requested_by.into(),
            generated_by: generated_by.into(),
            channel,
            subject: subject.map(|s| s.into()),
            content: content.into(),
            risk_level,
            status: ApprovalStatus::Draft,
            confidence: 0,
            risk_reason: None,
            reviewed_by: None,
            review_notes: None,
            reviewed_at: None,
            final_content: None,
            sent_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Submit for approval
    pub fn submit_for_approval(&mut self) {
        self.status = ApprovalStatus::Pending;
        self.updated_at = Utc::now();
    }

    /// Mark as auto-approved (for low risk)
    pub fn auto_approve(&mut self) {
        self.status = ApprovalStatus::AutoApproved;
        self.updated_at = Utc::now();
    }

    /// Approve the draft
    pub fn approve(&mut self, reviewer: impl Into<String>, notes: Option<impl Into<String>>) {
        self.status = ApprovalStatus::Approved;
        self.reviewed_by = Some(reviewer.into());
        self.review_notes = notes.map(|n| n.into());
        self.reviewed_at = Some(Utc::now());
        self.final_content = Some(self.content.clone());
        self.updated_at = Utc::now();
    }

    /// Reject the draft
    pub fn reject(&mut self, reviewer: impl Into<String>, reason: impl Into<String>) {
        self.status = ApprovalStatus::Rejected;
        self.reviewed_by = Some(reviewer.into());
        self.review_notes = Some(reason.into());
        self.reviewed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Edit the draft
    pub fn edit(&mut self, edited_content: impl Into<String>) {
        self.final_content = Some(edited_content.into());
        self.updated_at = Utc::now();
        // Reset to pending if already approved
        if matches!(
            self.status,
            ApprovalStatus::Approved | ApprovalStatus::AutoApproved
        ) {
            self.status = ApprovalStatus::Pending;
        }
    }

    /// Mark as sent
    pub fn mark_sent(&mut self) {
        self.status = ApprovalStatus::Sent;
        self.sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Get the effective content (final if edited, original if not)
    pub fn get_effective_content(&self) -> &str {
        self.final_content.as_deref().unwrap_or(&self.content)
    }

    /// Check if this draft can be approved
    pub fn can_approve(&self) -> bool {
        matches!(self.status, ApprovalStatus::Draft | ApprovalStatus::Pending)
    }

    /// Check if this draft is ready to send
    pub fn is_ready_to_send(&self) -> bool {
        matches!(
            self.status,
            ApprovalStatus::Approved | ApprovalStatus::AutoApproved
        )
    }

    /// Add AI disclosure header
    pub fn with_disclosure(&self, agent_name: &str) -> String {
        let disclosure = format!("[AI DRAFT - Reviewed by {}]\n\n", agent_name);
        format!("{}{}", disclosure, self.get_effective_content())
    }
}

/// Communication template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommunicationTemplate {
    /// Unique identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Description of when to use
    pub description: Option<String>,
    /// Communication channel
    pub channel: CommunicationChannel,
    /// Template subject
    pub subject_template: Option<String>,
    /// Template body/content
    pub body_template: String,
    /// Default risk level for this template
    pub default_risk_level: RiskLevel,
    /// Variables available in template (JSON array)
    pub variables: Option<serde_json::Value>,
    /// Is this template active
    pub is_active: bool,
    /// Created by
    pub created_by: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl CommunicationTemplate {
    /// Create a new template
    pub fn new(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        channel: CommunicationChannel,
        subject: Option<impl Into<String>>,
        body: impl Into<String>,
        default_risk: RiskLevel,
        created_by: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.map(|d| d.into()),
            channel,
            subject_template: subject.map(|s| s.into()),
            body_template: body.into(),
            default_risk_level: default_risk,
            variables: None,
            is_active: true,
            created_by: created_by.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Request to generate a new draft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDraftRequest {
    pub lead_id: String,
    pub channel: CommunicationChannel,
    pub subject: Option<String>,
    pub template_id: Option<String>,
    pub prompt: String,
    pub context: Option<String>,
}

/// Request to approve/reject a draft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDraftRequest {
    pub action: ReviewAction,
    pub notes: Option<String>,
}

/// Review action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewAction {
    Approve,
    Reject,
}

/// Communication filter for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationFilter {
    pub lead_id: Option<String>,
    pub status: Option<ApprovalStatus>,
    pub risk_level: Option<RiskLevel>,
    pub channel: Option<CommunicationChannel>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub reviewed_by: Option<String>,
}

/// Pending approval summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApprovalSummary {
    pub id: String,
    pub lead_id: String,
    pub lead_name: String,
    pub channel: CommunicationChannel,
    pub subject: Option<String>,
    pub risk_level: RiskLevel,
    pub created_at: DateTime<Utc>,
    pub preview: String,
}

/// Edit history entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DraftEditHistory {
    pub id: String,
    pub draft_id: String,
    pub previous_content: String,
    pub new_content: String,
    pub edited_by: String,
    pub edit_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Sent communication record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SentCommunication {
    pub id: String,
    pub draft_id: String,
    pub lead_id: String,
    pub channel: CommunicationChannel,
    pub subject: Option<String>,
    pub content: String,
    pub sent_by: String,
    pub sent_at: DateTime<Utc>,
    pub delivery_status: Option<String>,
    pub opened_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
}
