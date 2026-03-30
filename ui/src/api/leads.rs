use crate::api::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub source: String,
    pub inquiry_type: Option<String>,
    pub property_type: Option<String>,
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
    pub desired_location: Option<String>,
    pub timeline: Option<String>,
    pub notes: Option<String>,
    pub is_urgent: bool,
    pub qualification: Option<String>,
    pub assigned_agent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LeadSummary {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: String,
    pub source: String,
    pub inquiry_type: Option<String>,
    pub is_urgent: bool,
    pub created_at: String,
    pub last_contact_at: Option<String>,
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
pub struct LeadFilter {
    pub status: Option<String>,
    pub source: Option<String>,
    pub inquiry_type: Option<String>,
    pub assigned_agent_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CreateLeadRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub preferred_contact: Option<String>,
    pub source: Option<String>,
    pub inquiry_type: Option<String>,
    pub property_type: Option<String>,
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
    pub desired_location: Option<String>,
    pub timeline: Option<String>,
    pub notes: Option<String>,
    pub is_urgent: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateLeadRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<String>,
    pub property_type: Option<String>,
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
    pub desired_location: Option<String>,
    pub timeline: Option<String>,
    pub notes: Option<String>,
    pub is_urgent: Option<bool>,
    pub assigned_agent_id: Option<String>,
}

pub async fn list_leads(
    filter: Option<LeadFilter>,
    page: Option<i32>,
    limit: Option<i32>,
) -> Result<PaginatedList<LeadSummary>, String> {
    let client = ApiClient::new();
    
    let mut params = vec![];
    if let Some(f) = &filter {
        if let Some(status) = &f.status {
            params.push(format!("status={}", status));
        }
        if let Some(source) = &f.source {
            params.push(format!("source={}", source));
        }
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
    
    let endpoint = format!("/leads/listing{}", query);
    
    let response: ApiResponse<PaginatedList<LeadSummary>> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn get_lead(id: &str) -> Result<Lead, String> {
    let client = ApiClient::new();
    let endpoint = format!("/leads/{}", id);
    
    let response: ApiResponse<Lead> = client.get(&endpoint).await?;
    
    response.data.ok_or_else(|| response.error.unwrap_or_else(|| "Unknown error".to_string()))
}

pub async fn create_lead(request: CreateLeadRequest) -> Result<Lead, String> {
    let client = ApiClient::new();
    
    #[derive(Deserialize)]
    struct CreateResponse {
        pub id: String,
    }
    
    let response: ApiResponse<CreateResponse> = client.post("/leads/", &request).await?;
    
    if let Some(id) = response.data.map(|d| d.id) {
        get_lead(&id).await
    } else {
        Err(response.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

pub async fn update_lead(id: &str, request: UpdateLeadRequest) -> Result<Lead, String> {
    let client = ApiClient::new();
    let endpoint = format!("/leads/{}", id);
    
    let _: ApiResponse<()> = client.put(&endpoint, &request).await?;
    
    get_lead(id).await
}

pub async fn delete_lead(id: &str) -> Result<(), String> {
    let client = ApiClient::new();
    let endpoint = format!("/leads/{}", id);
    
    let _: ApiResponse<()> = client.delete(&endpoint).await?;
    
    Ok(())
}

impl ApiClient {
    pub async fn delete<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut opts = web_sys::RequestInit::new();
        opts.method("DELETE");
        opts.mode(web_sys::RequestMode::Cors);

        let request = web_sys::Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| format!("Request error: {:?}", e))?;

        let window = web_sys::window().ok_or("No window")?;
        let resp_value = window
            .fetch_with_request(&request)
            .await
            .map_err(|e| format!("Fetch error: {:?}", e))?;

        let resp: web_sys::Response = resp_value.dyn_into()
            .map_err(|_| "Failed to cast response")?;

        if resp.ok() {
            let json_str = resp.text()
                .await
                .map_err(|e| format!("Text error: {:?}", e))?
                .as_string()
                .unwrap_or_default();
            
            if json_str.is_empty() {
                #[derive(Deserialize)]
                struct Empty {}
                Ok(Empty {} as T)
            } else {
                serde_json::from_str(&json_str)
                    .map_err(|e| format!("JSON parse error: {}", e))
            }
        } else {
            Err(format!("HTTP error: {}", resp.status()))
        }
    }
}
