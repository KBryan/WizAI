-- Skills System Migration
-- Creates tables for storing, executing, and improving skills

-- Main skills table
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
);

-- Skill execution history for tracking and improvement
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
);

-- Skill generation records (for audit trail)
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
);

-- Create indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_skills_category ON skills(category);
CREATE INDEX IF NOT EXISTS idx_skills_author ON skills(author);
CREATE INDEX IF NOT EXISTS idx_skills_usage ON skills(usage_count DESC);
CREATE INDEX IF NOT EXISTS idx_skills_success_rate ON skills(success_rate DESC);

CREATE INDEX IF NOT EXISTS idx_skill_executions_skill ON skill_executions(skill_id);
CREATE INDEX IF NOT EXISTS idx_skill_executions_agent ON skill_executions(agent_id);
CREATE INDEX IF NOT EXISTS idx_skill_executions_executed ON skill_executions(executed_at DESC);

CREATE INDEX IF NOT EXISTS idx_skill_generations_agent ON skill_generations(agent_id);
CREATE INDEX IF NOT EXISTS idx_skill_generations_created ON skill_generations(created_at DESC);

-- Create FTS5 virtual table for full-text search on skills
CREATE VIRTUAL TABLE IF NOT EXISTS skills_fts USING fts5(
    name,
    description,
    tags,
    content='skills',
    content_rowid='rowid'
);

-- Trigger to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS skills_ai AFTER INSERT ON skills BEGIN
    INSERT INTO skills_fts(rowid, name, description, tags)
    VALUES (new.rowid, new.name, new.description, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS skills_ad AFTER DELETE ON skills BEGIN
    INSERT INTO skills_fts(skills_fts, rowid, name, description, tags)
    VALUES ('delete', old.rowid, old.name, old.description, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS skills_au AFTER UPDATE ON skills BEGIN
    INSERT INTO skills_fts(skills_fts, rowid, name, description, tags)
    VALUES ('delete', old.rowid, old.name, old.description, old.tags);
    INSERT INTO skills_fts(rowid, name, description, tags)
    VALUES (new.rowid, new.name, new.description, new.tags);
END;
