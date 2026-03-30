use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    pub id: Option<i64>,
    pub source: String,
    pub source_id: String,
    pub address: String,
    pub municipality: String,
    pub postal_code: Option<String>,
    pub price: Option<f64>,
    pub sold_price: Option<f64>,
    pub property_type: String,
    pub bedrooms: Option<f64>,
    pub bathrooms: Option<f64>,
    pub square_feet: Option<f64>,
    pub listing_date: Option<DateTime<Utc>>,
    pub sold_date: Option<DateTime<Utc>>,
    pub days_on_market: Option<i64>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub total_listings: i64,
    pub active_listings: i64,
    pub sold_listings: i64,
    pub sources: Vec<String>,
    pub last_updated: DateTime<Utc>,
    pub earliest_date: DateTime<Utc>,
    pub latest_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MarketStats {
    pub count: i64,
    pub avg_price: f64,
    pub median_price: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub avg_price_per_sqft: f64,
    pub avg_days_on_market: f64,
}

#[derive(Debug, Clone)]
pub struct Trend {
    pub month: String,
    pub property_type: String,
    pub avg_price: f64,
    pub count: i64,
}

#[derive(Debug, Clone)]
pub struct MunicipalityComparison {
    pub municipality: String,
    pub avg_price: f64,
    pub median_price: f64,
    pub count: i64,
}

impl Listing {
    pub fn new() -> Self {
        Self {
            id: None,
            source: String::new(),
            source_id: String::new(),
            address: String::new(),
            municipality: String::new(),
            postal_code: None,
            price: None,
            sold_price: None,
            property_type: String::new(),
            bedrooms: None,
            bathrooms: None,
            square_feet: None,
            listing_date: None,
            sold_date: None,
            days_on_market: None,
            description: None,
            url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
