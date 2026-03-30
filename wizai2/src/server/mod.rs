pub mod api;
pub mod ws;

use crate::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .layer(cors)
        // WebSocket endpoint
        .route("/ws", get(ws::websocket_handler))
        // REST API endpoints
        .route("/api/agents", get(api::list_agents).post(api::create_agent))
        .route(
            "/api/agents/:id",
            get(api::get_agent).post(api::send_message),
        )
        .route("/api/agents/:id/subordinates", get(api::get_subordinates))
        .route("/api/organization", get(api::get_organization_tree))
        .route("/api/tasks", post(api::submit_task))
        // Skills API endpoints
        .route("/api/skills", get(api::list_skills))
        .route("/api/skills/:name", get(api::get_skill))
        // Real Estate Dashboard API
        .route("/api/realestate/execute", post(api::realestate_execute))
        .route("/api/realestate/status", get(api::realestate_status))
        // Honest Mode API
        .route("/api/honest-mode", get(api::get_honest_mode).post(api::update_honest_mode))
        // Market Research API
        .route("/api/market-research/cma", post(api::create_cma).get(api::list_cmas))
        .route("/api/market-research/cma/:id", get(api::get_cma))
        .route("/api/market-research/cma/:id/status", get(api::get_cma_status))
        .route("/api/market-research/comparables", get(api::search_comparables))
        .route("/api/market-research/trends/:area", get(api::get_market_trends))
        .route("/api/market-research/areas", get(api::get_supported_areas_handler))
        .route("/api/market-research/neighborhoods/:city", get(api::get_neighborhoods))
        // Repliers Extended API
        .route("/api/market-research/listings/active", get(api::get_active_listings))
        .route("/api/market-research/property/:mls_number", get(api::get_property_details))
        .route("/api/market-research/statistics/:area", get(api::get_area_statistics))
        // Serve static files
        .nest_service("/", ServeDir::new("assets"))
        .with_state(state)
}
