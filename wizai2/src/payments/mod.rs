pub mod ledger;
pub mod pricing;

pub use ledger::*;
pub use pricing::*;

use crate::agent::core::AgentId;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Payment system inspired by mppx (Machine Payments Protocol)
/// Tracks resource usage and facilitates agent-to-agent payments

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub id: String,
    pub from: AgentId,
    pub to: AgentId,
    pub amount_usd: f64,
    pub resource_type: ResourceType,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    LLMCall { model: String, tokens_in: i64, tokens_out: i64 },
    Compute { duration_secs: i64, cpu_cores: i32 },
    Storage { bytes: i64 },
    ToolExecution { tool_name: String },
    ExperimentRun { experiment_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct PaymentSystem {
    ledger: Arc<RwLock<PaymentLedger>>,
    pricing: PricingModel,
}

impl PaymentSystem {
    pub fn new() -> Self {
        Self {
            ledger: Arc::new(RwLock::new(PaymentLedger::new())),
            pricing: PricingModel::default(),
        }
    }

    /// Charge for LLM usage (like mppx per-token pricing)
    pub async fn charge_llm(
        &self,
        agent_id: AgentId,
        model: &str,
        tokens_in: i64,
        tokens_out: i64,
    ) -> Result<f64> {
        let cost = self.pricing.calculate_llm_cost(model, tokens_in, tokens_out);
        
        let payment = PaymentRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from: agent_id,
            to: AgentId::default(), // System receives payment
            amount_usd: cost,
            resource_type: ResourceType::LLMCall {
                model: model.to_string(),
                tokens_in,
                tokens_out,
            },
            description: format!("LLM call: {} ({} in, {} out tokens)", model, tokens_in, tokens_out),
            timestamp: Utc::now(),
            status: PaymentStatus::Completed,
        };

        self.ledger.write().await.add_payment(payment).await?;
        
        info!("Charged agent {:?} ${:.4} for LLM usage", agent_id, cost);
        Ok(cost)
    }

    /// Charge for compute time
    pub async fn charge_compute(
        &self,
        agent_id: AgentId,
        duration_secs: i64,
        cpu_cores: i32,
    ) -> Result<f64> {
        let cost = self.pricing.calculate_compute_cost(duration_secs, cpu_cores);
        
        let payment = PaymentRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from: agent_id,
            to: AgentId::default(),
            amount_usd: cost,
            resource_type: ResourceType::Compute {
                duration_secs,
                cpu_cores,
            },
            description: format!("Compute: {} cores for {} seconds", cpu_cores, duration_secs),
            timestamp: Utc::now(),
            status: PaymentStatus::Completed,
        };

        self.ledger.write().await.add_payment(payment).await?;
        
        info!("Charged agent {:?} ${:.4} for compute", agent_id, cost);
        Ok(cost)
    }

    /// Charge for experiment (autoresearch-style fixed budget)
    pub async fn charge_experiment(
        &self,
        agent_id: AgentId,
        experiment_id: String,
        budget_usd: f64,
    ) -> Result<f64> {
        let payment = PaymentRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from: agent_id,
            to: AgentId::default(),
            amount_usd: budget_usd,
            resource_type: ResourceType::ExperimentRun { experiment_id: experiment_id.clone() },
            description: format!("Experiment: {} (fixed budget)", experiment_id),
            timestamp: Utc::now(),
            status: PaymentStatus::Completed,
        };

        self.ledger.write().await.add_payment(payment).await?;
        
        info!("Charged agent {:?} ${:.4} for experiment {}", agent_id, budget_usd, experiment_id);
        Ok(budget_usd)
    }

    /// Transfer payment between agents (for delegation)
    pub async fn transfer(
        &self,
        from: AgentId,
        to: AgentId,
        amount_usd: f64,
        description: &str,
    ) -> Result<String> {
        let payment = PaymentRecord {
            id: uuid::Uuid::new_v4().to_string(),
            from,
            to,
            amount_usd,
            resource_type: ResourceType::ToolExecution {
                tool_name: "payment_transfer".to_string(),
            },
            description: description.to_string(),
            timestamp: Utc::now(),
            status: PaymentStatus::Completed,
        };

        let payment_id = payment.id.clone();
        self.ledger.write().await.add_payment(payment).await?;
        
        info!("Transferred ${:.4} from {:?} to {:?}", amount_usd, from, to);
        Ok(payment_id)
    }

    /// Get agent's balance (negative means they owe money)
    pub async fn get_balance(&self, agent_id: AgentId) -> Result<f64> {
        self.ledger.read().await.get_agent_balance(agent_id).await
    }

    /// Get payment history for an agent
    pub async fn get_payment_history(&self, agent_id: AgentId) -> Result<Vec<PaymentRecord>> {
        self.ledger.read().await.get_agent_payments(agent_id).await
    }

    /// Get cost breakdown by resource type
    pub async fn get_cost_breakdown(&self, agent_id: AgentId) -> Result<HashMap<String, f64>> {
        self.ledger.read().await.get_cost_breakdown(agent_id).await
    }

    /// Generate invoice for an agent
    pub async fn generate_invoice(&self, agent_id: AgentId) -> Result<Invoice> {
        self.ledger.read().await.generate_invoice(agent_id).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_id: String,
    pub agent_id: AgentId,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub line_items: Vec<InvoiceLineItem>,
    pub total_usd: f64,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price_usd: f64,
    pub total_usd: f64,
    pub resource_type: String,
}

/// Mock payment processor (in production, integrate with Stripe, etc.)
pub async fn process_payment(amount_usd: f64, payment_method: &str) -> Result<String> {
    info!("Processing payment of ${:.2} via {}", amount_usd, payment_method);
    // Mock success
    Ok(uuid::Uuid::new_v4().to_string())
}
