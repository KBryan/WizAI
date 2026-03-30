-- Migration: Communications and Approval Tables
-- Description: Create tables for AI-generated communications and approval workflow

-- Communication drafts table
CREATE TABLE IF NOT EXISTS communication_drafts (
    id TEXT PRIMARY KEY,
    lead_id TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    generated_by TEXT NOT NULL,
    channel TEXT NOT NULL CHECK (channel IN ('email', 'sms', 'phone', 'in_person', 'video')),
    subject TEXT,
    content TEXT NOT NULL,
    risk_level TEXT NOT NULL CHECK (risk_level IN ('low', 'medium', 'high')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'pending', 'auto_approved', 'approved', 'rejected', 'sent', 'archived')),
    confidence INTEGER CHECK (confidence >= 0 AND confidence <= 100),
    risk_reason TEXT,
    reviewed_by TEXT,
    review_notes TEXT,
    reviewed_at DATETIME,
    final_content TEXT,
    sent_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (lead_id) REFERENCES leads(id) ON DELETE CASCADE
);

-- Communication templates table
CREATE TABLE IF NOT EXISTS communication_templates (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    channel TEXT NOT NULL CHECK (channel IN ('email', 'sms', 'phone', 'in_person', 'video')),
    subject_template TEXT,
    body_template TEXT NOT NULL,
    default_risk_level TEXT NOT NULL CHECK (default_risk_level IN ('low', 'medium', 'high')),
    variables JSON,
    is_active BOOLEAN DEFAULT TRUE,
    created_by TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Draft edit history
CREATE TABLE IF NOT EXISTS draft_edit_history (
    id TEXT PRIMARY KEY,
    draft_id TEXT NOT NULL,
    previous_content TEXT NOT NULL,
    new_content TEXT NOT NULL,
    edited_by TEXT NOT NULL,
    edit_reason TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (draft_id) REFERENCES communication_drafts(id) ON DELETE CASCADE
);

-- Sent communications log
CREATE TABLE IF NOT EXISTS sent_communications (
    id TEXT PRIMARY KEY,
    draft_id TEXT NOT NULL,
    lead_id TEXT NOT NULL,
    channel TEXT NOT NULL CHECK (channel IN ('email', 'sms', 'phone', 'in_person', 'video')),
    subject TEXT,
    content TEXT NOT NULL,
    sent_by TEXT NOT NULL,
    sent_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    delivery_status TEXT,
    opened_at DATETIME,
    clicked_at DATETIME,
    FOREIGN KEY (draft_id) REFERENCES communication_drafts(id) ON DELETE CASCADE,
    FOREIGN KEY (lead_id) REFERENCES leads(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX idx_communication_drafts_lead_id ON communication_drafts(lead_id);
CREATE INDEX idx_communication_drafts_status ON communication_drafts(status);
CREATE INDEX idx_communication_drafts_risk_level ON communication_drafts(risk_level);
CREATE INDEX idx_communication_drafts_created_at ON communication_drafts(created_at);
CREATE INDEX idx_communication_drafts_reviewed_by ON communication_drafts(reviewed_by);
CREATE INDEX idx_sent_communications_lead_id ON sent_communications(lead_id);
CREATE INDEX idx_sent_communications_sent_at ON sent_communications(sent_at);

-- Insert default templates
INSERT INTO communication_templates (id, name, description, channel, subject_template, body_template, default_risk_level, created_by)
VALUES 
('tmpl-001', 'Initial Buyer Consultation', 'Welcome email for new buyer leads', 'email', 'Welcome to Your Home Search Journey', 'Hi {{name}},

Thank you for reaching out about finding your perfect home in {{location}}. I''m excited to help you navigate the Durham Region real estate market.

I''d love to learn more about what you''re looking for. Could you share:
- Your ideal timeline
- Must-have features
- Preferred neighborhoods

Looking forward to connecting!

Best regards,
{{agent_name}}', 'low', 'system'),

('tmpl-002', 'Seller Listing Inquiry', 'Response to seller consultation requests', 'email', 'Let''s Discuss Selling Your Home', 'Hi {{name}},

Thank you for considering me to help sell your property at {{address}}. I''d be honored to represent you in this important transaction.

I''ll prepare a comprehensive market analysis for your property and we can discuss pricing strategy, staging recommendations, and marketing approach.

When would be a good time for us to connect?

Best regards,
{{agent_name}}', 'low', 'system'),

('tmpl-003', 'Follow-Up Reminder', 'Gentle follow-up for leads not contacted recently', 'email', 'Following Up on Your Real Estate Inquiry', 'Hi {{name}},

I wanted to follow up on your recent inquiry about {{inquiry_type}}. I know the real estate market can be overwhelming, and I''m here to help guide you through the process.

Have your plans or timeline changed at all? I''d love to update my records and make sure I''m providing you with relevant information.

Feel free to reply or give me a call anytime.

Best regards,
{{agent_name}}', 'medium', 'system');
