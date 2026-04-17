//! Lead Management Service
//!
//! Business logic for lead CRUD operations, qualification, and routing.

use crate::models::{
    CreateLeadRequest, Lead, LeadActivity, LeadFilter, LeadQualification, LeadSource, LeadStatus,
    LeadSummary, QueuePriority, Timeline, UpdateLeadRequest,
};
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Pool, Sqlite};

/// Service for managing leads
pub struct LeadService {
    db_pool: Pool<Sqlite>,
}

impl LeadService {
    /// Create a new lead service
    pub fn new(db_pool: Pool<Sqlite>) -> Self {
        Self { db_pool }
    }

    /// Create a new lead
    pub async fn create_lead(&self, request: CreateLeadRequest) -> Result<Lead> {
        let lead = Lead::new(
            &request.name,
            request.email.as_deref(),
            request.phone.as_deref(),
            request.source,
            request.inquiry_type,
        );

        sqlx::query(
            r#"
            INSERT INTO leads (
                id, name, email, phone, preferred_contact, status, source, inquiry_type,
                property_type, budget_min, budget_max, desired_location, timeline, notes,
                is_urgent, qualification, assigned_agent_id, created_at, updated_at,
                first_contact_at, last_contact_at, contact_attempts
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&lead.id)
        .bind(&lead.name)
        .bind(&lead.email)
        .bind(&lead.phone)
        .bind(&request.preferred_contact)
        .bind(format!("{:?}", lead.status).to_lowercase())
        .bind(format!("{:?}", lead.source).to_lowercase())
        .bind(format!("{:?}", lead.inquiry_type).to_lowercase())
        .bind(request.property_type.map(|p| format!("{:?}", p).to_lowercase()))
        .bind(request.budget_min)
        .bind(request.budget_max)
        .bind(&request.desired_location)
        .bind(request.timeline.map(|t| format!("{:?}", t).to_lowercase()))
        .bind(&request.notes)
        .bind(lead.is_urgent)
        .bind(lead.qualification.as_ref().map(|q| q.to_string()))
        .bind(&lead.assigned_agent_id)
        .bind(lead.created_at)
        .bind(lead.updated_at)
        .bind(lead.first_contact_at)
        .bind(lead.last_contact_at)
        .bind(lead.contact_attempts)
        .execute(&self.db_pool)
        .await
        .context("Failed to create lead")?;

        // Log activity
        self.log_activity(&lead.id, "created", "Lead created", None::<&str>, None)
            .await?;

        Ok(lead)
    }

    /// Get lead by ID
    pub async fn get_lead(&self, lead_id: &str) -> Result<Option<Lead>> {
        let row = sqlx::query_as::<_, LeadRow>(
            r#"
            SELECT * FROM leads WHERE id = ? AND status != 'archived'
            "#,
        )
        .bind(lead_id)
        .fetch_optional(&self.db_pool)
        .await
        .context("Failed to fetch lead")?;

        Ok(row.map(|r| r.into()))
    }

    /// Update a lead - simplified to always update all fields
    pub async fn update_lead(
        &self,
        lead_id: &str,
        request: UpdateLeadRequest,
    ) -> Result<Option<Lead>> {
        // Get current lead
        let current = self.get_lead(lead_id).await?;
        if current.is_none() {
            return Ok(None);
        }

        // Build simple update query - always update provided fields
        let name = request.name.unwrap_or_else(|| current.as_ref().unwrap().name.clone());
        let email = request.email.unwrap_or_else(|| current.as_ref().unwrap().email.clone().unwrap_or_default());
        let phone = request.phone.unwrap_or_else(|| current.as_ref().unwrap().phone.clone().unwrap_or_default());
        let status = request.status.map(|s| format!("{:?}", s).to_lowercase())
            .unwrap_or_else(|| format!("{:?}", current.as_ref().unwrap().status).to_lowercase());
        let property_type = request.property_type.map(|p| format!("{:?}", p).to_lowercase())
            .unwrap_or_else(|| format!("{:?}", current.as_ref().unwrap().property_type).to_lowercase());
        let budget_min = request.budget_min.unwrap_or(current.as_ref().unwrap().budget_min.unwrap_or(0));
        let budget_max = request.budget_max.unwrap_or(current.as_ref().unwrap().budget_max.unwrap_or(0));
        let desired_location = request.desired_location.unwrap_or_else(|| current.as_ref().unwrap().desired_location.clone().unwrap_or_default());
        let timeline = request.timeline.map(|t| format!("{:?}", t).to_lowercase())
            .unwrap_or_else(|| format!("{:?}", current.as_ref().unwrap().timeline).to_lowercase());
        let notes = request.notes.unwrap_or_else(|| current.as_ref().unwrap().notes.clone().unwrap_or_default());
        let is_urgent = request.is_urgent.unwrap_or(current.as_ref().unwrap().is_urgent);
        let assigned_agent_id = request.assigned_agent_id.unwrap_or_else(|| current.as_ref().unwrap().assigned_agent_id.clone().unwrap_or_default());

        sqlx::query(
            r#"
            UPDATE leads SET 
                name = ?,
                email = ?,
                phone = ?,
                status = ?,
                property_type = ?,
                budget_min = ?,
                budget_max = ?,
                desired_location = ?,
                timeline = ?,
                notes = ?,
                is_urgent = ?,
                assigned_agent_id = ?,
                updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&name)
        .bind(&email)
        .bind(&phone)
        .bind(&status)
        .bind(&property_type)
        .bind(budget_min)
        .bind(budget_max)
        .bind(&desired_location)
        .bind(&timeline)
        .bind(&notes)
        .bind(is_urgent)
        .bind(&assigned_agent_id)
        .bind(Utc::now())
        .bind(lead_id)
        .execute(&self.db_pool)
        .await?;

        // Get updated lead
        let updated = self.get_lead(lead_id).await?;

        // Log activity
        self.log_activity(lead_id, "updated", "Lead updated", None::<&str>, None)
            .await?;

        Ok(updated)
    }

    /// List leads with filtering
    pub async fn list_leads(
        &self,
        filter: Option<LeadFilter>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LeadSummary>> {
        let mut query = String::from(
            "SELECT id, name, email, phone, status, source, inquiry_type, is_urgent, created_at, last_contact_at FROM leads WHERE status != 'archived'",
        );
        let mut params: Vec<String> = Vec::new();

        if let Some(f) = filter {
            if let Some(status) = &f.status {
                query.push_str(" AND status = ?");
                params.push(format!("{:?}", status).to_lowercase());
            }
            if let Some(source) = &f.source {
                query.push_str(" AND source = ?");
                params.push(format!("{:?}", source).to_lowercase());
            }
            if let Some(inquiry_type) = &f.inquiry_type {
                query.push_str(" AND inquiry_type = ?");
                params.push(format!("{:?}", inquiry_type).to_lowercase());
            }
            if let Some(agent_id) = &f.assigned_agent_id {
                query.push_str(" AND assigned_agent_id = ?");
                params.push(agent_id.clone());
            }
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

        let mut query_builder = sqlx::query_as::<_, LeadSummaryRow>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await
            .context("Failed to list leads")?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Delete (archive) a lead
    pub async fn archive_lead(&self, lead_id: &str) -> Result<bool> {
        let result = sqlx::query("UPDATE leads SET status = 'archived' WHERE id = ?")
            .bind(lead_id)
            .execute(&self.db_pool)
            .await?;

        if result.rows_affected() > 0 {
            self.log_activity(lead_id, "archived", "Lead archived", None::<&str>, None)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Qualify a lead with AI scoring
    pub async fn qualify_lead(
        &self,
        lead_id: &str,
        qualification: LeadQualification,
    ) -> Result<Option<Lead>> {
        let lead = self.get_lead(lead_id).await?;
        if lead.is_none() {
            return Ok(None);
        }

        let mut lead = lead.unwrap();
        lead.set_qualification(qualification.clone());

        // Update in database
        sqlx::query(
            r#"
            UPDATE leads SET 
                status = 'qualified',
                qualification = ?,
                is_urgent = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(serde_json::to_string(&qualification)?)
        .bind(qualification.recommended_queue == QueuePriority::Priority)
        .bind(Utc::now())
        .bind(lead_id)
        .execute(&self.db_pool)
        .await?;

        // Log activity
        self.log_activity(
            lead_id,
            "qualified",
            &format!("Lead qualified with score {}", qualification.score),
            None::<&str>,
            Some(serde_json::json!({"score": qualification.score, "queue": format!("{:?}", qualification.recommended_queue)})),
        )
        .await?;

        self.get_lead(lead_id).await
    }

    /// Mark lead as contacted
    pub async fn mark_contacted(&self, lead_id: &str) -> Result<Option<Lead>> {
        let lead = self.get_lead(lead_id).await?;
        if lead.is_none() {
            return Ok(None);
        }

        let mut lead = lead.unwrap();
        lead.mark_contacted();

        sqlx::query(
            r#"
            UPDATE leads SET 
                status = 'contacted',
                first_contact_at = ?,
                last_contact_at = ?,
                contact_attempts = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(lead.first_contact_at)
        .bind(lead.last_contact_at)
        .bind(lead.contact_attempts)
        .bind(Utc::now())
        .bind(lead_id)
        .execute(&self.db_pool)
        .await?;

        self.log_activity(lead_id, "contacted", "Lead contacted", None::<&str>, None)
            .await?;

        self.get_lead(lead_id).await
    }

    /// Get lead activities
    pub async fn get_activities(&self, lead_id: &str) -> Result<Vec<LeadActivity>> {
        let rows = sqlx::query_as::<_, LeadActivityRow>(
            r#"
            SELECT * FROM lead_activities 
            WHERE lead_id = ? 
            ORDER BY created_at DESC
            "#,
        )
        .bind(lead_id)
        .fetch_all(&self.db_pool)
        .await
        .context("Failed to fetch activities")?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Log an activity
    async fn log_activity(
        &self,
        lead_id: &str,
        activity_type: &str,
        description: &str,
        performed_by: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let activity = LeadActivity::new(
            lead_id,
            activity_type,
            description,
            performed_by,
            metadata,
        );

        sqlx::query(
            r#"
            INSERT INTO lead_activities (id, lead_id, activity_type, description, performed_by, metadata, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&activity.id)
        .bind(&activity.lead_id)
        .bind(&activity.activity_type)
        .bind(&activity.description)
        .bind(&activity.performed_by)
        .bind(activity.metadata.as_ref().map(|m| m.to_string()))
        .bind(activity.created_at)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

// Database row types for mapping

#[derive(sqlx::FromRow)]
struct LeadRow {
    id: String,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    preferred_contact: Option<String>,
    status: String,
    source: String,
    inquiry_type: String,
    property_type: Option<String>,
    budget_min: Option<i64>,
    budget_max: Option<i64>,
    desired_location: Option<String>,
    timeline: Option<String>,
    notes: Option<String>,
    is_urgent: bool,
    qualification: Option<String>,
    assigned_agent_id: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    first_contact_at: Option<chrono::DateTime<Utc>>,
    last_contact_at: Option<chrono::DateTime<Utc>>,
    contact_attempts: i32,
}

impl From<LeadRow> for Lead {
    fn from(row: LeadRow) -> Self {
        Lead {
            id: row.id,
            name: row.name,
            email: row.email,
            phone: row.phone,
            preferred_contact: row.preferred_contact,
            status: parse_status(&row.status),
            source: parse_source(&row.source),
            inquiry_type: parse_inquiry_type(&row.inquiry_type),
            property_type: row.property_type.as_deref().map(parse_property_type),
            budget_min: row.budget_min,
            budget_max: row.budget_max,
            desired_location: row.desired_location,
            timeline: row.timeline.as_deref().map(parse_timeline).unwrap_or_default(),
            notes: row.notes,
            is_urgent: row.is_urgent,
            qualification: row.qualification.and_then(|q| serde_json::from_str(&q).ok()),
            assigned_agent_id: row.assigned_agent_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            first_contact_at: row.first_contact_at,
            last_contact_at: row.last_contact_at,
            contact_attempts: row.contact_attempts,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LeadSummaryRow {
    id: String,
    name: String,
    email: Option<String>,
    phone: Option<String>,
    status: String,
    source: String,
    inquiry_type: String,
    is_urgent: bool,
    created_at: chrono::DateTime<Utc>,
    last_contact_at: Option<chrono::DateTime<Utc>>,
}

impl From<LeadSummaryRow> for LeadSummary {
    fn from(row: LeadSummaryRow) -> Self {
        LeadSummary {
            id: row.id,
            name: row.name,
            email: row.email,
            phone: row.phone,
            status: parse_status(&row.status),
            source: parse_source(&row.source),
            inquiry_type: parse_inquiry_type(&row.inquiry_type),
            qualification_score: 0, // Would need to fetch full lead
            is_urgent: row.is_urgent,
            created_at: row.created_at,
            last_contact_at: row.last_contact_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LeadActivityRow {
    id: String,
    lead_id: String,
    activity_type: String,
    description: String,
    performed_by: Option<String>,
    metadata: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

impl From<LeadActivityRow> for LeadActivity {
    fn from(row: LeadActivityRow) -> Self {
        LeadActivity {
            id: row.id,
            lead_id: row.lead_id,
            activity_type: row.activity_type,
            description: row.description,
            performed_by: row.performed_by,
            metadata: row.metadata.and_then(|m| serde_json::from_str(&m).ok()),
            created_at: row.created_at,
        }
    }
}

// Helper functions for parsing enums from strings

fn parse_status(s: &str) -> LeadStatus {
    match s {
        "new" => LeadStatus::New,
        "qualified" => LeadStatus::Qualified,
        "contacted" => LeadStatus::Contacted,
        "converted" => LeadStatus::Converted,
        "lost" => LeadStatus::Lost,
        "archived" => LeadStatus::Archived,
        _ => LeadStatus::New,
    }
}

fn parse_source(s: &str) -> LeadSource {
    match s {
        "website" => LeadSource::Website,
        "referral" => LeadSource::Referral,
        "portal" => LeadSource::Portal,
        "email" => LeadSource::Email,
        "sms" => LeadSource::Sms,
        "walk_in" => LeadSource::WalkIn,
        "social_media" => LeadSource::SocialMedia,
        "paid_ad" => LeadSource::PaidAd,
        _ => LeadSource::Other,
    }
}

fn parse_inquiry_type(s: &str) -> crate::models::InquiryType {
    match s {
        "buyer" => crate::models::InquiryType::Buyer,
        "seller" => crate::models::InquiryType::Seller,
        "renter" => crate::models::InquiryType::Renter,
        "landlord" => crate::models::InquiryType::Landlord,
        _ => crate::models::InquiryType::General,
    }
}

fn parse_property_type(s: &str) -> crate::models::PropertyType {
    match s {
        "detached" => crate::models::PropertyType::Detached,
        "semi_detached" => crate::models::PropertyType::SemiDetached,
        "townhouse" => crate::models::PropertyType::Townhouse,
        "condo" => crate::models::PropertyType::Condo,
        "commercial" => crate::models::PropertyType::Commercial,
        "multi_family" => crate::models::PropertyType::MultiFamily,
        "land" => crate::models::PropertyType::Land,
        _ => crate::models::PropertyType::Any,
    }
}

fn parse_timeline(s: &str) -> Timeline {
    match s {
        "immediate" => Timeline::Immediate,
        "short_term" => Timeline::ShortTerm,
        "medium_term" => Timeline::MediumTerm,
        "long_term" => Timeline::LongTerm,
        _ => Timeline::Browsing,
    }
}
