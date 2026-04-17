use std::collections::HashMap;

/// Pricing model inspired by mppx and cloud providers
/// Per-token pricing for LLM calls, per-second for compute

#[derive(Debug, Clone)]
pub struct PricingModel {
    /// Price per 1K tokens for each model
    llm_pricing: HashMap<String, (f64, f64)>, // (input_price, output_price) per 1K tokens
    /// Price per core-second
    compute_price_per_sec: f64,
    /// Price per GB-month of storage
    storage_price_per_gb_month: f64,
    /// Base price for tool execution
    tool_base_price: f64,
}

impl Default for PricingModel {
    fn default() -> Self {
        let mut llm_pricing = HashMap::new();
        // Venice AI pricing (example rates)
        llm_pricing.insert("venice-uncensored".to_string(), (0.0001, 0.0002)); // $0.10 per 1K input, $0.20 per 1K output
        llm_pricing.insert("venice-llama-3.3-70b".to_string(), (0.0005, 0.0010));
        llm_pricing.insert("venice-qwen-2.5-coder-32b".to_string(), (0.0003, 0.0006));

        // OpenAI pricing for comparison
        llm_pricing.insert("gpt-4".to_string(), (0.03, 0.06));
        llm_pricing.insert("gpt-4-turbo".to_string(), (0.01, 0.03));
        llm_pricing.insert("gpt-3.5-turbo".to_string(), (0.0005, 0.0015));

        Self {
            llm_pricing,
            compute_price_per_sec: 0.0001, // $0.36 per core-hour
            storage_price_per_gb_month: 0.02,
            tool_base_price: 0.001, // Base $0.001 per tool execution
        }
    }
}

impl PricingModel {
    /// Calculate cost for LLM call
    pub fn calculate_llm_cost(&self, model: &str, tokens_in: i64, tokens_out: i64) -> f64 {
        let (in_price, out_price) = self.llm_pricing.get(model).unwrap_or(&(0.0001, 0.0002));

        let in_cost = (tokens_in as f64 / 1000.0) * in_price;
        let out_cost = (tokens_out as f64 / 1000.0) * out_price;

        in_cost + out_cost
    }

    /// Calculate cost for compute time
    pub fn calculate_compute_cost(&self, duration_secs: i64, cpu_cores: i32) -> f64 {
        (duration_secs as f64) * (cpu_cores as f64) * self.compute_price_per_sec
    }

    /// Calculate cost for storage
    pub fn calculate_storage_cost(&self, bytes: i64) -> f64 {
        let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        gb * self.storage_price_per_gb_month / 30.0 / 24.0 / 3600.0 // Per second
    }

    /// Calculate cost for tool execution
    pub fn calculate_tool_cost(&self, tool_name: &str) -> f64 {
        // Different tools may have different costs
        match tool_name {
            "bash" => self.tool_base_price * 1.0,
            "python" => self.tool_base_price * 1.5, // Python is more expensive
            "read_file" => self.tool_base_price * 0.5,
            "write_file" => self.tool_base_price * 0.5,
            "subordinate" => self.tool_base_price * 10.0, // Creating agents is expensive
            _ => self.tool_base_price,
        }
    }

    /// Get price for a specific model
    pub fn get_model_price(&self, model: &str) -> Option<(f64, f64)> {
        self.llm_pricing.get(model).copied()
    }

    /// Add custom pricing for a model
    pub fn add_model_pricing(&mut self, model: &str, input_price: f64, output_price: f64) {
        self.llm_pricing
            .insert(model.to_string(), (input_price, output_price));
    }
}

/// Budget management for agents
#[derive(Debug, Clone)]
pub struct BudgetManager {
    total_budget_usd: f64,
    spent_usd: f64,
    alert_threshold: f64, // Percentage at which to alert
}

impl BudgetManager {
    pub fn new(total_budget_usd: f64) -> Self {
        Self {
            total_budget_usd,
            spent_usd: 0.0,
            alert_threshold: 0.8, // Alert at 80% budget
        }
    }

    pub fn spend(&mut self, amount_usd: f64) -> bool {
        if self.spent_usd + amount_usd > self.total_budget_usd {
            false // Cannot spend, over budget
        } else {
            self.spent_usd += amount_usd;
            true
        }
    }

    pub fn remaining(&self) -> f64 {
        self.total_budget_usd - self.spent_usd
    }

    pub fn is_over_threshold(&self) -> bool {
        self.spent_usd / self.total_budget_usd >= self.alert_threshold
    }

    pub fn percentage_used(&self) -> f64 {
        (self.spent_usd / self.total_budget_usd) * 100.0
    }
}

/// Cost estimation before execution
pub fn estimate_experiment_cost(
    model: &str,
    expected_tokens: i64,
    compute_duration_secs: i64,
) -> f64 {
    let pricing = PricingModel::default();
    let llm_cost = pricing.calculate_llm_cost(model, expected_tokens / 2, expected_tokens / 2);
    let compute_cost = pricing.calculate_compute_cost(compute_duration_secs, 1);
    llm_cost + compute_cost
}
