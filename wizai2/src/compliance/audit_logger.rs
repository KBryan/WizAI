//! Audit logging for compliance
//!
//! Immutable audit trail for all AI actions and human approvals.

use crate::models::{ApprovalStatus, RiskLevel};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

/// Audit record for tracking actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// Unique identifier
    pub id: String,
    /// Entity type (lead, communication, cma, etc.)
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Action type (created, updated, drafted, approved, etc.)
    pub action_type: String,
    /// Human-readable description
    pub action_description: String,
    /// Who performed the action (user/agent ID)
    pub performed_by: String,
    /// Type of performer (human or ai)
    pub performed_by_type: PerformerType,
    /// Risk level if applicable
    pub risk_level: Option<RiskLevel>,
    /// Previous values (for updates)
    pub old_values: Option<serde_json::Value>,
    /// New values
    pub new_values: Option<serde_json::Value>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
    /// Hash of previous record (for chain)
    pub previous_hash: Option<String>,
    /// Hash of this record
    pub record_hash: String,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Type of performer
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformerType {
    Human,
    AI,
}

/// Audit logger for compliance tracking
pub struct AuditLogger {
    db_pool: Pool<Sqlite>,
    last_record_hash: Option<String>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(db_pool: Pool<Sqlite>) -> Self {
        Self {
            db_pool,
            last_record_hash: None,
        }
    }

    /// Log a lead creation event
    pub async fn log_lead_created(
        &mut self,
        lead_id: &str,
        performed_by: &str,
        performed_by_type: PerformerType,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let record = self.create_record(
            "lead",
            lead_id,
            "created",
            "Lead record created",
            performed_by,
            performed_by_type,
            None,
            None,
            None,
            metadata,
        );
        self.save_record(&record).await
    }

    /// Log a lead qualification event
    pub async fn log_lead_qualified(
        &mut self,
        lead_id: &str,
        performed_by: &str,
        performed_by_type: PerformerType,
        old_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
    ) -> Result<()> {
        let record = self.create_record(
            "lead",
            lead_id,
            "qualified",
            "Lead qualified by AI",
            performed_by,
            performed_by_type,
            None,
            old_values,
            new_values,
            None,
        );
        self.save_record(&record).await
    }

    /// Log a communication draft event
    pub async fn log_draft_created(
        &mut self,
        draft_id: &str,
        lead_id: &str,
        performed_by: &str,
        risk_level: RiskLevel,
    ) -> Result<()> {
        let record = self.create_record(
            "communication",
            draft_id,
            "drafted",
            &format!("Communication draft created for lead {}", lead_id),
            performed_by,
            PerformerType::AI,
            Some(risk_level),
            None,
            None,
            Some(json!({"lead_id": lead_id})),
        );
        self.save_record(&record).await
    }

    /// Log a draft approval event
    pub async fn log_draft_approved(
        &mut self,
        draft_id: &str,
        reviewed_by: &str,
        notes: Option<&str>,
    ) -> Result<()> {
        let record = self.create_record(
            "communication",
            draft_id,
            "approved",
            &format!("Draft approved by {}", reviewed_by),
            reviewed_by,
            PerformerType::Human,
            None,
            Some(json!({"status": "pending"})),
            Some(json!({"status": "approved", "review_notes": notes})),
            None,
        );
        self.save_record(&record).await
    }

    /// Log a draft rejection event
    pub async fn log_draft_rejected(
        &mut self,
        draft_id: &str,
        reviewed_by: &str,
        reason: &str,
    ) -> Result<()> {
        let record = self.create_record(
            "communication",
            draft_id,
            "rejected",
            &format!("Draft rejected by {}: {}", reviewed_by, reason),
            reviewed_by,
            PerformerType::Human,
            None,
            Some(json!({"status": "pending"})),
            Some(json!({"status": "rejected", "reason": reason})),
            None,
        );
        self.save_record(&record).await
    }

    /// Log a communication sent event
    pub async fn log_communication_sent(
        &mut self,
        draft_id: &str,
        sent_by: &str,
    ) -> Result<()> {
        let record = self.create_record(
            "communication",
            draft_id,
            "sent",
            "Communication sent to client",
            sent_by,
            PerformerType::Human,
            None,
            Some(json!({"status": "approved"})),
            Some(json!({"status": "sent"})),
            None,
        );
        self.save_record(&record).await
    }

    /// Log a CMA creation event
    pub async fn log_cma_created(
        &mut self,
        cma_id: &str,
        performed_by: &str,
        confidence: i32,
    ) -> Result<()> {
        let record = self.create_record(
            "cma",
            cma_id,
            "created",
            &format!("CMA report generated (confidence: {}%)", confidence),
            performed_by,
            PerformerType::AI,
            Some(RiskLevel::High),
            None,
            None,
            Some(json!({"confidence": confidence})),
        );
        self.save_record(&record).await
    }

    /// Log a CMA approval event
    pub async fn log_cma_reviewed(
        &mut self,
        cma_id: &str,
        reviewed_by: &str,
        status: ApprovalStatus,
    ) -> Result<()> {
        let action = match status {
            ApprovalStatus::Approved => "approved",
            ApprovalStatus::Rejected => "rejected",
            _ => "reviewed",
        };
        let record = self.create_record(
            "cma",
            cma_id,
            action,
            &format!("CMA {} by {}", action, reviewed_by),
            reviewed_by,
            PerformerType::Human,
            Some(RiskLevel::High),
            Some(json!({"status": "draft"})),
            Some(json!({"status": format!("{:?}", status).to_lowercase()})),
            None,
        );
        self.save_record(&record).await
    }

    /// Create a new audit record
    fn create_record(
        &self,
        entity_type: &str,
        entity_id: &str,
        action_type: &str,
        action_description: &str,
        performed_by: &str,
        performed_by_type: PerformerType,
        risk_level: Option<RiskLevel>,
        old_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
        metadata: Option<serde_json::Value>,
    ) -> AuditRecord {
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        
        // Calculate hash of this record
        let hash_input = format!(
            "{}:{}:{}:{}:{}:{:?}:{}",
            id, entity_type, entity_id, action_type, performed_by, performed_by_type, now
        );
        let record_hash = format!("{:x}", Sha256::digest(hash_input.as_bytes()));

        AuditRecord {
            id,
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            action_type: action_type.to_string(),
            action_description: action_description.to_string(),
            performed_by: performed_by.to_string(),
            performed_by_type,
            risk_level,
            old_values,
            new_values,
            metadata,
            previous_hash: self.last_record_hash.clone(),
            record_hash: record_hash.clone(),
            created_at: now,
        }
    }

    /// Save record to database
    async fn save_record(&mut self, record: &AuditRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, entity_type, entity_id, action_type, action_description,
                performed_by, performed_by_type, risk_level, old_values, new_values,
                metadata, previous_hash, record_hash, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&record.id)
        .bind(&record.entity_type)
        .bind(&record.entity_id)
        .bind(&record.action_type)
        .bind(&record.action_description)
        .bind(&record.performed_by)
        .bind(format!("{:?}", record.performed_by_type).to_lowercase())
        .bind(record.risk_level.map(|r| format!("{:?}", r).to_lowercase()))
        .bind(record.old_values.as_ref().map(|v| v.to_string()))
        .bind(record.new_values.as_ref().map(|v| v.to_string()))
        .bind(record.metadata.as_ref().map(|v| v.to_string()))
        .bind(&record.previous_hash)
        .bind(&record.record_hash)
        .bind(record.created_at)
        .execute(&self.db_pool)
        .await?;

        // Update the chain
        self.last_record_hash = Some(record.record_hash.clone());
        Ok(())
    }

    /// Query audit logs by entity
    pub async fn query_by_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<AuditRecord>> {
        let rows = sqlx::query_as::<_, AuditRecordRow>(
            r#"
            SELECT * FROM audit_logs 
            WHERE entity_type = ? AND entity_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Query by date range
    pub async fn query_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditRecord>> {
        let rows = sqlx::query_as::<_, AuditRecordRow>(
            r#"
            SELECT * FROM audit_logs 
            WHERE created_at BETWEEN ? AND ?
            ORDER BY created_at DESC
            "#
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Query high-risk actions
    pub async fn query_high_risk_actions(&self) -> Result<Vec<AuditRecord>> {
        let rows = sqlx::query_as::<_, AuditRecordRow>(
            r#"
            SELECT * FROM audit_logs 
            WHERE risk_level = 'high'
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Generate compliance report for period
    pub async fn generate_compliance_report(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<ComplianceReport> {
        // Get all actions in period
        let actions = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM audit_logs 
            WHERE created_at BETWEEN ? AND ?
            "#
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.db_pool)
        .await?;

        let total_actions = actions.0;

        // Get high-risk count
        let high_risk = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM audit_logs 
            WHERE created_at BETWEEN ? AND ? AND risk_level = 'high'
            "#
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.db_pool)
        .await?;

        let high_risk_count = high_risk.0;

        // Get approved count (for approval rate)
        let approved = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM audit_logs 
            WHERE created_at BETWEEN ? AND ? AND action_type = 'approved'
            "#
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.db_pool)
        .await?;

        let approved_count = approved.0;

        // Get rejected count
        let rejected = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*) FROM audit_logs 
            WHERE created_at BETWEEN ? AND ? AND action_type = 'rejected'
            "#
        )
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.db_pool)
        .await?;

        let rejected_count = rejected.0;

        let total_reviewed = approved_count + rejected_count;
        let approval_rate = if total_reviewed > 0 {
            (approved_count as f64 / total_reviewed as f64) * 100.0
        } else {
            0.0
        };

        Ok(ComplianceReport {
            period_start,
            period_end,
            total_ai_actions: total_actions,
            high_risk_actions: high_risk_count,
            approved_count,
            rejected_count,
            approval_rate,
            generated_at: Utc::now(),
        })
    }
}

/// Database row for audit record
#[derive(sqlx::FromRow)]
struct AuditRecordRow {
    id: String,
    entity_type: String,
    entity_id: String,
    action_type: String,
    action_description: String,
    performed_by: String,
    performed_by_type: String,
    risk_level: Option<String>,
    old_values: Option<String>,
    new_values: Option<String>,
    metadata: Option<String>,
    previous_hash: Option<String>,
    record_hash: String,
    created_at: DateTime<Utc>,
}

impl From<AuditRecordRow> for AuditRecord {
    fn from(row: AuditRecordRow) -> Self {
        AuditRecord {
            id: row.id,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            action_type: row.action_type,
            action_description: row.action_description,
            performed_by: row.performed_by,
            performed_by_type: match row.performed_by_type.as_str() {
                "human" => PerformerType::Human,
                _ => PerformerType::AI,
            },
            risk_level: row.risk_level.and_then(|r| match r.as_str() {
                "low" => Some(RiskLevel::Low),
                "medium" => Some(RiskLevel::Medium),
                "high" => Some(RiskLevel::High),
                _ => None,
            }),
            old_values: row.old_values.and_then(|v| serde_json::from_str(&v).ok()),
            new_values: row.new_values.and_then(|v| serde_json::from_str(&v).ok()),
            metadata: row.metadata.and_then(|v| serde_json::from_str(&v).ok()),
            previous_hash: row.previous_hash,
            record_hash: row.record_hash,
            created_at: row.created_at,
        }
    }
}

/// Compliance report structure
#[derive(Debug, Clone)]
pub struct ComplianceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_ai_actions: i64,
    pub high_risk_actions: i64,
    pub approved_count: i64,
    pub rejected_count: i64,
    pub approval_rate: f64,
    pub generated_at: DateTime<Utc>,
}
