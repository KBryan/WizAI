use crate::agent::core::{AgentId, Message, MessageRole};
use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub struct MemoryStore {
    pool: Pool<Sqlite>,
}

impl MemoryStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let db_path = std::path::Path::new(database_url);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let store = Self { pool };
        store.init_tables().await?;
        
        info!("MemoryStore initialized at {}", database_url);
        
        Ok(store)
    }

    async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                metadata TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS solutions (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                task_description TEXT NOT NULL,
                solution_content TEXT NOT NULL,
                tags TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                memory_type TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                metadata TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_messages_agent ON messages(agent_id);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);
            CREATE INDEX IF NOT EXISTS idx_solutions_agent ON solutions(agent_id);
            CREATE INDEX IF NOT EXISTS idx_memories_agent ON memories(agent_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        debug!("Database tables initialized");
        
        Ok(())
    }

    pub async fn store_message(&self, agent_id: AgentId, message: &Message) -> Result<()> {
        let metadata_json = if let Some(meta) = &message.metadata {
            serde_json::to_string(meta)?
        } else {
            "{}".to_string()
        };

        let timestamp_str = message.timestamp.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO messages (id, agent_id, role, content, timestamp, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(message.id.to_string())
        .bind(agent_id.0.to_string())
        .bind(format!("{:?}", message.role))
        .bind(&message.content)
        .bind(timestamp_str)
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

        debug!("Stored message {} for agent {:?}", message.id, agent_id);
        
        Ok(())
    }

    pub async fn get_messages(&self, agent_id: AgentId, limit: i64) -> Result<Vec<Message>> {
        let rows = sqlx::query(
            r#"
            SELECT id, role, content, timestamp, metadata
            FROM messages
            WHERE agent_id = ?1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#,
        )
        .bind(agent_id.0.to_string())
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut messages = Vec::new();
        
        for row in rows {
            let id_str: String = row.try_get(0)?;
            let role_str: String = row.try_get(1)?;
            let content: String = row.try_get(2)?;
            let timestamp_str: String = row.try_get(3)?;
            let metadata_str: String = row.try_get(4).unwrap_or_default();

            let id = uuid::Uuid::parse_str(&id_str)
                .unwrap_or_else(|_| uuid::Uuid::new_v4());
            
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let role = match role_str.as_str() {
                "User" => MessageRole::User,
                "Assistant" => MessageRole::Assistant,
                "System" => MessageRole::System,
                "Tool" => MessageRole::Tool,
                _ => MessageRole::User,
            };

            let metadata = if metadata_str.is_empty() || metadata_str == "{}" {
                None
            } else {
                serde_json::from_str(&metadata_str).ok()
            };

            messages.push(Message {
                id,
                role,
                content,
                timestamp,
                metadata,
            });
        }

        Ok(messages)
    }

    pub async fn store_memory(
        &self,
        agent_id: AgentId,
        memory_type: &str,
        content: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let metadata_json = metadata.map(|m| m.to_string()).unwrap_or_else(|| "{}".to_string());
        let timestamp = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO memories (id, agent_id, memory_type, content, created_at, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&id)
        .bind(agent_id.0.to_string())
        .bind(memory_type)
        .bind(content)
        .bind(timestamp)
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

        debug!("Stored memory {} for agent {:?}", id, agent_id);
        
        Ok(())
    }

    pub async fn search_memories(&self, agent_id: AgentId, query: &str) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query(
            r#"
            SELECT memory_type, content
            FROM memories
            WHERE agent_id = ?1
            AND (
                content LIKE ?2
                OR memory_type LIKE ?2
            )
            ORDER BY created_at DESC
            LIMIT 10
            "#,
        )
        .bind(agent_id.0.to_string())
        .bind(format!("%{}%", query))
        .fetch_all(&self.pool)
        .await?;

        let mut memories = Vec::new();
        for row in rows {
            let memory_type: String = row.try_get(0)?;
            let content: String = row.try_get(1)?;
            memories.push((memory_type, content));
        }

        Ok(memories)
    }

    pub async fn store_solution(
        &self,
        agent_id: AgentId,
        task: &str,
        solution: &str,
        tags: Vec<String>,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let tags_json = serde_json::to_string(&tags)?;
        let timestamp = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO solutions (id, agent_id, task_description, solution_content, tags, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&id)
        .bind(agent_id.0.to_string())
        .bind(task)
        .bind(solution)
        .bind(tags_json)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        debug!("Stored solution {} for agent {:?}", id, agent_id);
        
        Ok(())
    }

    pub async fn find_similar_solutions(&self, task: &str) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query(
            r#"
            SELECT task_description, solution_content
            FROM solutions
            WHERE task_description LIKE ?1
            ORDER BY created_at DESC
            LIMIT 5
            "#,
        )
        .bind(format!("%{}%", task))
        .fetch_all(&self.pool)
        .await?;

        let mut solutions = Vec::new();
        for row in rows {
            let task: String = row.try_get(0)?;
            let solution: String = row.try_get(1)?;
            solutions.push((task, solution));
        }

        Ok(solutions)
    }
}
