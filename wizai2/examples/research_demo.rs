// Research + Payment Integration Demo
// Demonstrates Spree agents conducting paid ML research experiments

use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use spree_agent::agent::core::{AgentId, AgentRegistry, AgentRole};
use spree_agent::agent::executor::TaskRequest;
use spree_agent::agent::roles::generate_role_prompt;
use spree_agent::agent::roles::research::{ExperimentConfig, ResearchAgent};
use spree_agent::llm::VeniceClient;
use spree_agent::memory::MemoryStore;
use spree_agent::payments::PaymentSystem;
use spree_agent::tools::ToolRegistry;
use spree_agent::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       SPREE RESEARCH + PAYMENT INTEGRATION DEMO              ║");
    println!("║  Autoresearch-style experiments with mppx-style micropayments ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Initialize systems
    info!("Initializing Spree Agent Framework...");
    
    let memory_store = Arc::new(RwLock::new(
        MemoryStore::new("research_demo.db").await?,
    ));
    
    let llm_client = Arc::new(VeniceClient::new()?);
    let tool_registry = Arc::new(RwLock::new(ToolRegistry::new()));
    let payment_system = PaymentSystem::new();

    let mut agent_registry = AgentRegistry::new(
        memory_store.clone(),
        llm_client.clone(),
        tool_registry.clone(),
    );

    // Create Chief AI Officer who will oversee research
    let chief_ai_name = "Dr. AI Chief";
    let chief_ai_prompt = generate_role_prompt(
        &AgentRole::ChiefAI,
        chief_ai_name,
        Some("You oversee ML research and manage research budgets."),
    );
    
    let chief_ai_id = agent_registry
        .create_agent(
            chief_ai_name.to_string(),
            AgentRole::ChiefAI,
            None,
            chief_ai_prompt,
        )
        .await?;

    println!("✓ Created Chief AI Officer: {:?}", chief_ai_id);

    // Create Research Team Lead
    let research_lead_name = "Research Lead Alpha";
    let research_lead_prompt = generate_role_prompt(
        &AgentRole::Lead,
        research_lead_name,
        Some("You manage a team of research scientists conducting ML experiments."),
    );
    
    let research_lead_id = agent_registry
        .create_agent(
            research_lead_name.to_string(),
            AgentRole::Lead,
            Some(chief_ai_id),
            research_lead_prompt,
        )
        .await?;

    println!("✓ Created Research Lead: {:?}", research_lead_id);

    // Assign budget to Chief AI Officer
    let initial_budget = 10.0; // $10 USD
    println!("\n💰 Allocating ${:.2} budget to Chief AI Officer", initial_budget);

    // Create research agents under the lead
    let hypotheses = vec![
        "Test if increasing model depth from 8 to 12 layers improves validation perplexity",
        "Experiment with different learning rate schedules for faster convergence",
        "Compare AdamW vs Muon optimizer for nano-scale models",
    ];

    println!("\n🔬 Starting Research Campaign with {} experiments", hypotheses.len());
    println!("═══════════════════════════════════════════════════\n");

    let mut total_experiment_cost = 0.0;
    let mut results = Vec::new();

    for (i, hypothesis) in hypotheses.iter().enumerate() {
        println!("\n📊 Experiment {}: {}", i + 1, hypothesis);
        println!("────────────────────────────────────────");

        // Create a research agent for this experiment
        let researcher_name = format!("Researcher {}", i + 1);
        let researcher_prompt = generate_role_prompt(
            &AgentRole::Specialist,
            &researcher_name,
            Some(&format!("You are conducting research on: {}", hypothesis)),
        );
        
        let researcher_id = agent_registry
            .create_agent(
                researcher_name.clone(),
                AgentRole::Specialist,
                Some(research_lead_id),
                researcher_prompt,
            )
            .await?;

        println!("  👤 Created researcher: {}", researcher_name);

        // Create research agent executor
        let research_agent = ResearchAgent::new(researcher_id, Arc::new(RwLock::new(agent_registry.clone()))).await?;

        // Design experiment
        println!("  🧠 Designing experiment configuration...");
        let config = research_agent.design_experiment(hypothesis).await?;
        println!("  📋 Configuration: {:?}", config);

        // Estimate cost
        let estimated_cost = config.budget_usd;
        println!("  💵 Estimated cost: ${:.4}", estimated_cost);

        // Check if we can afford it
        let chief_ai_balance = payment_system.get_balance(chief_ai_id).await?;
        if chief_ai_balance < estimated_cost {
            println!("  ⚠️  Insufficient funds! Skipping experiment.");
            continue;
        }

        // Charge for the experiment
        let actual_cost = payment_system
            .charge_experiment(researcher_id, config.name.clone(), estimated_cost)
            .await?;
        
        total_experiment_cost += actual_cost;
        println!("  💸 Charged: ${:.4}", actual_cost);

        // Run the experiment
        println!("  ⏱️  Running experiment (5-minute budget)...");
        let result = research_agent.run_experiment(config).await?;
        
        println!("  ✅ Experiment completed!");
        println!("     - Status: {:?}", result.status);
        println!("     - Duration: {} seconds", result.duration_secs);
        println!("     - Cost: ${:.4}", result.cost_usd);
        
        if let Some(val_bpb) = result.val_bpb {
            println!("     - Validation BPB: {:.4}", val_bpb);
        }

        results.push(result);

        // Simulate additional LLM costs during research
        let llm_cost = payment_system
            .charge_llm(researcher_id, "venice-uncensored", 2000, 1500)
            .await?;
        println!("  💬 LLM usage cost: ${:.4}", llm_cost);
    }

    // Analyze results
    println!("\n═══════════════════════════════════════════════════");
    println!("📈 RESEARCH ANALYSIS");
    println!("═══════════════════════════════════════════════════\n");

    let research_lead = ResearchAgent::new(research_lead_id, Arc::new(RwLock::new(agent_registry.clone()))).await?;
    
    if !results.is_empty() {
        let analysis = research_lead.analyze_results(&results).await?;
        println!("Analysis:\n{}", analysis);
    }

    // Show payment summary
    println!("\n═══════════════════════════════════════════════════");
    println!("💰 PAYMENT SUMMARY");
    println!("═══════════════════════════════════════════════════\n");

    // Get all agents
    let agents = agent_registry.list_agents();
    
    for agent_id in agents {
        let balance = payment_system.get_balance(agent_id).await?;
        let cost_breakdown = payment_system.get_cost_breakdown(agent_id).await?;
        
        if let Some(agent) = agent_registry.get_agent(agent_id) {
            let agent_guard = agent.read().await;
            println!("👤 {} ({:?}):", agent_guard.name, agent_guard.role);
            println!("   Balance: ${:.4}", balance);
            
            if !cost_breakdown.is_empty() {
                println!("   Cost breakdown:");
                for (resource, cost) in cost_breakdown.iter().take(5) {
                    println!("     - {}: ${:.4}", resource, cost);
                }
            }
            println!();
        }
    }

    // Generate invoice for Chief AI Officer
    println!("📄 Generating Invoice...");
    let invoice = payment_system.generate_invoice(chief_ai_id).await?;
    println!("\nInvoice ID: {}", invoice.invoice_id);
    println!("Period: {} to {}", invoice.period_start.format("%Y-%m-%d"), invoice.period_end.format("%Y-%m-%d"));
    println!("Total: ${:.4}", invoice.total_usd);
    println!("\nLine Items:");
    for item in &invoice.line_items {
        println!("  - {}: ${:.4} ({})", item.description, item.total_usd, item.resource_type);
    }

    println!("\n═══════════════════════════════════════════════════");
    println!("✅ DEMO COMPLETE");
    println!("═══════════════════════════════════════════════════");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Hierarchical agent structure (Chief AI → Lead → Researchers)");
    println!("  ✓ Autonomous experiment design and execution");
    println!("  ✓ Per-token LLM pricing (like mppx)");
    println!("  ✓ Per-experiment budget management (like autoresearch)");
    println!("  ✓ Cost tracking and invoicing");
    println!("  ✓ Agent-to-agent delegation with payment tracking");

    Ok(())
}
