use spree_agent::agent::core::{AgentId, AgentRole, AgentRegistry};
use spree_agent::agent::roles::generate_role_prompt;
use spree_agent::server::create_router;
use spree_agent::AppState;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting Spree Agent Framework...");
    
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize application state
    let state = Arc::new(AppState::new().await?);
    
    // Create initial CEO agent if none exists
    initialize_ceo(state.clone()).await?;
    
    // Create Axum router
    let app = create_router(state);
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server starting on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn initialize_ceo(state: Arc<AppState>) -> Result<()> {
    let mut registry = state.agent_registry.write().await;
    
    // Check if we already have agents
    let existing_agents = registry.list_agents();
    if existing_agents.is_empty() {
        info!("Creating initial CEO agent...");
        
        let ceo_name = "Spree CEO".to_string();
        let system_prompt = generate_role_prompt(&AgentRole::CEO, &ceo_name, Some(
            "You are the top-level agent in the Spree organization. Users will interact with you directly. \
            Delegate tasks to appropriate C-level executives or create new agents as needed."
        ));
        
        let ceo_id = registry.create_agent(
            ceo_name,
            AgentRole::CEO,
            None, // No superior - reports to user
            system_prompt,
        ).await?;
        
        info!("Created CEO agent with ID: {:?}", ceo_id);
        
        // Create initial C-level team
        create_initial_team(&mut registry, ceo_id).await?;
    }
    
    Ok(())
}

async fn create_initial_team(
    registry: &mut AgentRegistry,
    ceo_id: AgentId,
) -> Result<()> {
    info!("Creating initial C-level team...");
    
    // CTO
    let cto_name = "Spree CTO".to_string();
    let cto_prompt = generate_role_prompt(&AgentRole::CTO, &cto_name, None);
    let cto_id = registry.create_agent(
        cto_name,
        AgentRole::CTO,
        Some(ceo_id),
        cto_prompt,
    ).await?;
    info!("Created CTO agent: {:?}", cto_id);
    
    // CFO
    let cfo_name = "Spree CFO".to_string();
    let cfo_prompt = generate_role_prompt(&AgentRole::CFO, &cfo_name, None);
    let cfo_id = registry.create_agent(
        cfo_name,
        AgentRole::CFO,
        Some(ceo_id),
        cfo_prompt,
    ).await?;
    info!("Created CFO agent: {:?}", cfo_id);
    
    // Chief AI Officer
    let chief_ai_name = "Spree Chief AI Officer".to_string();
    let chief_ai_prompt = generate_role_prompt(&AgentRole::ChiefAI, &chief_ai_name, None);
    let chief_ai_id = registry.create_agent(
        chief_ai_name,
        AgentRole::ChiefAI,
        Some(ceo_id),
        chief_ai_prompt,
    ).await?;
    info!("Created Chief AI Officer agent: {:?}", chief_ai_id);
    
    // Chief Product Officer
    let chief_product_name = "Spree Chief Product Officer".to_string();
    let chief_product_prompt = generate_role_prompt(&AgentRole::ChiefProduct, &chief_product_name, None);
    let chief_product_id = registry.create_agent(
        chief_product_name,
        AgentRole::ChiefProduct,
        Some(ceo_id),
        chief_product_prompt,
    ).await?;
    info!("Created Chief Product Officer agent: {:?}", chief_product_id);
    
    info!("Initial team created successfully!");
    Ok(())
}
