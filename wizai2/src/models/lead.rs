//! Lead data models for AI Real Estate Team
//!
//! Defines structures for lead capture, qualification, and management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Lead status in the workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum LeadStatus {
    /// New lead, not yet processed
    New,
    /// AI has qualified the lead
    Qualified,
    /// Human agent has made contact
    Contacted,
    /// Lead converted to client/opportunity
    Converted,
    /// Lead not interested or invalid
    Lost,
    /// Archived for historical record
    Archived,
}

impl Default for LeadStatus {
    fn default() -> Self {
        LeadStatus::New
    }
}

/// Source of the lead
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum LeadSource {
    /// Website contact form
    Website,
    /// Referral from existing client
    Referral,
    /// Real estate portal (e.g., Realtor.ca)
    Portal,
    /// Email inquiry
    Email,
    /// SMS/text message
    Sms,
    /// Walk-in to office
    WalkIn,
    /// Social media
    SocialMedia,
    /// Paid advertising
    PaidAd,
    /// Other sources
    Other,
}

impl Default for LeadSource {
    fn default() -> Self {
        LeadSource::Other
    }
}

/// Type of real estate inquiry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum InquiryType {
    /// Looking to buy a property
    Buyer,
    /// Looking to sell a property
    Seller,
    /// Looking to rent
    Renter,
    /// Looking to lease out property
    Landlord,
    /// General inquiry
    General,
}

impl Default for InquiryType {
    fn default() -> Self {
        InquiryType::General
    }
}

/// Property type preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum PropertyType {
    /// Single family detached home
    Detached,
    /// Semi-detached home
    SemiDetached,
    /// Townhouse/row house
    Townhouse,
    /// Condominium apartment
    Condo,
    /// Commercial property
    Commercial,
    /// Multi-family/residential investment
    MultiFamily,
    /// Vacant land
    Land,
    /// No preference
    Any,
}

impl Default for PropertyType {
    fn default() -> Self {
        PropertyType::Any
    }
}

/// Timeline for purchase/sale
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum Timeline {
    /// Immediate (within 30 days)
    Immediate,
    /// Short term (1-3 months)
    ShortTerm,
    /// Medium term (3-6 months)
    MediumTerm,
    /// Long term (6+ months)
    LongTerm,
    /// Just browsing, no timeline
    Browsing,
}

impl Default for Timeline {
    fn default() -> Self {
        Timeline::Browsing
    }
}

/// Lead qualification data calculated by AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadQualification {
    /// Overall score 0-10
    pub score: i32,
    /// Budget clarity score (0-3)
    pub budget_clarity: i32,
    /// Timeline clarity score (0-3)
    pub timeline_clarity: i32,
    /// Location specificity score (0-2)
    pub location_specificity: i32,
    /// Motivation/urgency score (0-2)
    pub motivation: i32,
    /// AI-generated rationale for scoring
    pub rationale: String,
    /// Recommended priority queue
    pub recommended_queue: QueuePriority,
    /// Timestamp of qualification
    pub qualified_at: DateTime<Utc>,
}

/// Priority queue for lead routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum QueuePriority {
    /// Hot lead, immediate attention needed
    Priority,
    /// Standard follow-up queue
    Standard,
    /// Nurture queue for long-term leads
    Nurture,
}

impl Default for QueuePriority {
    fn default() -> Self {
        QueuePriority::Standard
    }
}

/// Complete lead record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Lead {
    /// Unique identifier
    pub id: String,
    /// Lead name
    pub name: String,
    /// Email address
    pub email: Option<String>,
    /// Phone number
    pub phone: Option<String>,
    /// Preferred contact method
    pub preferred_contact: Option<String>,
    /// Current status in workflow
    pub status: LeadStatus,
    /// Source of the lead
    pub source: LeadSource,
    /// Type of inquiry (buyer, seller, etc.)
    pub inquiry_type: InquiryType,
    /// Desired property type
    pub property_type: Option<PropertyType>,
    /// Budget range (min)
    pub budget_min: Option<i64>,
    /// Budget range (max)
    pub budget_max: Option<i64>,
    /// Desired location/area
    pub desired_location: Option<String>,
    /// Timeline for action
    pub timeline: Timeline,
    /// Free-form notes from lead
    pub notes: Option<String>,
    /// Urgency flag set by AI
    pub is_urgent: bool,
    /// Qualification data (JSON stored in DB)
    pub qualification: Option<serde_json::Value>,
    /// Assigned agent (if any)
    pub assigned_agent_id: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// When lead was first contacted
    pub first_contact_at: Option<DateTime<Utc>>,
    /// When lead was last contacted
    pub last_contact_at: Option<DateTime<Utc>>,
    /// Number of contact attempts
    pub contact_attempts: i32,
}

impl Lead {
    /// Create a new lead from basic info
    pub fn new(
        name: impl Into<String>,
        email: Option<impl Into<String>>,
        phone: Option<impl Into<String>>,
        source: LeadSource,
        inquiry_type: InquiryType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            email: email.map(|e| e.into()),
            phone: phone.map(|p| p.into()),
            preferred_contact: None,
            status: LeadStatus::New,
            source,
            inquiry_type,
            property_type: None,
            budget_min: None,
            budget_max: None,
            desired_location: None,
            timeline: Timeline::Browsing,
            notes: None,
            is_urgent: false,
            qualification: None,
            assigned_agent_id: None,
            created_at: now,
            updated_at: now,
            first_contact_at: None,
            last_contact_at: None,
            contact_attempts: 0,
        }
    }

    /// Update qualification data
    pub fn set_qualification(&mut self, qualification: LeadQualification) {
        self.qualification = Some(serde_json::to_value(qualification).unwrap_or_default());
        self.updated_at = Utc::now();
    }

    /// Check if lead has complete contact info
    pub fn has_complete_contact(&self) -> bool {
        (!self.name.is_empty()) && (self.email.is_some() || self.phone.is_some())
    }

    /// Get qualification score (0-10), defaults to 0 if not qualified
    pub fn get_qualification_score(&self) -> i32 {
        self.qualification
            .as_ref()
            .and_then(|q| q.get("score").and_then(|s| s.as_i64()).map(|s| s as i32))
            .unwrap_or(0)
    }

    /// Check if this is a hot lead (score >= 8)
    pub fn is_hot_lead(&self) -> bool {
        self.get_qualification_score() >= 8
    }

    /// Mark as contacted and update timestamps
    pub fn mark_contacted(&mut self) {
        if self.first_contact_at.is_none() {
            self.first_contact_at = Some(Utc::now());
        }
        self.last_contact_at = Some(Utc::now());
        self.contact_attempts += 1;
        self.status = LeadStatus::Contacted;
        self.updated_at = Utc::now();
    }

    /// Get budget display string
    pub fn get_budget_display(&self) -> String {
        match (self.budget_min, self.budget_max) {
            (Some(min), Some(max)) => format!("${} - ${}", min, max),
            (Some(min), None) => format!("${}+", min),
            (None, Some(max)) => format!("Up to ${}", max),
            (None, None) => "Not specified".to_string(),
        }
    }
}

/// Request to create a new lead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLeadRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub preferred_contact: Option<String>,
    pub source: LeadSource,
    pub inquiry_type: InquiryType,
    pub property_type: Option<PropertyType>,
    pub budget_min: Option<i64>,
    pub budget_max: Option<i64>,
    pub desired_location: Option<String>,
    pub timeline: Option<Timeline>,
    pub notes: Option<String>,
}

/// Request to update a lead
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLeadRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub preferred_contact: Option<String>,
    pub status: Option<LeadStatus>,
    pub property_type: Option<PropertyType>,
    pub budget_min: Option<i64>,
    pub budget_max: Option<i64>,
    pub desired_location: Option<String>,
    pub timeline: Option<Timeline>,
    pub notes: Option<String>,
    pub is_urgent: Option<bool>,
    pub assigned_agent_id: Option<String>,
}

/// Lead filtering parameters for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadFilter {
    pub status: Option<LeadStatus>,
    pub source: Option<LeadSource>,
    pub inquiry_type: Option<InquiryType>,
    pub assigned_agent_id: Option<String>,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub is_urgent: Option<bool>,
}

/// Lead activity/event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LeadActivity {
    pub id: String,
    pub lead_id: String,
    pub activity_type: String,
    pub description: String,
    pub performed_by: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl LeadActivity {
    pub fn new(
        lead_id: impl Into<String>,
        activity_type: impl Into<String>,
        description: impl Into<String>,
        performed_by: Option<impl Into<String>>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            lead_id: lead_id.into(),
            activity_type: activity_type.into(),
            description: description.into(),
            performed_by: performed_by.map(|p| p.into()),
            metadata,
            created_at: Utc::now(),
        }
    }
}

/// Lead summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadSummary {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: LeadStatus,
    pub source: LeadSource,
    pub inquiry_type: InquiryType,
    pub qualification_score: i32,
    pub is_urgent: bool,
    pub created_at: DateTime<Utc>,
    pub last_contact_at: Option<DateTime<Utc>>,
}

impl From<Lead> for LeadSummary {
    fn from(lead: Lead) -> Self {
        // Calculate qualification score BEFORE moving fields
        let score = lead.get_qualification_score();
        Self {
            id: lead.id,
            name: lead.name,
            email: lead.email,
            phone: lead.phone,
            status: lead.status,
            source: lead.source,
            inquiry_type: lead.inquiry_type,
            qualification_score: score,
            is_urgent: lead.is_urgent,
            created_at: lead.created_at,
            last_contact_at: lead.last_contact_at,
        }
    }
}
