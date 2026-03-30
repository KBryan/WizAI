use super::{Invoice, InvoiceLineItem, PaymentRecord, PaymentStatus};
use crate::agent::core::AgentId;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug)]
pub struct PaymentLedger {
    pool: Pool<Sqlite>,
}

impl PaymentLedger {
    pub fn new() -> Self {
        // In-memory SQLite for demo
        let pool = tokio::runtime::Handle::current().block_on(async {
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect(":memory:")
                .await
                .expect("Failed to create in-memory database")
        });

        let ledger = Self { pool };
        tokio::runtime::Handle::current().block_on(async {
            ledger.init_tables().await.expect("Failed to init tables")
        });
        ledger
    }

    async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS payments (
                id TEXT PRIMARY KEY,
                from_agent_id TEXT NOT NULL,
                to_agent_id TEXT NOT NULL,
                amount_usd REAL NOT NULL,
                resource_type TEXT NOT NULL,
                description TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                status TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_payments_from ON payments(from_agent_id);
            CREATE INDEX IF NOT EXISTS idx_payments_to ON payments(to_agent_id);
            CREATE INDEX IF NOT EXISTS idx_payments_timestamp ON payments(timestamp);
            "#,
        )
        .execute(&self.pool)
        .await?;

        debug!("Payment ledger tables initialized");
        Ok(())
    }

    pub async fn add_payment(&self, payment: PaymentRecord) -> Result<()> {
        let resource_type_json = serde_json::to_string(&payment.resource_type)?;

        sqlx::query(
            r#"
            INSERT INTO payments (id, from_agent_id, to_agent_id, amount_usd, resource_type, description, timestamp, status)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(payment.id.clone())
        .bind(payment.from.0.to_string())
        .bind(payment.to.0.to_string())
        .bind(payment.amount_usd)
        .bind(resource_type_json)
        .bind(&payment.description)
        .bind(payment.timestamp.to_rfc3339())
        .bind(format!("{:?}", payment.status))
        .execute(&self.pool)
        .await?;

        debug!("Added payment {} to ledger", payment.id);
        Ok(())
    }

    pub async fn get_agent_payments(&self, agent_id: AgentId) -> Result<Vec<PaymentRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, from_agent_id, to_agent_id, amount_usd, resource_type, description, timestamp, status
            FROM payments
            WHERE from_agent_id = ?1 OR to_agent_id = ?1
            ORDER BY timestamp DESC
            "#,
        )
        .bind(agent_id.0.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut payments = Vec::new();
        for row in rows {
            let id: String = row.try_get(0)?;
            let from_id: String = row.try_get(1)?;
            let to_id: String = row.try_get(2)?;
            let amount_usd: f64 = row.try_get(3)?;
            let resource_type_json: String = row.try_get(4)?;
            let description: String = row.try_get(5)?;
            let timestamp_str: String = row.try_get(6)?;
            let status_str: String = row.try_get(7)?;

            let from = AgentId(
                uuid::Uuid::parse_str(&from_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            );
            let to = AgentId(
                uuid::Uuid::parse_str(&to_id).unwrap_or_else(|_| uuid::Uuid::new_v4()),
            );
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let resource_type = serde_json::from_str(&resource_type_json)?;
            let status = match status_str.as_str() {
                "Pending" => PaymentStatus::Pending,
                "Completed" => PaymentStatus::Completed,
                _ => PaymentStatus::Failed("Unknown".to_string()),
            };

            payments.push(PaymentRecord {
                id,
                from,
                to,
                amount_usd,
                resource_type,
                description,
                timestamp,
                status,
            });
        }

        Ok(payments)
    }

    pub async fn get_agent_balance(&self, agent_id: AgentId) -> Result<f64> {
        // Sum of all outgoing payments
        let spent: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount_usd), 0)
            FROM payments
            WHERE from_agent_id = ?1
            "#,
        )
        .bind(agent_id.0.to_string())
        .fetch_one(&self.pool)
        .await?;

        // Sum of all incoming payments
        let received: f64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount_usd), 0)
            FROM payments
            WHERE to_agent_id = ?1
            "#,
        )
        .bind(agent_id.0.to_string())
        .fetch_one(&self.pool)
        .await?;

        // Balance = received - spent (negative means they owe money)
        Ok(received - spent)
    }

    pub async fn get_cost_breakdown(&self, agent_id: AgentId) -> Result<HashMap<String, f64>> {
        let rows = sqlx::query(
            r#"
            SELECT resource_type, SUM(amount_usd) as total
            FROM payments
            WHERE from_agent_id = ?1
            GROUP BY resource_type
            "#,
        )
        .bind(agent_id.0.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut breakdown = HashMap::new();
        for row in rows {
            let resource_type_json: String = row.try_get(0)?;
            let total: f64 = row.try_get(1)?;
            breakdown.insert(resource_type_json, total);
        }

        Ok(breakdown)
    }

    pub async fn generate_invoice(&self, agent_id: AgentId) -> Result<Invoice> {
        let now = Utc::now();
        let period_start = now - Duration::days(30);

        let payments = sqlx::query(
            r#"
            SELECT resource_type, description, amount_usd, timestamp
            FROM payments
            WHERE from_agent_id = ?1
            AND timestamp >= ?2
            ORDER BY timestamp DESC
            "#,
        )
        .bind(agent_id.0.to_string())
        .bind(period_start.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut line_items = Vec::new();
        let mut total = 0.0;

        for row in payments {
            let resource_type_json: String = row.try_get(0)?;
            let description: String = row.try_get(1)?;
            let amount: f64 = row.try_get(2)?;

            line_items.push(InvoiceLineItem {
                description: description.clone(),
                quantity: 1.0,
                unit_price_usd: amount,
                total_usd: amount,
                resource_type: resource_type_json.clone(),
            });

            total += amount;
        }

        Ok(Invoice {
            invoice_id: uuid::Uuid::new_v4().to_string(),
            agent_id,
            period_start,
            period_end: now,
            line_items,
            total_usd: total,
            generated_at: now,
        })
    }
}
