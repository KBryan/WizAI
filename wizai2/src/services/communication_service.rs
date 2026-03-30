//! Communication Service
//!
//! Business logic for generating communication drafts and managing approval workflow.

use crate::compliance::{ApprovalEngine, AuditLogger};
use crate::models::{
    ApprovalStatus, CommunicationChannel, CommunicationDraft, CommunicationFilter,
    CommunicationTemplate, DraftEditHistory, GenerateDraftRequest, PendingApprovalSummary,
    ReviewAction, ReviewDraftRequest, RiskLevel,
};
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Pool, Sqlite};

/// Service for managing AI-generated communications
pub struct CommunicationService {
    db_pool: Pool<Sqlite>,
    approval_engine: ApprovalEngine,
}

impl CommunicationService {
    /// Create a new communication service
    pub fn new(db_pool: Pool<Sqlite>) -> Self {
        Self {
            db_pool,
            approval_engine: ApprovalEngine::new(),
        }
    }

    /// Generate a communication draft
    pub async fn generate_draft(
        &self,
        lead_id: &str,
        requested_by: &str,
        generated_by: &str,
        channel: CommunicationChannel,
        subject: Option<String>,
        content: String,
    ) -> Result<CommunicationDraft> {
        // Classify risk
        let risk_classification = self.approval_engine.classify_risk(&content, "email");

        // Create draft
        let mut draft = CommunicationDraft::new(
            lead_id,
            requested_by,
            generated_by,
            channel,
            subject,
            content,
            risk_classification.level,
        );

        draft.confidence = risk_classification.confidence;
        draft.risk_reason = risk_classification.reason;

        // Auto-approve low risk if enabled
        if self.approval_engine.should_auto_approve(&draft.risk_level) {
            draft.auto_approve();
        } else {
            draft.submit_for_approval();
        }

        // Save to database
        sqlx::query(
            r#"
            INSERT INTO communication_drafts (
                id, lead_id, requested_by, generated_by, channel, subject, content,
                risk_level, status, confidence, risk_reason, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&draft.id)
        .bind(&draft.lead_id)
        .bind(&draft.requested_by)
        .bind(&draft.generated_by)
        .bind(format!("{:?}", draft.channel).to_lowercase())
        .bind(&draft.subject)
        .bind(&draft.content)
        .bind(format!("{:?}", draft.risk_level).to_lowercase())
        .bind(format!("{:?}", draft.status).to_lowercase())
        .bind(draft.confidence)
        .bind(&draft.risk_reason)
        .bind(draft.created_at)
        .bind(draft.updated_at)
        .execute(&self.db_pool)
        .await
        .context("Failed to save draft")?;

        Ok(draft)
    }

    /// Get draft by ID
    pub async fn get_draft(&self, draft_id: &str) -> Result<Option<CommunicationDraft>> {
        let row = sqlx::query_as::<_, DraftRow>(
            r#"
            SELECT * FROM communication_drafts WHERE id = ? AND status != 'archived'
            "#,
        )
        .bind(draft_id)
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to fetch draft")?;

        Ok(row.map(|r| r.into()))
    }

    /// List drafts with filtering
    pub async fn list_drafts(
        &self,
        filter: Option<CommunicationFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommunicationDraft>> {
        let mut query = String::from(
            "SELECT * FROM communication_drafts WHERE status != 'archived'",
        );
        let mut params: Vec<String> = Vec::new();

        if let Some(f) = filter {
            if let Some(lead_id) = &f.lead_id {
                query.push_str(" AND lead_id = ?");
                params.push(lead_id.clone());
            }
            if let Some(status) = &f.status {
                query.push_str(" AND status = ?");
                params.push(format!("{:?}", status).to_lowercase());
            }
            if let Some(risk) = &f.risk_level {
                query.push_str(" AND risk_level = ?");
                params.push(format!("{:?}", risk).to_lowercase());
            }
            if let Some(channel) = &f.channel {
                query.push_str(" AND channel = ?");
                params.push(format!("{:?}", channel).to_lowercase());
            }
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query_builder = sqlx::query_as::<_, DraftRow>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await
            .context("Failed to list drafts")?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get pending approvals for dashboard
    pub async fn get_pending_approvals(&self, limit: i64) -> Result<Vec<PendingApprovalSummary>> {
        let rows = sqlx::query_as::<_, PendingApprovalRow>(
            r#"
            SELECT d.id, d.lead_id, l.name as lead_name, d.channel, d.subject, 
                   d.risk_level, d.created_at, SUBSTR(d.content, 1, 100) as preview
            FROM communication_drafts d
            JOIN leads l ON d.lead_id = l.id
            WHERE d.status = 'pending'
            ORDER BY 
                CASE d.risk_level 
                    WHEN 'high' THEN 1 
                    WHEN 'medium' THEN 2 
                    ELSE 3 
                END,
                d.created_at ASC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch pending approvals")?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Review/approve a draft
    pub async fn review_draft(
        &self,
        draft_id: &str,
        reviewer: &str,
        request: ReviewDraftRequest,
    ) -> Result<Option<CommunicationDraft>> {
        let draft = self.get_draft(draft_id).await?;
        if draft.is_none() {
            return Ok(None);
        }
        let mut draft = draft.unwrap();

        match request.action {
            ReviewAction::Approve => {
                draft.approve(reviewer, request.notes);
            }
            ReviewAction::Reject => {
                if let Some(notes) = &request.notes {
                    draft.reject(reviewer, notes);
                } else {
                    draft.reject(reviewer, "No reason provided");
                }
            }
        }

        // Update in database
        sqlx::query(
            r#"
            UPDATE communication_drafts SET 
                status = ?,
                reviewed_by = ?,
                review_notes = ?,
                reviewed_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(format!("{:?}", draft.status).to_lowercase())
        .bind(&draft.reviewed_by)
        .bind(&draft.review_notes)
        .bind(draft.reviewed_at)
        .bind(Utc::now())
        .bind(draft_id)
        .execute(&self.db_pool)
        .await?;

        Ok(Some(draft))
    }

    /// Edit a draft
    pub async fn edit_draft(
        &self,
        draft_id: &str,
        edited_by: &str,
        new_content: String,
        reason: Option<&str>,
    ) -> Result<Option<CommunicationDraft>> {
        let draft = self.get_draft(draft_id).await?;
        if draft.is_none() {
            return Ok(None);
        }
        let draft = draft.unwrap();
        let old_content = draft.get_effective_content().to_string();

        // Create edit history
        let edit_history = DraftEditHistory {
            id: uuid::Uuid::new_v4().to_string(),
            draft_id: draft_id.to_string(),
            previous_content: old_content,
            new_content: new_content.clone(),
            edited_by: edited_by.to_string(),
            edit_reason: reason.map(|r| r.to_string()),
            created_at: Utc::now(),
        };

        sqlx::query(
            r#"
            INSERT INTO draft_edit_history (id, draft_id, previous_content, new_content, edited_by, edit_reason, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&edit_history.id)
        .bind(&edit_history.draft_id)
        .bind(&edit_history.previous_content)
        .bind(&edit_history.new_content)
        .bind(&edit_history.edited_by)
        .bind(&edit_history.edit_reason)
        .bind(edit_history.created_at)
        .execute(&self.db_pool)
        .await?;

        // Update draft with new content and reset to pending
        sqlx::query(
            r#"
            UPDATE communication_drafts SET 
                final_content = ?,
                status = 'pending',
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&new_content)
        .bind(Utc::now())
        .bind(draft_id)
        .execute(&self.db_pool)
        .await?;

        self.get_draft(draft_id).await
    }

    /// Mark draft as sent
    pub async fn mark_sent(&self, draft_id: &str, sent_by: &str) -> Result<Option<CommunicationDraft>> {
        let draft = self.get_draft(draft_id).await?;
        if draft.is_none() {
            return Ok(None);
        }
        let mut draft = draft.unwrap();

        if !draft.is_ready_to_send() {
            return Err(anyhow::anyhow!("Draft not approved"));
        }

        draft.mark_sent();

        // Update status
        sqlx::query(
            r#"
            UPDATE communication_drafts SET 
                status = 'sent',
                sent_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(draft.sent_at)
        .bind(Utc::now())
        .bind(draft_id)
        .execute(&self.db_pool)
        .await?;

        // Log to sent_communications
        let content = draft.get_effective_content().to_string();
        sqlx::query(
            r#"
            INSERT INTO sent_communications (id, draft_id, lead_id, channel, subject, content, sent_by, sent_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(&draft.id)
        .bind(&draft.lead_id)
        .bind(format!("{:?}", draft.channel).to_lowercase())
        .bind(&draft.subject)
        .bind(&content)
        .bind(sent_by)
        .bind(Utc::now())
        .execute(&self.db_pool)
        .await?;

        Ok(Some(draft))
    }

    /// Get templates
    pub async fn get_templates(&self) -> Result<Vec<CommunicationTemplate>> {
        let rows = sqlx::query_as::<_, TemplateRow>(
            r#"
            SELECT * FROM communication_templates 
            WHERE is_active = TRUE
            ORDER BY name
            "#,
        )
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch templates")?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get template by ID
    pub async fn get_template(&self, template_id: &str) -> Result<Option<CommunicationTemplate>> {
        let row = sqlx::query_as::<_, TemplateRow>(
            r#"
            SELECT * FROM communication_templates WHERE id = ? AND is_active = TRUE
            "#,
        )
        .bind(template_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }
}

// Database row types

#[derive(sqlx::FromRow)]
struct DraftRow {
    id: String,
    lead_id: String,
    requested_by: String,
    generated_by: String,
    channel: String,
    subject: Option<String>,
    content: String,
    risk_level: String,
    status: String,
    confidence: i32,
    risk_reason: Option<String>,
    reviewed_by: Option<String>,
    review_notes: Option<String>,
    reviewed_at: Option<chrono::DateTime<Utc>>,
    final_content: Option<String>,
    sent_at: Option<chrono::DateTime<Utc>>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<DraftRow> for CommunicationDraft {
    fn from(row: DraftRow) -> Self {
        CommunicationDraft {
            id: row.id,
            lead_id: row.lead_id,
            requested_by: row.requested_by,
            generated_by: row.generated_by,
            channel: parse_channel(&row.channel),
            subject: row.subject,
            content: row.content,
            risk_level: parse_risk_level(&row.risk_level),
            status: parse_approval_status(&row.status),
            confidence: row.confidence,
            risk_reason: row.risk_reason,
            reviewed_by: row.reviewed_by,
            review_notes: row.review_notes,
            reviewed_at: row.reviewed_at,
            final_content: row.final_content,
            sent_at: row.sent_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct PendingApprovalRow {
    id: String,
    lead_id: String,
    lead_name: String,
    channel: String,
    subject: Option<String>,
    risk_level: String,
    created_at: chrono::DateTime<Utc>,
    preview: String,
}

impl From<PendingApprovalRow> for PendingApprovalSummary {
    fn from(row: PendingApprovalRow) -> Self {
        PendingApprovalSummary {
            id: row.id,
            lead_id: row.lead_id,
            lead_name: row.lead_name,
            channel: parse_channel(&row.channel),
            subject: row.subject,
            risk_level: parse_risk_level(&row.risk_level),
            created_at: row.created_at,
            preview: row.preview,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TemplateRow {
    id: String,
    name: String,
    description: Option<String>,
    channel: String,
    subject_template: Option<String>,
    body_template: String,
    default_risk_level: String,
    variables: Option<String>,
    is_active: bool,
    created_by: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<TemplateRow> for CommunicationTemplate {
    fn from(row: TemplateRow) -> Self {
        CommunicationTemplate {
            id: row.id,
            name: row.name,
            description: row.description,
            channel: parse_channel(&row.channel),
            subject_template: row.subject_template,
            body_template: row.body_template,
            default_risk_level: parse_risk_level(&row.default_risk_level),
            variables: row.variables.and_then(|v| serde_json::from_str(&v).ok()),
            is_active: row.is_active,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

// Helper functions

fn parse_channel(s: &str) -> CommunicationChannel {
    match s {
        "email" => CommunicationChannel::Email,
        "sms" => CommunicationChannel::Sms,
        "phone" => CommunicationChannel::Phone,
        "in_person" => CommunicationChannel::InPerson,
        "video" => CommunicationChannel::Video,
        _ => CommunicationChannel::Email,
    }
}

fn parse_risk_level(s: &str) -> RiskLevel {
    match s {
        "low" => RiskLevel::Low,
        "medium" => RiskLevel::Medium,
        "high" => RiskLevel::High,
        _ => RiskLevel::Medium,
    }
}

fn parse_approval_status(s: &str) -> ApprovalStatus {
    match s {
        "draft" => ApprovalStatus::Draft,
        "pending" => ApprovalStatus::Pending,
        "auto_approved" => ApprovalStatus::AutoApproved,
        "approved" => ApprovalStatus::Approved,
        "rejected" => ApprovalStatus::Rejected,
        "sent" => ApprovalStatus::Sent,
        "archived" => ApprovalStatus::Archived,
        _ => ApprovalStatus::Draft,
    }
}
