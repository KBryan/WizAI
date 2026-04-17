//! API request and response models for Market Research endpoints
//!
//! These models define the structure of incoming requests and outgoing responses
//! for the market research REST API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>, code: Option<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                message: message.into(),
                code,
            }),
        }
    }
}

/// API error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub code: Option<String>,
}

// CMA API Models

/// Request to generate a new CMA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCmaRequest {
    pub address: String,
    pub property_type: String,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub square_feet: Option<i32>,
}

/// Response after initiating CMA generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCmaResponse {
    pub cma_id: String,
    pub status: CmaJobStatus,
    pub estimated_completion: String,
}

/// CMA job status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CmaJobStatus {
    Queued,
    Processing,
    AnalyzingComparables,
    GeneratingReport,
    Completed,
    Failed,
}

/// CMA status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmaStatusResponse {
    pub cma_id: String,
    pub status: CmaJobStatus,
    pub progress: i32,
    pub message: String,
}

/// Query parameters for listing CMAs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListCmasQuery {
    pub status: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
}

/// Paginated list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

// Comparables API Models

/// Query parameters for searching comparables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparablesQuery {
    pub address: String,
    pub radius: Option<f64>,
    pub property_type: Option<String>,
    pub limit: Option<i32>,
}

/// Response for comparables search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparablesResponse {
    pub target_property: TargetProperty,
    pub comparables: Vec<ComparableApiModel>,
    pub count: i32,
}

/// Target property information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetProperty {
    pub address: String,
    pub property_type: Option<String>,
    pub bedrooms: Option<i32>,
    pub bathrooms: Option<i32>,
    pub square_feet: Option<i32>,
}

/// Comparable property API model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparableApiModel {
    pub address: String,
    pub mls_number: String,
    pub price: i64,
    pub price_type: String, // "listed" or "sold"
    pub property_type: String,
    pub bedrooms: i32,
    pub bathrooms: i32,
    pub square_feet: i32,
    pub days_on_market: i32,
    pub distance_km: f64,
    pub match_confidence: i32, // 0-100
}

// Market Trends API Models

/// Query parameters for market trends
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendsQuery {
    pub period: Option<String>, // "30d", "90d", "1y"
}

/// Market trends response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsResponse {
    pub area: String,
    pub period: String,
    pub average_price: String,
    pub median_price: String,
    pub price_per_sqft: String,
    pub median_days_on_market: i32,
    pub inventory_level: i32,
    pub sales_volume: i32,
    pub price_trend_direction: String, // e.g., "+2.3%"
    pub market_status: MarketStatus,
    pub generated_at: String,
    pub data_as_of: String,
    pub neighborhoods: Option<Vec<String>>,
}

/// Market status indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketStatus {
    BuyersMarket,
    SellersMarket,
    BalancedMarket,
}

/// List of supported areas response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedAreasResponse {
    pub areas: Vec<String>,
    pub message: String,
}

// API Request Logging Models

/// Log entry for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequestLog {
    pub id: String,
    pub agent_id: String,
    pub endpoint: String,
    pub method: String,
    pub request_body: Option<String>,
    pub response_status: i32,
    pub duration_ms: i64,
    pub timestamp: String,
    pub ip_address: Option<String>,
}

/// Request to log API call
#[derive(Debug, Clone)]
pub struct LogApiRequest {
    pub agent_id: String,
    pub endpoint: String,
    pub method: String,
    pub request_body: Option<String>,
    pub response_status: i32,
    pub duration_ms: i64,
    pub ip_address: Option<String>,
}

impl ApiRequestLog {
    pub fn new(request: LogApiRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id: request.agent_id,
            endpoint: request.endpoint,
            method: request.method,
            request_body: request.request_body,
            response_status: request.response_status,
            duration_ms: request.duration_ms,
            timestamp: chrono::Utc::now().to_rfc3339(),
            ip_address: request.ip_address,
        }
    }
}

// Validation helpers

impl CreateCmaRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.address.trim().is_empty() {
            return Err("Address is required".to_string());
        }
        if self.property_type.trim().is_empty() {
            return Err("Property type is required".to_string());
        }
        Ok(())
    }
}

impl ComparablesQuery {
    pub fn validate(&self) -> Result<(), String> {
        if self.address.trim().is_empty() {
            return Err("Address is required".to_string());
        }
        if let Some(radius) = self.radius {
            if radius <= 0.0 || radius > 10.0 {
                return Err("Radius must be between 0.1 and 10 km".to_string());
            }
        }
        if let Some(limit) = self.limit {
            if limit <= 0 || limit > 10 {
                return Err("Limit must be between 1 and 10".to_string());
            }
        }
        Ok(())
    }

    pub fn get_radius(&self) -> f64 {
        self.radius.unwrap_or(1.0)
    }

    pub fn get_limit(&self) -> i32 {
        self.limit.unwrap_or(5).min(10)
    }
}

impl TrendsQuery {
    pub fn get_period(&self) -> String {
        match self.period.as_deref() {
            Some("30d") => "30d".to_string(),
            Some("90d") => "90d".to_string(),
            Some("1y") => "1y".to_string(),
            _ => "90d".to_string(), // default
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(period) = &self.period {
            if !["30d", "90d", "1y"].contains(&period.as_str()) {
                return Err(format!(
                    "Invalid period: {}. Supported values: 30d, 90d, 1y",
                    period
                ));
            }
        }
        Ok(())
    }
}

/// List of supported areas in Durham Region
pub fn get_supported_areas() -> Vec<String> {
    vec![
        "pickering".to_string(),
        "ajax".to_string(),
        "whitby".to_string(),
        "oshawa".to_string(),
        "courtice".to_string(),
        "bowmanville".to_string(),
        "newcastle".to_string(),
        "uxbridge".to_string(),
        "scugog".to_string(),
        "brock".to_string(),
        "durham-region".to_string(),
    ]
}

/// Check if area is supported
pub fn is_area_supported(area: &str) -> bool {
    let normalized = area.to_lowercase().replace(" ", "-");
    get_supported_areas().contains(&normalized)
}

/// Get neighborhoods for a city
pub fn get_neighborhoods(city: &str) -> Vec<String> {
    let normalized = city.to_lowercase().replace(" ", "-");
    match normalized.as_str() {
        "pickering" => vec![
            "amberleigh".to_string(),
            "highlands".to_string(),
            "rouge-park".to_string(),
            "village-east".to_string(),
            "rosebank".to_string(),
        ],
        "ajax" => vec![
            "south-ajax".to_string(),
            "central-ajax".to_string(),
            "north-ajax".to_string(),
            "downtown".to_string(),
        ],
        "whitby" => vec![
            "brooklin".to_string(),
            "downtown-whitby".to_string(),
            "pringle-creek".to_string(),
            "williamsburg".to_string(),
        ],
        "oshawa" => vec![
            "north-oshawa".to_string(),
            "downtown-oshawa".to_string(),
            "oshawa-creek".to_string(),
            "windfields".to_string(),
            "mclaughlin".to_string(),
        ],
        _ => vec![],
    }
}

// Additional API Models for Active Listings & Property Details

/// Query parameters for active listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveListingsQuery {
    pub area: String,
    pub property_type: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub bedrooms_min: Option<i32>,
    pub bathrooms_min: Option<i32>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

impl ActiveListingsQuery {
    pub fn validate(&self) -> Result<(), String> {
        if self.area.trim().is_empty() {
            return Err("Area is required".to_string());
        }
        if let Some(limit) = self.limit {
            if limit <= 0 || limit > 50 {
                return Err("Limit must be between 1 and 50".to_string());
            }
        }
        Ok(())
    }

    pub fn get_limit(&self) -> i32 {
        self.limit.unwrap_or(20).min(50)
    }

    pub fn to_service_filter(&self) -> crate::services::ActiveListingsFilter {
        crate::services::ActiveListingsFilter {
            property_type: self.property_type.clone(),
            price_min: self.price_min,
            price_max: self.price_max,
            bedrooms_min: self.bedrooms_min,
            bathrooms_min: self.bathrooms_min,
            limit: self.get_limit(),
            offset: self.offset,
        }
    }
}

/// Paginated active listings response
#[derive(Debug, Clone, Serialize)]
pub struct ActiveListingsApiResponse {
    pub listings: Vec<crate::services::ActiveListing>,
    pub total: i32,
    pub page: i32,
    pub has_more: bool,
}

/// Property details API response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PropertyDetailsApiResponse {
    pub property: crate::services::PropertyDetails,
}

/// Area statistics API response
#[derive(Debug, Clone, Serialize)]
pub struct AreaStatisticsApiResponse {
    pub statistics: crate::services::AreaStatistics,
}

/// Query for area statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AreaStatisticsQuery {
    pub property_type: Option<String>,
}
