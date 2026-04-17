use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub status: String,
    pub score: i32,
    pub source: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CmaReport {
    pub id: String,
    pub subject_address: String,
    pub property_type: String,
    pub price_recommendation_low: Option<f64>,
    pub price_recommendation_mid: Option<f64>,
    pub price_recommendation_high: Option<f64>,
    pub confidence: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketTrend {
    pub area: String,
    pub avg_price: f64,
    pub yoy_change: f64,
    pub avg_dom: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommunicationDraft {
    pub id: String,
    pub draft_type: String,
    pub recipient: String,
    pub subject: String,
    pub content: String,
    pub risk_level: String,
    pub status: String,
}
