-- Migration: Audit Logging Tables
-- Description: Create tables for compliance audit trail and risk assessment

-- Audit log table (append-only, immutable records)
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL, -- 'lead', 'communication', 'cma', 'market_report', etc.
    entity_id TEXT NOT NULL, -- the ID of the entity being audited
    action_type TEXT NOT NULL, -- 'created', 'updated', 'deleted', 'qualified', 'drafted', 'approved', 'rejected', 'sent', etc.
    action_description TEXT NOT NULL,
    performed_by TEXT NOT NULL, -- user or agent ID
    performed_by_type TEXT NOT NULL CHECK (performed_by_type IN ('human', 'ai')),
    risk_level TEXT CHECK (risk_level IN ('low', 'medium', 'high')),
    old_values JSON, -- previous state (for updates)
    new_values JSON, -- new state
    metadata JSON, -- additional context
    previous_hash TEXT, -- hash of previous record for chain verification
    record_hash TEXT NOT NULL, -- hash of this record
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- High-risk actions log (subset for quick querying)
CREATE TABLE IF NOT EXISTS high_risk_actions (
    id TEXT PRIMARY KEY,
    audit_log_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action_type TEXT NOT NULL,
    description TEXT NOT NULL,
    performed_by TEXT NOT NULL,
    reviewed_by TEXT, -- if null, still needs review
    reviewed_at DATETIME,
    review_notes TEXT,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'escalated')),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (audit_log_id) REFERENCES audit_logs(id) ON DELETE CASCADE
);

-- Compliance reports generated
CREATE TABLE IF NOT EXISTS compliance_reports (
    id TEXT PRIMARY KEY,
    report_type TEXT NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    generated_by TEXT NOT NULL,
    report_data JSON NOT NULL,
    total_ai_actions INTEGER,
    approval_rate REAL,
    rejected_count INTEGER,
    high_risk_count INTEGER,
    generated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for audit queries
CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action_type);
CREATE INDEX idx_audit_logs_performed_by ON audit_logs(performed_by);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_risk_level ON audit_logs(risk_level);
CREATE INDEX idx_high_risk_actions_status ON high_risk_actions(status);
CREATE INDEX idx_high_risk_actions_performed_by ON high_risk_actions(performed_by);
CREATE INDEX idx_high_risk_actions_created_at ON high_risk_actions(created_at);

-- CMA reports table
CREATE TABLE IF NOT EXISTS cma_reports (
    id TEXT PRIMARY KEY,
    subject_address TEXT NOT NULL,
    subject_mls_number TEXT,
    property_type TEXT NOT NULL CHECK (property_type IN ('detached', 'semi_detached', 'townhouse', 'condo', 'commercial', 'multi_family', 'land')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'under_review', 'approved', 'rejected', 'archived')),
    generated_by TEXT NOT NULL,
    reviewed_by TEXT,
    reviewed_at DATETIME,
    comparables JSON NOT NULL DEFAULT '[]',
    market_analysis JSON NOT NULL DEFAULT '{}',
    summary TEXT,
    price_recommendation_low REAL,
    price_recommendation_mid REAL,
    price_recommendation_high REAL,
    market_conditions TEXT,
    avg_days_on_market REAL,
    price_per_sqft REAL,
    list_to_sale_ratio REAL,
    confidence INTEGER CHECK (confidence >= 0 AND confidence <= 100),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Market reports table
CREATE TABLE IF NOT EXISTS market_reports (
    id TEXT PRIMARY KEY,
    area TEXT NOT NULL,
    property_types JSON,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    generated_by TEXT NOT NULL,
    analysis_data JSON NOT NULL DEFAULT '{}',
    summary TEXT,
    key_findings JSON,
    recommendations JSON,
    data_sources JSON,
    confidence INTEGER CHECK (confidence >= 0 AND confidence <= 100),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for CMA and market reports
CREATE INDEX idx_cma_reports_subject_address ON cma_reports(subject_address);
CREATE INDEX idx_cma_reports_status ON cma_reports(status);
CREATE INDEX idx_cma_reports_created_at ON cma_reports(created_at);
CREATE INDEX idx_market_reports_area ON market_reports(area);
CREATE INDEX idx_market_reports_period ON market_reports(period_start, period_end);
CREATE INDEX idx_market_reports_created_at ON market_reports(created_at);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS leads_updated_at 
AFTER UPDATE ON leads
BEGIN
    UPDATE leads SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS communication_drafts_updated_at 
AFTER UPDATE ON communication_drafts
BEGIN
    UPDATE communication_drafts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS cma_reports_updated_at 
AFTER UPDATE ON cma_reports
BEGIN
    UPDATE cma_reports SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
