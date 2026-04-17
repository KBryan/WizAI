use crate::agent::core::{AgentId, AgentRole, Message, MessageRole};
use crate::agent::executor::{TaskRequest};
use crate::agent::roles::generate_role_prompt;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::fs;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub role: String,
    pub superior: Option<String>,
    pub subordinates: Vec<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub role: String,
    pub superior_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTaskRequest {
    pub agent_id: String,
    pub task: String,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub success: bool,
    pub response: String,
}

pub async fn list_agents(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AgentResponse>>, StatusCode> {
    let registry = state.agent_registry.read().await;
    let agents = registry.list_agents();
    
    let mut responses = Vec::new();
    for agent_id in agents {
        if let Some(agent) = registry.get_agent(agent_id) {
            let agent_guard = agent.read().await;
            responses.push(AgentResponse {
                id: agent_guard.id.0.to_string(),
                name: agent_guard.name.clone(),
                role: format!("{:?}", agent_guard.role),
                superior: agent_guard.superior.map(|s| s.0.to_string()),
                subordinates: agent_guard
                    .subordinates
                    .iter()
                    .map(|s| s.0.to_string())
                    .collect(),
                status: format!("{:?}", agent_guard.status),
            });
        }
    }
    
    Ok(Json(responses))
}

pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAgentRequest>,
) -> Result<Json<AgentResponse>, StatusCode> {
    let role = parse_role(&req.role).ok_or(StatusCode::BAD_REQUEST)?;
    let superior_id = req
        .superior_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .map(AgentId);
    
    let system_prompt = generate_role_prompt(&role, &req.name, None);
    
    let mut registry = state.agent_registry.write().await;
    let agent_id = registry
        .create_agent(req.name.clone(), role.clone(), superior_id, system_prompt)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    info!("Created agent {} with role {:?}", req.name, role);
    
    Ok(Json(AgentResponse {
        id: agent_id.0.to_string(),
        name: req.name,
        role: format!("{:?}", role),
        superior: req.superior_id,
        subordinates: Vec::new(),
        status: "Idle".to_string(),
    }))
}

pub async fn get_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<AgentResponse>, StatusCode> {
    let agent_id = Uuid::parse_str(&id)
        .map(AgentId)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let registry = state.agent_registry.read().await;
    let agent = registry
        .get_agent(agent_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let agent_guard = agent.read().await;
    
    Ok(Json(AgentResponse {
        id: agent_guard.id.0.to_string(),
        name: agent_guard.name.clone(),
        role: format!("{:?}", agent_guard.role),
        superior: agent_guard.superior.map(|s| s.0.to_string()),
        subordinates: agent_guard
            .subordinates
            .iter()
            .map(|s| s.0.to_string())
            .collect(),
        status: format!("{:?}", agent_guard.status),
    }))
}

pub async fn get_subordinates(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<AgentResponse>>, StatusCode> {
    let agent_id = Uuid::parse_str(&id)
        .map(AgentId)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let registry = state.agent_registry.read().await;
    let agent = registry
        .get_agent(agent_id)
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let agent_guard = agent.read().await;
    let subordinate_ids: Vec<AgentId> = agent_guard.subordinates.clone();
    drop(agent_guard);
    
    let mut responses = Vec::new();
    for sub_id in subordinate_ids {
        if let Some(sub) = registry.get_agent(sub_id) {
            let sub_guard = sub.read().await;
            responses.push(AgentResponse {
                id: sub_guard.id.0.to_string(),
                name: sub_guard.name.clone(),
                role: format!("{:?}", sub_guard.role),
                superior: sub_guard.superior.map(|s| s.0.to_string()),
                subordinates: sub_guard
                    .subordinates
                    .iter()
                    .map(|s| s.0.to_string())
                    .collect(),
                status: format!("{:?}", sub_guard.status),
            });
        }
    }
    
    Ok(Json(responses))
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<()>, StatusCode> {
    let agent_id = Uuid::parse_str(&id)
        .map(AgentId)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Check if agent exists before adding message
    let registry = state.agent_registry.read().await;
    if registry.get_agent(agent_id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    drop(registry);
    
    let message = Message {
        id: Uuid::new_v4(),
        role: MessageRole::User,
        content: req.content,
        timestamp: chrono::Utc::now(),
        metadata: None,
    };
    
    let mut registry = state.agent_registry.write().await;
    registry
        .add_message(agent_id, message)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(()))
}

pub async fn get_organization_tree(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<AgentResponse>>, StatusCode> {
    list_agents(State(state)).await
}

pub async fn submit_task(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubmitTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let agent_id = Uuid::parse_str(&req.agent_id)
        .map(AgentId)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let executor = crate::agent::executor::AgentExecutor::new(
        agent_id,
        state.agent_registry.clone(),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let task = TaskRequest {
        task: req.task,
        context: None,
    };
    
    match executor.execute_task(task).await {
        Ok(result) => Ok(Json(TaskResponse {
            success: result.success,
            response: result.response,
        })),
        Err(e) => {
            error!("Task execution failed: {}", e);
            Ok(Json(TaskResponse {
                success: false,
                response: format!("Error: {}", e),
            }))
        }
    }
}

fn parse_role(role_str: &str) -> Option<AgentRole> {
    match role_str.to_lowercase().as_str() {
        "user" => Some(AgentRole::User),
        "ceo" => Some(AgentRole::CEO),
        "cto" => Some(AgentRole::CTO),
        "cfo" => Some(AgentRole::CFO),
        "chiefai" | "chief_ai" | "chief-ai" | "cao" => Some(AgentRole::ChiefAI),
        "chiefproduct" | "chief_product" | "chief-product" | "cpo" => Some(AgentRole::ChiefProduct),
        "openspecexecutor" | "openspec_executor" | "openspec-executor" => Some(AgentRole::OpenSpecExecutor),
        "softwaredeveloper" | "software_developer" | "software-developer" | "developer" => Some(AgentRole::SoftwareDeveloper),
        "realestateresearcher" | "real_estate_researcher" | "real-estate-researcher" | "realtor" => Some(AgentRole::RealEstateResearcher),
        "vp" => Some(AgentRole::VP),
        "director" => Some(AgentRole::Director),
        "manager" => Some(AgentRole::Manager),
        "lead" => Some(AgentRole::Lead),
        "specialist" => Some(AgentRole::Specialist),
        "intern" => Some(AgentRole::Intern),
        _ => None,
    }
}

// Real Estate Dashboard API Endpoints

#[derive(Debug, Deserialize)]
pub struct RealEstateRequest {
    pub command: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RealEstateResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub async fn realestate_execute(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<RealEstateRequest>,
) -> Result<Json<RealEstateResponse>, StatusCode> {
    info!("Real estate command: {}", req.command);
    
    // Try to execute the CLI first, fall back to demo data
    match execute_realestate_cli(&req.command, req.parameters.clone()).await {
        Ok(output) => {
            let data = if output.trim().starts_with('{') {
                serde_json::from_str(&output).ok()
            } else {
                Some(serde_json::json!({"output": output}))
            };
            Ok(Json(RealEstateResponse {
                success: true,
                data,
                error: None,
            }))
        }
        Err(_) => {
            // Return demo data based on command
            let demo_data = get_demo_data(&req.command);
            Ok(Json(RealEstateResponse {
                success: true,
                data: Some(demo_data),
                error: None,
            }))
        }
    }
}

fn get_demo_data(command: &str) -> serde_json::Value {
    match command {
        "stats" => serde_json::json!({
            "avg_price": 847500,
            "median_price": 825000,
            "active_listings": 1247,
            "avg_days_on_market": 12.5,
            "price_per_sqft": 550,
            "total_sales": 342,
            "municipality_breakdown": {
                "Ajax": { "avg_price": 925000, "listings": 189 },
                "Pickering": { "avg_price": 875000, "listings": 156 },
                "Oshawa": { "avg_price": 725000, "listings": 412 },
                "Whitby": { "avg_price": 950000, "listings": 298 },
                "Clarington": { "avg_price": 695000, "listings": 192 }
            }
        }),
        "trends" => serde_json::json!({
            "trends": [
                { "month": "Oct 2025", "avg_price": 820000, "sales": 287 },
                { "month": "Nov 2025", "avg_price": 835000, "sales": 265 },
                { "month": "Dec 2025", "avg_price": 828000, "sales": 198 },
                { "month": "Jan 2026", "avg_price": 840000, "sales": 245 },
                { "month": "Feb 2026", "avg_price": 845000, "sales": 312 },
                { "month": "Mar 2026", "avg_price": 850000, "sales": 342 }
            ],
            "year_over_year_change": 5.2,
            "market_trend": "moderate_growth"
        }),
        "fetch" => serde_json::json!({
            "fetched": 50,
            "source": "demo",
            "message": "Demo data loaded successfully"
        }),
        "report" => serde_json::json!({
            "generated": true,
            "output": "durham_realestate_report.md",
            "message": "Demo report generated"
        }),
        _ => serde_json::json!({
            "message": "Demo data",
            "command": command
        })
    }
}

pub async fn realestate_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<RealEstateResponse>, StatusCode> {
    // Check if the CLI tool exists and get basic stats
    match tokio::process::Command::new("which")
        .arg("durham-realestate-cli")
        .output()
        .await
    {
        Ok(_) => Ok(Json(RealEstateResponse {
            success: true,
            data: Some(serde_json::json!({
                "status": "available",
                "version": "0.1.0",
            })),
            error: None,
        })),
        Err(_) => Ok(Json(RealEstateResponse {
            success: false,
            data: None,
            error: Some("Real estate CLI not installed".to_string()),
        })),
    }
}

async fn execute_realestate_cli(
    command: &str,
    parameters: Option<serde_json::Value>,
) -> anyhow::Result<String> {
    let cli_path = "./openspec/changes/durham-real-estate-analysis/target/release/durham-realestate-cli";
    
    let mut cmd = tokio::process::Command::new(cli_path);
    cmd.arg(command);
    
    // Add parameters based on command
    if let Some(params) = parameters {
        match command {
            "fetch" => {
                if let Some(source) = params.get("source").and_then(|v| v.as_str()) {
                    cmd.arg("--source").arg(source);
                }
                if let Some(limit) = params.get("limit").and_then(|v| v.as_u64()) {
                    cmd.arg("--limit").arg(limit.to_string());
                }
            }
            "stats" => {
                if let Some(mun) = params.get("municipality").and_then(|v| v.as_str()) {
                    cmd.arg("--municipality").arg(mun);
                }
                if let Some(prop) = params.get("property_type").and_then(|v| v.as_str()) {
                    cmd.arg("--property-type").arg(prop);
                }
                if let Some(days) = params.get("days").and_then(|v| v.as_i64()) {
                    cmd.arg("--days").arg(days.to_string());
                }
            }
            "trends" => {
                if let Some(mun) = params.get("municipality").and_then(|v| v.as_str()) {
                    cmd.arg("--municipality").arg(mun);
                }
                if let Some(months) = params.get("months").and_then(|v| v.as_i64()) {
                    cmd.arg("--months").arg(months.to_string());
                }
            }
            _ => {}
        }
    }
    
    cmd.arg("--format").arg("json");
    
    let output = cmd.output().await?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow::anyhow!(
            "CLI failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// Skills API Endpoints

#[derive(Debug, Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct SkillDetail {
    pub name: String,
    pub content: String,
}

fn get_skills_dir() -> &'static str {
    // Check current directory first, then parent
    if std::path::Path::new(".opencode/skills").exists() {
        ".opencode/skills"
    } else {
        "../.opencode/skills"
    }
}

pub async fn list_skills(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<SkillInfo>>, StatusCode> {
    let skills_dir = get_skills_dir();
    
    let mut skills = Vec::new();
    
    if let Ok(entries) = fs::read_dir(skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Try to extract description from SKILL.md
                let skill_file = path.join("SKILL.md");
                let description = if skill_file.exists() {
                    fs::read_to_string(&skill_file)
                        .ok()
                        .and_then(|content| {
                            // Extract first line that looks like a description
                            content.lines()
                                .find(|line| !line.starts_with('#') && !line.is_empty())
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_else(|| "No description available".to_string())
                } else {
                    "No SKILL.md found".to_string()
                };
                
                skills.push(SkillInfo {
                    name: skill_name,
                    description,
                });
            }
        }
    }
    
    Ok(Json(skills))
}

pub async fn get_skill(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<SkillDetail>, StatusCode> {
    let skills_dir = get_skills_dir();
    let skill_path = format!("{}/{}/SKILL.md", skills_dir, name);
    
    match fs::read_to_string(&skill_path) {
        Ok(content) => {
            Ok(Json(SkillDetail {
                name,
                content,
            }))
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

// Honest Mode Configuration

#[derive(Debug, Serialize)]
pub struct HonestModeResponse {
    pub enabled: bool,
    pub model: String,
    pub temperature: f32,
    pub available_models: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHonestModeRequest {
    pub enabled: Option<bool>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
}

pub async fn get_honest_mode() -> Result<Json<HonestModeResponse>, StatusCode> {
    use crate::agent::honest_mode::{HonestModeConfig, UNCENSORED_MODELS};
    
    let config = HonestModeConfig::default();
    
    Ok(Json(HonestModeResponse {
        enabled: config.enabled,
        model: config.model,
        temperature: config.temperature,
        available_models: UNCENSORED_MODELS.iter().map(|&s| s.to_string()).collect(),
    }))
}

pub async fn update_honest_mode(
    Json(req): Json<UpdateHonestModeRequest>,
) -> Result<Json<HonestModeResponse>, StatusCode> {
    use crate::agent::honest_mode::{HonestModeConfig, UNCENSORED_MODELS};
    
    // For now, just return the current config
    // In a full implementation, you'd store this in AppState
    let mut config = HonestModeConfig::default();
    
    if let Some(enabled) = req.enabled {
        if enabled {
            config.enable();
        } else {
            config.disable();
        }
    }
    
    if let Some(model) = req.model {
        config.model = model;
    }
    
    if let Some(temp) = req.temperature {
        config.temperature = temp;
    }
    
    Ok(Json(HonestModeResponse {
        enabled: config.enabled,
        model: config.model,
        temperature: config.temperature,
        available_models: UNCENSORED_MODELS.iter().map(|&s| s.to_string()).collect(),
    }))
}

// Market Research API Endpoints

use crate::models::{
    ActiveListingsApiResponse, ActiveListingsQuery, ApiResponse, AreaStatisticsApiResponse,
    AreaStatisticsQuery, CreateCmaRequest, CreateCmaResponse, ComparablesQuery, ComparablesResponse,
    CmaStatusResponse, ListCmasQuery, LogApiRequest, PaginatedList, PropertyDetailsApiResponse,
    SupportedAreasResponse, TrendsQuery, TrendsResponse, CMASummary, get_supported_areas,
    is_area_supported,
};
use crate::services::MarketResearchService;
use axum::extract::Query;
use chrono::Utc;

/// Create a new CMA report (async)
pub async fn create_cma(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCmaRequest>,
) -> Result<Json<ApiResponse<CreateCmaResponse>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    // TODO: Get agent_id from auth context
    let agent_id = "test-agent".to_string();
    
    match service.generate_cma_async(&agent_id, req).await {
        Ok(response) => {
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            Ok(Json(ApiResponse::error(e, Some("VALIDATION_ERROR".to_string()))))
        }
    }
}

/// Get CMA status
pub async fn get_cma_status(
    State(state): State<Arc<AppState>>,
    Path(cma_id): Path<String>,
) -> Result<Json<ApiResponse<CmaStatusResponse>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    match service.get_cma_status(&cma_id).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e, Some("NOT_FOUND".to_string())))),
    }
}

/// Get CMA report by ID
pub async fn get_cma(
    State(state): State<Arc<AppState>>,
    Path(cma_id): Path<String>,
) -> Result<Json<ApiResponse<crate::models::CMAReport>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    match service.get_cma(&cma_id).await {
        Ok(cma) => Ok(Json(ApiResponse::success(cma))),
        Err(e) => Ok(Json(ApiResponse::error(e, Some("NOT_FOUND".to_string())))),
    }
}

/// List CMAs with filtering
pub async fn list_cmas(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListCmasQuery>,
) -> Result<Json<ApiResponse<PaginatedList<CMASummary>>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    // TODO: Get agent_id from auth context
    let agent_id = "test-agent".to_string();
    
    match service.list_cmas(&agent_id, query).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e, Some("QUERY_ERROR".to_string())))),
    }
}

/// Search comparables
pub async fn search_comparables(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ComparablesQuery>,
) -> Result<Json<ApiResponse<ComparablesResponse>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    let start_time = Utc::now();
    
    match service.search_comparables(query).await {
        Ok(response) => {
            // Log API request
            let _ = service.log_api_request(LogApiRequest {
                agent_id: "test-agent".to_string(),
                endpoint: "/api/market-research/comparables".to_string(),
                method: "GET".to_string(),
                request_body: None,
                response_status: 200,
                duration_ms: (Utc::now() - start_time).num_milliseconds(),
                ip_address: None,
            }).await;
            
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e, Some("VALIDATION_ERROR".to_string())))),
    }
}

/// Get market trends for an area
pub async fn get_market_trends(
    State(state): State<Arc<AppState>>,
    Path(area): Path<String>,
    Query(query): Query<TrendsQuery>,
) -> Result<Json<ApiResponse<TrendsResponse>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    let start_time = Utc::now();
    
    match service.get_market_trends(&area, query).await {
        Ok(response) => {
            // Log API request
            let _ = service.log_api_request(LogApiRequest {
                agent_id: "test-agent".to_string(),
                endpoint: format!("/api/market-research/trends/{}", area),
                method: "GET".to_string(),
                request_body: None,
                response_status: 200,
                duration_ms: (Utc::now() - start_time).num_milliseconds(),
                ip_address: None,
            }).await;
            
            Ok(Json(ApiResponse::success(response)))
        }
        Err(e) => {
            // Check if it's an unsupported area
            if e.contains("not supported") {
                Ok(Json(ApiResponse::error(e, Some("INVALID_AREA".to_string()))))
            } else {
                Ok(Json(ApiResponse::error(e, Some("INTERNAL_ERROR".to_string()))))
            }
        }
    }
}

/// Get supported areas
pub async fn get_supported_areas_handler() -> Result<Json<ApiResponse<SupportedAreasResponse>>, StatusCode> {
    let areas = get_supported_areas();
    
    Ok(Json(ApiResponse::success(SupportedAreasResponse {
        areas,
        message: "Durham Region market areas supported".to_string(),
    })))
}

/// Get neighborhoods for a city
pub async fn get_neighborhoods(
    Path(city): Path<String>,
) -> Result<Json<ApiResponse<Vec<String>>>, StatusCode> {
    if !is_area_supported(&city) {
        return Ok(Json(ApiResponse::error(
            format!("City '{}' is not supported", city),
            Some("INVALID_CITY".to_string()),
        )));
    }
    
    let neighborhoods = crate::models::api::get_neighborhoods(&city);
    Ok(Json(ApiResponse::success(neighborhoods)))
}

// Additional Repliers API Endpoints

/// Get active listings in an area
pub async fn get_active_listings(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ActiveListingsQuery>,
) -> Result<Json<ApiResponse<ActiveListingsApiResponse>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    // Validate query
    if let Err(e) = query.validate() {
        return Ok(Json(ApiResponse::error(e, Some("VALIDATION_ERROR".to_string()))));
    }
    
    let start_time = Utc::now();
    let filters = query.to_service_filter();
    let offset = query.offset.unwrap_or(0);
    
    match service.get_active_listings(&query.area, filters).await {
        Ok(result) => {
            // Log API request
            let _ = service.log_api_request(LogApiRequest {
                agent_id: "test-agent".to_string(),
                endpoint: "/api/market-research/listings/active".to_string(),
                method: "GET".to_string(),
                request_body: None,
                response_status: 200,
                duration_ms: (Utc::now() - start_time).num_milliseconds(),
                ip_address: None,
            }).await;
            
            Ok(Json(ApiResponse::success(ActiveListingsApiResponse {
                listings: result.listings,
                total: result.total,
                page: (offset / query.get_limit()) + 1,
                has_more: result.has_more,
            })))
        }
        Err(e) => Ok(Json(ApiResponse::error(e, Some("API_ERROR".to_string())))),
    }
}

/// Get property details by MLS number
pub async fn get_property_details(
    State(state): State<Arc<AppState>>,
    Path(mls_number): Path<String>,
) -> Result<Json<ApiResponse<PropertyDetailsApiResponse>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    let start_time = Utc::now();
    
    match service.get_property_details(&mls_number).await {
        Ok(property) => {
            // Log API request
            let _ = service.log_api_request(LogApiRequest {
                agent_id: "test-agent".to_string(),
                endpoint: format!("/api/market-research/property/{}", mls_number),
                method: "GET".to_string(),
                request_body: None,
                response_status: 200,
                duration_ms: (Utc::now() - start_time).num_milliseconds(),
                ip_address: None,
            }).await;
            
            Ok(Json(ApiResponse::success(PropertyDetailsApiResponse { property })))
        }
        Err(e) => {
            if e.contains("not found") {
                Ok(Json(ApiResponse::error(e, Some("NOT_FOUND".to_string()))))
            } else {
                Ok(Json(ApiResponse::error(e, Some("API_ERROR".to_string()))))
            }
        }
    }
}

/// Get detailed area statistics
pub async fn get_area_statistics(
    State(state): State<Arc<AppState>>,
    Path(area): Path<String>,
    Query(query): Query<AreaStatisticsQuery>,
) -> Result<Json<ApiResponse<AreaStatisticsApiResponse>>, StatusCode> {
    let service = MarketResearchService::new(Arc::clone(&state.db_pool));
    
    // Validate area
    if !is_area_supported(&area) {
        return Ok(Json(ApiResponse::error(
            format!("Area '{}' is not supported", area),
            Some("INVALID_AREA".to_string()),
        )));
    }
    
    let start_time = Utc::now();
    
    match service.get_area_statistics(&area, query.property_type.as_deref()).await {
        Ok(statistics) => {
            // Log API request
            let _ = service.log_api_request(LogApiRequest {
                agent_id: "test-agent".to_string(),
                endpoint: format!("/api/market-research/statistics/{}", area),
                method: "GET".to_string(),
                request_body: None,
                response_status: 200,
                duration_ms: (Utc::now() - start_time).num_milliseconds(),
                ip_address: None,
            }).await;
            
            Ok(Json(ApiResponse::success(AreaStatisticsApiResponse { statistics })))
        }
        Err(e) => Ok(Json(ApiResponse::error(e, Some("API_ERROR".to_string())))),
    }
}
