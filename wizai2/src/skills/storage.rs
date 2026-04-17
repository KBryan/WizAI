use super::models::*;
use crate::agent::core::AgentId;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Row, Sqlite};
use tracing::{debug, error, info, warn};

pub struct SkillStorage {
    pool: Pool<Sqlite>,
}

impl std::fmt::Debug for SkillStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillStorage").finish()
    }
}

impl SkillStorage {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn init_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                version TEXT NOT NULL,
                author TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                usage_count INTEGER DEFAULT 0,
                success_rate REAL DEFAULT 1.0,
                average_execution_time_ms INTEGER DEFAULT 0,
                tags TEXT,
                category TEXT NOT NULL,
                triggers TEXT,
                implementation TEXT NOT NULL,
                metadata TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS skill_executions (
                id TEXT PRIMARY KEY,
                skill_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                input TEXT NOT NULL,
                output TEXT,
                success INTEGER NOT NULL,
                execution_time_ms INTEGER,
                executed_at TEXT NOT NULL,
                metadata TEXT,
                FOREIGN KEY (skill_id) REFERENCES skills(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS skill_generations (
                id TEXT PRIMARY KEY,
                skill_id TEXT,
                agent_id TEXT NOT NULL,
                task_description TEXT NOT NULL,
                task_context TEXT,
                execution_trace TEXT,
                outcome TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (skill_id) REFERENCES skills(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);
            CREATE INDEX IF NOT EXISTS idx_skills_author ON skills(author);
            CREATE INDEX IF NOT EXISTS idx_skill_executions_skill ON skill_executions(skill_id);
            CREATE INDEX IF NOT EXISTS idx_skill_executions_agent ON skill_executions(agent_id);
            CREATE INDEX IF NOT EXISTS idx_skill_generations_agent ON skill_generations(agent_id);
            "#,
        )
        .execute(&self.pool)
        .await?;

        debug!("Skill tables initialized");
        Ok(())
    }

    pub async fn store_skill(&self, skill: &Skill) -> Result<()> {
        let tags_json = serde_json::to_string(&skill.tags)?;
        let triggers_json = serde_json::to_string(&skill.triggers)?;
        let implementation_json = serde_json::to_string(&skill.implementation)?;
        let metadata_json = serde_json::to_string(&skill.metadata)?;

        sqlx::query(
            r#"
            INSERT INTO skills (
                id, name, description, version, author, created_at, updated_at,
                usage_count, success_rate, average_execution_time_ms, tags, category,
                triggers, implementation, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                version = excluded.version,
                updated_at = excluded.updated_at,
                usage_count = excluded.usage_count,
                success_rate = excluded.success_rate,
                average_execution_time_ms = excluded.average_execution_time_ms,
                tags = excluded.tags,
                category = excluded.category,
                triggers = excluded.triggers,
                implementation = excluded.implementation,
                metadata = excluded.metadata
            "#,
        )
        .bind(&skill.id)
        .bind(&skill.name)
        .bind(&skill.description)
        .bind(&skill.version)
        .bind(&skill.author)
        .bind(skill.created_at.to_rfc3339())
        .bind(skill.updated_at.to_rfc3339())
        .bind(skill.usage_count)
        .bind(skill.success_rate)
        .bind(skill.average_execution_time_ms)
        .bind(tags_json)
        .bind(skill.category.to_string())
        .bind(triggers_json)
        .bind(implementation_json)
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

        debug!("Stored skill {}", skill.id);
        Ok(())
    }

    pub async fn get_skill(&self, skill_id: &str) -> Result<Option<Skill>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, version, author, created_at, updated_at,
                usage_count, success_rate, average_execution_time_ms, tags, category,
                triggers, implementation, metadata
            FROM skills
            WHERE id = ?1
            "#,
        )
        .bind(skill_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_skill(row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_skill_by_name(&self, name: &str) -> Result<Option<Skill>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, version, author, created_at, updated_at,
                usage_count, success_rate, average_execution_time_ms, tags, category,
                triggers, implementation, metadata
            FROM skills
            WHERE name = ?1
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_skill(row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_skills(&self, category: Option<&str>) -> Result<Vec<Skill>> {
        let rows = if let Some(cat) = category {
            sqlx::query(
                r#"
                SELECT id, name, description, version, author, created_at, updated_at,
                    usage_count, success_rate, average_execution_time_ms, tags, category,
                    triggers, implementation, metadata
                FROM skills
                WHERE category = ?1
                ORDER BY usage_count DESC
                "#,
            )
            .bind(cat)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, name, description, version, author, created_at, updated_at,
                    usage_count, success_rate, average_execution_time_ms, tags, category,
                    triggers, implementation, metadata
                FROM skills
                ORDER BY usage_count DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await?
        };

        let mut skills = Vec::new();
        for row in rows {
            skills.push(self.row_to_skill(row)?);
        }

        Ok(skills)
    }

    pub async fn search_skills(&self, query: &str) -> Result<Vec<Skill>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, version, author, created_at, updated_at,
                usage_count, success_rate, average_execution_time_ms, tags, category,
                triggers, implementation, metadata
            FROM skills
            WHERE name LIKE ?1
            OR description LIKE ?1
            OR tags LIKE ?1
            ORDER BY usage_count DESC
            LIMIT 20
            "#,
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await?;

        let mut skills = Vec::new();
        for row in rows {
            skills.push(self.row_to_skill(row)?);
        }

        Ok(skills)
    }

    pub async fn delete_skill(&self, skill_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM skills WHERE id = ?1")
            .bind(skill_id)
            .execute(&self.pool)
            .await?;

        debug!("Deleted skill {}", skill_id);
        Ok(())
    }

    pub async fn record_execution(
        &self,
        skill_id: &str,
        agent_id: &str,
        input: &serde_json::Value,
        output: Option<&serde_json::Value>,
        success: bool,
        execution_time_ms: i64,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let input_json = serde_json::to_string(input)?;
        let output_json = output.map(|o| o.to_string());
        let metadata = serde_json::json!({});

        sqlx::query(
            r#"
            INSERT INTO skill_executions (id, skill_id, agent_id, input, output, success, execution_time_ms, executed_at, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(&id)
        .bind(skill_id)
        .bind(agent_id)
        .bind(input_json)
        .bind(output_json)
        .bind(success)
        .bind(execution_time_ms)
        .bind(Utc::now().to_rfc3339())
        .bind(metadata.to_string())
        .execute(&self.pool)
        .await?;

        // Update skill statistics
        self.update_skill_stats(skill_id).await?;

        debug!("Recorded execution {} for skill {}", id, skill_id);
        Ok(())
    }

    async fn update_skill_stats(&self, skill_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE skills
            SET usage_count = (
                SELECT COUNT(*) FROM skill_executions WHERE skill_id = ?1
            ),
            success_rate = COALESCE((
                SELECT AVG(CAST(success AS REAL)) FROM skill_executions WHERE skill_id = ?1
            ), 1.0),
            average_execution_time_ms = COALESCE((
                SELECT AVG(execution_time_ms) FROM skill_executions WHERE skill_id = ?1
            ), 0)
            WHERE id = ?1
            "#,
        )
        .bind(skill_id)
        .bind(skill_id)
        .bind(skill_id)
        .bind(skill_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_generation(&self, request: &SkillGenerationRequest, skill_id: Option<&str>) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let trace_json = serde_json::to_string(&request.execution_trace)?;
        let outcome_json = serde_json::to_string(&request.outcome)?;

        sqlx::query(
            r#"
            INSERT INTO skill_generations (id, skill_id, agent_id, task_description, task_context, execution_trace, outcome, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(&id)
        .bind(skill_id)
        .bind(&request.agent_id)
        .bind(&request.task_description)
        .bind(&request.task_context)
        .bind(trace_json)
        .bind(outcome_json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    fn row_to_skill(&self, row: sqlx::sqlite::SqliteRow) -> Result<Skill> {
        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let description: String = row.try_get("description")?;
        let version: String = row.try_get("version")?;
        let author: String = row.try_get("author")?;
        let created_at_str: String = row.try_get("created_at")?;
        let updated_at_str: String = row.try_get("updated_at")?;
        let usage_count: i64 = row.try_get("usage_count")?;
        let success_rate: f64 = row.try_get("success_rate")?;
        let average_execution_time_ms: i64 = row.try_get("average_execution_time_ms")?;
        let tags_json: String = row.try_get("tags")?;
        let category_str: String = row.try_get("category")?;
        let triggers_json: String = row.try_get("triggers")?;
        let implementation_json: String = row.try_get("implementation")?;
        let metadata_json: String = row.try_get("metadata").unwrap_or_default();

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let tags: Vec<String> = serde_json::from_str(&tags_json)?;
        let triggers: Vec<String> = serde_json::from_str(&triggers_json)?;
        let implementation: SkillImplementation = serde_json::from_str(&implementation_json)?;
        let metadata: std::collections::HashMap<String, serde_json::Value> = 
            serde_json::from_str(&metadata_json).unwrap_or_default();

        Ok(Skill {
            id,
            name,
            description,
            version,
            author,
            created_at,
            updated_at,
            usage_count,
            success_rate,
            average_execution_time_ms,
            tags,
            category: SkillCategory::from(category_str),
            triggers,
            implementation,
            metadata,
        })
    }

    /// Export skills in agentskills.io format
    pub async fn export_to_agentskills(&self, skill_ids: Option<Vec<String>>) -> Result<AgentskillsExport> {
        let skills = if let Some(ids) = skill_ids {
            let mut skills = Vec::new();
            for id in ids {
                if let Some(skill) = self.get_skill(&id).await? {
                    skills.push(skill);
                }
            }
            skills
        } else {
            self.list_skills(None).await?
        };

        Ok(AgentskillsExport::new(skills))
    }

    /// Import skills from agentskills.io format
    pub async fn import_from_agentskills(&self, export: &AgentskillsExport, author: &str) -> Result<Vec<String>> {
        let mut imported_ids = Vec::new();
        
        for mut skill in export.skills.clone() {
            skill.id = uuid::Uuid::new_v4().to_string();
            skill.author = author.to_string();
            skill.created_at = Utc::now();
            skill.updated_at = Utc::now();
            skill.usage_count = 0;
            skill.success_rate = 1.0;
            
            self.store_skill(&skill).await?;
            imported_ids.push(skill.id);
        }

        info!("Imported {} skills from agentskills.io format", imported_ids.len());
        Ok(imported_ids)
    }
}
