pub mod agent;
pub mod cli_generator;
pub mod compliance;
pub mod llm;
pub mod memory;
pub mod mcp;
pub mod models;
pub mod payments;
pub mod server;
pub mod services;
pub mod skills;
pub mod tools;
pub mod webscraping;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct AppState {
    pub agent_registry: Arc<RwLock<agent::AgentRegistry>>,
    pub memory_store: Arc<RwLock<memory::MemoryStore>>,
    pub skill_storage: Arc<skills::SkillStorage>,
    pub skill_executor: Arc<skills::SkillExecutor>,
    pub skill_generator: Arc<skills::SkillGenerator>,
    pub db_pool: Arc<sqlx::Pool<sqlx::Sqlite>>,
    pub llm_client: Arc<llm::VeniceClient>,
    pub tool_registry: Arc<RwLock<tools::ToolRegistry>>,
    pub mcp_client: Arc<mcp::MCPClient>,
    pub web_scraper: Arc<webscraping::WebScrapingClient>,
}

impl AppState {
    pub async fn new() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        // Use data directory for persistent storage - resolve to absolute path
        let db_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data").join("wizai2.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let memory_store = Arc::new(RwLock::new(
            memory::MemoryStore::new(db_path.to_str().unwrap()).await?
        ));
        
        // Create database pool for services (needed before migrations)
        let db_pool = Arc::new(
            sqlx::SqlitePool::connect(db_path.to_str().unwrap()).await?
        );
        
        // Initialize skill storage and tables
        let skill_storage = Arc::new(skills::SkillStorage::new((*db_pool).clone()));
        skill_storage.init_tables().await?;
        
        let llm_client = Arc::new(llm::VeniceClient::new()?);
        
        let tool_registry = Arc::new(RwLock::new(tools::ToolRegistry::new()));
        
        // Initialize MCP client
        let mcp_client = Arc::new(mcp::MCPClient::new());
        
        // Initialize skill executor and generator
        let skill_executor = Arc::new(skills::SkillExecutor::new(
            skill_storage.clone(),
            llm_client.clone(),
            tool_registry.clone(),
        ));
        
        let skill_generator = Arc::new(skills::SkillGenerator::new(
            skill_storage.clone(),
            llm_client.clone(),
        ));
        
        // Register skill management tools
        {
            let mut registry = tool_registry.write().await;
            registry.register_skill_tools(skill_storage.clone(), skill_executor.clone());
        }
        
        // Try to connect to Repliers MCP server if API key is available
        if let Ok(api_key) = std::env::var("REPLIERS_API_KEY") {
            let mcp_server_path = std::path::Path::new("/Users/kwamebryan/Documents/GitHub/WizAI/repliers-mcp-server/mcpServer.js");
            if mcp_server_path.exists() {
                tracing::info!("Connecting to Repliers MCP server...");
                match mcp_client.connect_stdio(
                    "repliers",
                    "node",
                    &[mcp_server_path.to_str().unwrap().to_string()],
                    Some(HashMap::from([
                        ("REPLIERS_API_KEY".to_string(), api_key),
                        ("NODE_ENV".to_string(), "production".to_string()),
                    ])),
                ).await {
                    Ok(_) => {
                        tracing::info!("✅ Connected to Repliers MCP server");
                        // Load MCP tools into tool registry
                        let mut registry = tool_registry.write().await;
                        if let Err(e) = registry.load_mcp_tools(&mcp_client).await {
                            tracing::warn!("Failed to load MCP tools: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect to Repliers MCP server: {}", e);
                    }
                }
            } else {
                tracing::warn!("Repliers MCP server not found at {:?}", mcp_server_path);
            }
        } else {
            tracing::warn!("REPLIERS_API_KEY not set, skipping Repliers MCP connection");
        }
        
        // Initialize web scraping client
        let web_scraper = Arc::new(webscraping::WebScrapingClient::from_env());
        
        // Register web scraping tools
        if web_scraper.is_configured() {
            tracing::info!("Web scraping configured: {}", web_scraper.get_config_info()["provider"]);
            // Web scraping tools will be registered by the agent when needed
        } else {
            tracing::info!("Web scraping not configured. Set SERPAPI_KEY or SCRAPINGBEE_KEY to enable.");
        }
        
        let agent_registry = Arc::new(RwLock::new(
            agent::AgentRegistry::new(
                memory_store.clone(),
                llm_client.clone(),
                tool_registry.clone(),
            )
        ));

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&*db_pool)
            .await
            .map_err(|e| anyhow::anyhow!("Migration error: {}", e))?;

        Ok(Self {
            agent_registry,
            memory_store,
            skill_storage,
            skill_executor,
            skill_generator,
            db_pool,
            llm_client,
            tool_registry,
            mcp_client,
            web_scraper,
        })
    }
}
