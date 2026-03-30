use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub status: LeadStatus,
    pub score: i32,
    pub source: LeadSource,
    pub property_interest: Option<String>,
    pub budget: Option<String>,
    pub timeline: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LeadStatus {
    New,
    Qualified,
    Contacted,
    Negotiating,
    ClosedWon,
    ClosedLost,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LeadSource {
    Website,
    Referral,
    Portal,
    Email,
    SMS,
    WalkIn,
    Other,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CmaReport {
    pub id: String,
    pub subject_address: String,
    pub property_type: String,
    pub status: CmaStatus,
    pub price_recommendation_low: Option<f64>,
    pub price_recommendation_mid: Option<f64>,
    pub price_recommendation_high: Option<f64>,
    pub confidence: i32,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CmaStatus {
    Draft,
    UnderReview,
    Approved,
    Rejected,
    Archived,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComparableProperty {
    pub address: String,
    pub mls_number: String,
    pub price: i64,
    pub price_type: String,
    pub property_type: String,
    pub bedrooms: i32,
    pub bathrooms: i32,
    pub square_feet: i32,
    pub days_on_market: i32,
    pub distance_km: f64,
    pub match_confidence: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MarketTrends {
    pub area: String,
    pub period: String,
    pub average_price: String,
    pub median_price: String,
    pub price_per_sqft: String,
    pub median_days_on_market: i32,
    pub inventory_level: i32,
    pub sales_volume: i32,
    pub price_trend_direction: String,
    pub market_status: MarketStatus,
    pub generated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MarketStatus {
    SellersMarket,
    BuyersMarket,
    BalancedMarket,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CommunicationDraft {
    pub id: String,
    pub draft_type: DraftType,
    pub recipient: String,
    pub subject: String,
    pub content: String,
    pub risk_level: RiskLevel,
    pub status: DraftStatus,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DraftType {
    Email,
    SMS,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DraftStatus {
    Draft,
    PendingApproval,
    Approved,
    Sent,
    Rejected,
}
