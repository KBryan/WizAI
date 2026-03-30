use crate::api::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CmaReport {
    pub id: String,
    pub subject_address: String,
    pub subject_mls_number: Option<String>,
    pub property_type: String,
    pub status: String,
    pub generated_by: String,
    pub reviewed_by: Option<String>,
    pub reviewed_at: Option<String>,
    pub comparables: serde_json::Value,
    pub market_analysis: serde_json::Value,
    pub summary: String,
    pub price_recommendation_low: Option<f64>,
    pub price_recommendation_mid: Option<f64>,
    pub price_recommendation_high: Option<f64>,
    pub market_conditions: Option<String>,
    pub avg_days_on_market: Option<f32>,
    pub price_per_sqft: Option<f64>,
    pub list_to_sale_ratio: Option<f32>,
    pub confidence: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CMASummary {
    pub id: String,
    pub subject_address: String,
    pub property_type: String,
    pub status: String,
    pub price_recommendation_mid: Option<f64>,
    pub comparable_count: i32,
    pub confidence: i32,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub limit: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateCmaRequest {
    pub address: String,
    pub property_type: String,
    pub mls_number: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateCmaResponse {
    pub cma_id: String,
    pub status: String,
    pub estimated_completion: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CmaStatusResponse {
    pub cma_id: String,
    pub status: String,
    pub progress: i32,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComparablesQuery {
    pub address: String,
    pub property_type: Option<String>,
    pub radius_km: Option<f64>,
    pub limit: Option<i32>,
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
pub struct ComparablesResponse {
    pub target_property: serde_json::Value,
    pub comparables: Vec<ComparableProperty>,
    pub count: i32,
}

pub async fn list_cmas(
    status: Option<String>,
    page: Option<i32>,
    limit: Option<i32>,
) -> Result<PaginatedList<CMASummary>, String> {
    let client = ApiClient::new();
    
    let mut params = vec![];
    if let Some(s) = status {
        params.push(format!("status={}", s));
    }
    if let Some(p) = page {
        params.push(format!("page={}", p));
    }
    if let Some(l) = limit {
        params.push(format!("limit={}", l));
    }
    
    let query = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };
    
    let endpoint = format!("/market-research/cmas{}", query);
    
    let response: ApiResponse<PaginatedList<CMASummary>> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn create_cma(request: CreateCmaRequest) -> Result<CreateCmaResponse, String> {
    let client = ApiClient::new();
    
    let response: ApiResponse<CreateCmaResponse> = client.post("/market-research/cmas/", &request).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn get_cma(id: &str) -> Result<CmaReport, String> {
    let client = ApiClient::new();
    let endpoint = format!("/market-research/cmas/{}", id);
    
    let response: ApiResponse<CmaReport> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn get_cma_status(id: &str) -> Result<CmaStatusResponse, String> {
    let client = ApiClient::new();
    let endpoint = format!("/market-research/cmas/{}/status", id);
    
    let response: ApiResponse<CmaStatusResponse> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn search_comparables(query: ComparablesQuery) -> Result<ComparablesResponse, String> {
    let client = ApiClient::new();
    
    let mut params = vec![format!("address={}", urlencoding::encode(&query.address))];
    if let Some(pt) = &query.property_type {
        params.push(format!("property_type={}", pt));
    }
    if let Some(r) = query.radius_km {
        params.push(format!("radius_km={}", r));
    }
    if let Some(l) = query.limit {
        params.push(format!("limit={}", l));
    }
    
    let endpoint = format!("/market-research/comparables?{}", params.join("&"));
    
    let response: ApiResponse<ComparablesResponse> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}
