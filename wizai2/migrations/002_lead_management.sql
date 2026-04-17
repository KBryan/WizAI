-- Migration: Lead Management Tables
-- Description: Create tables for leads, communications, and audit logging

-- Lead table
CREATE TABLE IF NOT EXISTS leads (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    preferred_contact TEXT,
    status TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'qualified', 'contacted', 'converted', 'lost', 'archived')),
    source TEXT NOT NULL CHECK (source IN ('website', 'referral', 'portal', 'email', 'sms', 'walk_in', 'social_media', 'paid_ad', 'other')),
    inquiry_type TEXT NOT NULL DEFAULT 'general' CHECK (inquiry_type IN ('buyer', 'seller', 'renter', 'landlord', 'general')),
    property_type TEXT CHECK (property_type IN ('detached', 'semi_detached', 'townhouse', 'condo', 'commercial', 'multi_family', 'land', 'any')),
    budget_min INTEGER,
    budget_max INTEGER,
    desired_location TEXT,
    timeline TEXT CHECK (timeline IN ('immediate', 'short_term', 'medium_term', 'long_term', 'browsing')),
    notes TEXT,
    is_urgent BOOLEAN DEFAULT FALSE,
    qualification JSON,
    assigned_agent_id TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    first_contact_at DATETIME,
    last_contact_at DATETIME,
    contact_attempts INTEGER DEFAULT 0
);

-- Lead activity/history table
CREATE TABLE IF NOT EXISTS lead_activities (
    id TEXT PRIMARY KEY,
    lead_id TEXT NOT NULL,
    activity_type TEXT NOT NULL,
    description TEXT NOT NULL,
    performed_by TEXT,
    metadata JSON,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (lead_id) REFERENCES leads(id) ON DELETE CASCADE
);

-- Index for lead queries
CREATE INDEX idx_leads_status ON leads(status);
CREATE INDEX idx_leads_source ON leads(source);
CREATE INDEX idx_leads_assigned_agent ON leads(assigned_agent_id);
CREATE INDEX idx_leads_created_at ON leads(created_at);
CREATE INDEX idx_lead_activities_lead_id ON lead_activities(lead_id);
CREATE INDEX idx_lead_activities_created_at ON lead_activities(created_at);
