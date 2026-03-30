-- Migration: API Request Logging
-- Creates tables for logging market research API requests
-- This is required for the Market Research API Endpoints feature

-- Create API request logs table
CREATE TABLE IF NOT EXISTS api_request_logs (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    method TEXT NOT NULL,
    request_body TEXT,
    response_status INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    timestamp TEXT NOT NULL,
    ip_address TEXT,
    
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);

-- Index for querying by agent
CREATE INDEX IF NOT EXISTS idx_api_logs_agent_id ON api_request_logs(agent_id);

-- Index for querying by timestamp
CREATE INDEX IF NOT EXISTS idx_api_logs_timestamp ON api_request_logs(timestamp);

-- Index for querying by endpoint
CREATE INDEX IF NOT EXISTS idx_api_logs_endpoint ON api_request_logs(endpoint);

-- Index for querying by response status (e.g., for error analysis)
CREATE INDEX IF NOT EXISTS idx_api_logs_status ON api_request_logs(response_status);
