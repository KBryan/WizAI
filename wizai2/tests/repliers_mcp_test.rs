//! Integration tests for Repliers MCP

use std::collections::HashMap;
use spree_agent::agent::roles::real_estate_researcher::{
    ResearchRequest, PropertyType, PriceRange, FocusArea, OutputFormat
};
use spree_agent::mcp::protocol::MCPContent;

#[tokio::test]
async fn test_repliers_mcp_connection() {
    // Import MCP client
    use spree_agent::mcp::MCPClient;
    
    println!("Testing Repliers MCP Connection...");
    
    // Create MCP client
    let client = MCPClient::new();
    
    // Get API key from environment
    let api_key = std::env::var("REPLIERS_API_KEY")
        .expect("REPLIERS_API_KEY must be set");
    
    println!("Connecting to Repliers MCP server...");
    
    // Connect to Repliers MCP server
    let result = client.connect_stdio(
        "repliers",
        "node",
        &["/Users/kwamebryan/Documents/GitHub/WizAI/repliers-mcp-server/mcpServer.js".to_string()],
        Some(HashMap::from([
            ("REPLIERS_API_KEY".to_string(), api_key),
        ])),
    ).await;
    
    match result {
        Ok(_) => {
            println!("✅ Successfully connected to Repliers MCP server");
            
            // List available servers
            let servers = client.list_servers().await;
            println!("Connected servers: {:?}", servers);
            
            // Get available tools
            match client.get_server_tools("repliers").await {
                Ok(tools) => {
                    println!("✅ Discovered {} tools:", tools.len());
                    for tool in &tools {
                        println!("  - {}", tool.name);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get tools: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            panic!("Connection failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_durham_region_search() {
    use spree_agent::mcp::MCPClient;
    
    println!("\nTesting Durham Region Search...");
    
    let client = MCPClient::new();
    let api_key = std::env::var("REPLIERS_API_KEY")
        .expect("REPLIERS_API_KEY must be set");
    
    // Connect first
    client.connect_stdio(
        "repliers",
        "node",
        &["/Users/kwamebryan/Documents/GitHub/WizAI/repliers-mcp-server/mcpServer.js".to_string()],
        Some(HashMap::from([
            ("REPLIERS_API_KEY".to_string(), api_key),
        ])),
    ).await.expect("Failed to connect");
    
    // Test search parameters
    let search_params = serde_json::json!({
        "city": "Durham Region, ON",
        "status": "active",
        "propertyType": ["Detached", "Townhouse"],
        "minListPrice": 500000,
        "maxListPrice": 1000000,
    });
    
    println!("Searching Durham Region listings...");
    
    match client.call_tool("repliers", "repliers_listings_search", search_params).await {
        Ok(result) => {
            println!("✅ Search successful!");
            
            // Extract and display results
            let text_content: Vec<String> = result.content.iter()
                .filter_map(|c| match c {
                    MCPContent::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect();
            
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.join("")) {
                if let Some(results) = json["results"].as_array() {
                    println!("Found {} listings", results.len());
                    
                    // Show first 3 listings
                    for (i, listing) in results.iter().take(3).enumerate() {
                        println!("\nListing {}:", i + 1);
                        println!("  Address: {}", 
                            listing["address"]["streetAddress"].as_str().unwrap_or("N/A"));
                        println!("  Price: ${}", 
                            listing["listPrice"].as_f64().unwrap_or(0.0));
                        println!("  Type: {}", 
                            listing["propertyType"].as_str().unwrap_or("N/A"));
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Search failed: {}", e);
            panic!("Search failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_real_estate_researcher_role() {
    use spree_agent::agent::roles::real_estate_researcher::*;
    
    println!("\nTesting Real Estate Researcher Role...");
    
    // Create research request for Durham Region
    let request = ResearchRequest {
        location: "Durham Region, ON".to_string(),
        property_types: vec![PropertyType::Detached, PropertyType::Townhouse],
        price_range: Some(PriceRange { min: 500000, max: 1000000 }),
        focus_areas: vec![FocusArea::MarketTrends, FocusArea::PricingAnalysis],
        output_format: OutputFormat::MarketReport,
    };
    
    println!("Research Request:");
    println!("  Location: {}", request.location);
    println!("  Property Types: {:?}", request.property_types);
    println!("  Price Range: ${} - ${}", 
        request.price_range.as_ref().map(|p| p.min).unwrap_or(0),
        request.price_range.as_ref().map(|p| p.max).unwrap_or(0));
    println!("  Focus Areas: {:?}", request.focus_areas);
    
    // Note: Full integration test would require initializing the full agent system
    // This test validates the request structure works
    
    println!("✅ Research request structure validated");
}
