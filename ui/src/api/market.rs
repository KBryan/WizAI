use crate::api::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub error_code: Option<String>,
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
    pub market_status: String,
    pub generated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrendsQuery {
    pub period: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SupportedAreasResponse {
    pub areas: Vec<String>,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeighborhoodsResponse {
    pub data: Vec<String>,
}

pub async fn get_market_trends(area: &str, period: Option<&str>) -> Result<MarketTrends, String> {
    let client = ApiClient::new();
    
    let mut params = vec![];
    if let Some(p) = period {
        params.push(format!("period={}", p));
    }
    
    let query = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    
    let endpoint = format!("/market-research/trends/{}{}", area, query);
    
    let response: ApiResponse<MarketTrends> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn get_supported_areas() -> Result<Vec<String>, String> {
    let client = ApiClient::new();
    
    let response: ApiResponse<SupportedAreasResponse> = client.get("/market-research/areas").await?;
    
    response.data
        .map(|r| r.areas)
        .ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn get_neighborhoods(city: &str) -> Result<Vec<String>, String> {
    let client = ApiClient::new();
    
    let endpoint = format!("/market-research/neighborhoods/{}", city);
    
    let response: ApiResponse<Vec<String>> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AreaStatistics {
    pub avg_sold_price: f64,
    pub median_sold_price: f64,
    pub price_per_sqft: f64,
    pub avg_days_on_market: f64,
    pub total_sold: i32,
    pub total_listings: i32,
    pub months_of_inventory: Option<f64>,
}

pub async fn get_area_statistics(
    area: &str,
    property_type: Option<&str>,
) -> Result<AreaStatistics, String> {
    let client = ApiClient::new();
    
    let mut params = vec![];
    if let Some(pt) = property_type {
        params.push(format!("property_type={}", pt));
    }
    
    let query = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    
    let endpoint = format!("/market-research/statistics/{}{}", area, query);
    
    #[derive(Deserialize)]
    struct StatsResponse {
        pub statistics: AreaStatistics,
    }
    
    let response: ApiResponse<StatsResponse> = client.get(&endpoint).await?;
    
    response.data
        .map(|r| r.statistics)
        .ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}
