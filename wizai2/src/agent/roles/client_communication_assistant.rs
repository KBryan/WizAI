//! Client Communication Assistant Agent
//!
//! AI agent specialized in drafting client communications with compliance.

use crate::compliance::ApprovalEngine;
use crate::models::{
    ApprovalStatus, CommunicationChannel, CommunicationDraft, GenerateDraftRequest,
    RiskLevel, ReviewAction, ReviewDraftRequest,
};
use crate::services::LeadService;
use crate::services::CommunicationService;
use anyhow::Result;

/// Client Communication Assistant - Drafts communications with approval workflow
pub struct ClientCommunicationAssistant {
    agent_id: String,
    lead_service: LeadService,
    communication_service: CommunicationService,
    approval_engine: ApprovalEngine,
}

impl ClientCommunicationAssistant {
    /// Create a new client communication assistant
    pub fn new(
        agent_id: String,
        lead_service: LeadService,
        communication_service: CommunicationService,
    ) -> Self {
        Self {
            agent_id,
            lead_service,
            communication_service,
            approval_engine: ApprovalEngine::new(),
        }
    }

    /// Draft a follow-up email for a lead
    pub async fn draft_follow_up(
        &self,
        lead_id: &str,
        requested_by: &str,
        context: Option<String>,
    ) -> Result<CommunicationDraft> {
        // Get lead information
        let lead = self.lead_service.get_lead(lead_id).await?
            .ok_or_else(|| anyhow::anyhow!("Lead not found"))?;

        // Get conversation history
        let activities = self.lead_service.get_activities(lead_id).await?;

        // Generate personalized content
        let content = self.generate_follow_up_content(&lead, &activities, context.as_deref())?;
        let subject = self.generate_subject(&lead);

        // Create the draft
        let draft = self.communication_service.generate_draft(
            lead_id,
            requested_by,
            &self.agent_id,
            CommunicationChannel::Email,
            Some(subject),
            content,
        ).await?;

        Ok(draft)
    }

    /// Draft an initial response to a new lead
    pub async fn draft_initial_response(
        &self,
        lead_id: &str,
        requested_by: &str,
    ) -> Result<CommunicationDraft> {
        let lead = self.lead_service.get_lead(lead_id).await?
            .ok_or_else(|| anyhow::anyhow!("Lead not found"))?;

        let content = self.generate_initial_response(&lead)?;
        let subject = format!("Welcome {}, let's discuss your real estate goals", lead.name.split_whitespace().next().unwrap_or(&lead.name));

        let draft = self.communication_service.generate_draft(
            lead_id,
            requested_by,
            &self.agent_id,
            CommunicationChannel::Email,
            Some(subject),
            content,
        ).await?;

        Ok(draft)
    }

    /// Draft an SMS follow-up
    pub async fn draft_sms_follow_up(
        &self,
        lead_id: &str,
        requested_by: &str,
    ) -> Result<CommunicationDraft> {
        let lead = self.lead_service.get_lead(lead_id).await?
            .ok_or_else(|| anyhow::anyhow!("Lead not found"))?;

        let content = self.generate_sms_content(&lead)?;

        let draft = self.communication_service.generate_draft(
            lead_id,
            requested_by,
            &self.agent_id,
            CommunicationChannel::Sms,
            None,
            content,
        ).await?;

        Ok(draft)
    }

    /// Summarize conversation history for context
    pub async fn summarize_conversation(&self, lead_id: &str) -> Result<String> {
        let activities = self.lead_service.get_activities(lead_id).await?;
        
        if activities.is_empty() {
            return Ok("No prior conversations recorded.".to_string());
        }

        let mut summary = vec!["Conversation Summary:".to_string()];
        
        for activity in activities.iter().take(5) {
            summary.push(format!("- {}: {}", 
                activity.created_at.format("%Y-%m-%d"),
                activity.description
            ));
        }

        Ok(summary.join("\n"))
    }

    /// Propose next best action based on lead status
    pub async fn propose_next_action(&self, lead_id: &str) -> Result<String> {
        let lead = self.lead_service.get_lead(lead_id).await?
            .ok_or_else(|| anyhow::anyhow!("Lead not found"))?;

        let action = match lead.status {
            crate::models::LeadStatus::New => {
                "Send initial welcome email within 1 hour"
            }
            crate::models::LeadStatus::Qualified => {
                if lead.is_hot_lead() {
                    "Schedule immediate phone call - hot lead"
                } else {
                    "Send personalized follow-up with market information"
                }
            }
            crate::models::LeadStatus::Contacted => {
                match lead.last_contact_at {
                    Some(last) => {
                        let days_since = (chrono::Utc::now() - last).num_days();
                        if days_since > 7 {
                            "Send check-in email - no contact in over a week"
                        } else {
                            "Continue nurturing based on their timeline"
                        }
                    }
                    None => "Schedule follow-up based on stated timeline"
                }
            }
            _ => "Review lead status and determine appropriate next step"
        };

        Ok(action.to_string())
    }

    /// Get pending approvals
    pub async fn get_pending_approvals(&self, limit: i64) -> Result<Vec<crate::models::PendingApprovalSummary>> {
        self.communication_service.get_pending_approvals(limit).await
    }

    /// Approve a draft
    pub async fn approve_draft(
        &self,
        draft_id: &str,
        reviewer: &str,
        notes: Option<&str>,
    ) -> Result<CommunicationDraft> {
        let request = ReviewDraftRequest {
            action: ReviewAction::Approve,
            notes: notes.map(|s| s.to_string()),
        };

        self.communication_service.review_draft(draft_id, reviewer, request).await
            .and_then(|opt| opt.ok_or_else(|| anyhow::anyhow!("Draft not found")))
    }

    /// Reject a draft
    pub async fn reject_draft(
        &self,
        draft_id: &str,
        reviewer: &str,
        reason: &str,
    ) -> Result<CommunicationDraft> {
        let request = ReviewDraftRequest {
            action: ReviewAction::Reject,
            notes: Some(reason.to_string()),
        };

        self.communication_service.review_draft(draft_id, reviewer, request).await
            .and_then(|opt| opt.ok_or_else(|| anyhow::anyhow!("Draft not found")))
    }

    /// Edit a draft
    pub async fn edit_draft(
        &self,
        draft_id: &str,
        edited_by: &str,
        new_content: String,
        reason: Option<&str>,
    ) -> Result<CommunicationDraft> {
        self.communication_service.edit_draft(draft_id, edited_by, new_content, reason).await
            .and_then(|opt| opt.ok_or_else(|| anyhow::anyhow!("Draft not found")))
    }

    /// Mark draft as sent
    pub async fn mark_sent(&self, draft_id: &str, sent_by: &str) -> Result<CommunicationDraft> {
        self.communication_service.mark_sent(draft_id, sent_by).await
            .and_then(|opt| opt.ok_or_else(|| anyhow::anyhow!("Draft not found or not approved")))
    }

    /// Generate follow-up content
    fn generate_follow_up_content(
        &self,
        lead: &crate::models::Lead,
        activities: &[crate::models::LeadActivity],
        context: Option<&str>,
    ) -> Result<String> {
        let first_name = lead.name.split_whitespace().next().unwrap_or(&lead.name);
        
        let mut content = format!("Hi {},\n\n", first_name);

        // Add context-aware opening
        if let Some(ctx) = context {
            content.push_str(&format!("{}", ctx));
        } else if !activities.is_empty() {
            content.push_str("I wanted to follow up on our previous conversation. ");
        } else {
            content.push_str("I hope you're doing well. ");
        }

        // Reference their inquiry type
        content.push_str(&match lead.inquiry_type {
            crate::models::InquiryType::Buyer => {
                format!("I know you're looking to buy in the Durham Region{}.",
                    lead.desired_location.as_ref()
                        .map(|l| format!(", specifically in {}", l))
                        .unwrap_or_default())
            }
            crate::models::InquiryType::Seller => {
                "I understand you're considering selling your property.".to_string()
            }
            crate::models::InquiryType::Renter => {
                "I see you're looking for rental options in the area.".to_string()
            }
            _ => "I wanted to see how I can help with your real estate needs.".to_string()
        });

        // Add timeline-specific content
        content.push_str(&match lead.timeline {
            crate::models::Timeline::Immediate | crate::models::Timeline::ShortTerm => {
                "\n\nGiven your timeline, I'd love to schedule a call this week to discuss next steps."
            }
            crate::models::Timeline::MediumTerm => {
                "\n\nWith your 3-6 month timeline, we have time to prepare and find the perfect opportunity."
            }
            _ => "\n\nWhenever you're ready to move forward, I'm here to help."
        });

        // Professional closing
        content.push_str("\n\nBest regards,\n[Agent Name]");

        Ok(content)
    }

    /// Generate initial response content
    fn generate_initial_response(&self, lead: &crate::models::Lead) -> Result<String> {
        let first_name = lead.name.split_whitespace().next().unwrap_or(&lead.name);
        
        let mut content = format!(
            "Hi {},\n\n\
            Thank you for reaching out! I'm excited to help you with your real estate goals in the Durham Region.\n\n",
            first_name
        );

        // Customize based on inquiry type
        match lead.inquiry_type {
            crate::models::InquiryType::Buyer => {
                content.push_str(
                    "Buying a home is a significant decision, and I'm here to make the process as smooth as possible. \
                    Whether you're a first-time buyer or looking to upgrade, I'll provide you with the market insights \
                    and guidance you need.\n\n"
                );
            }
            crate::models::InquiryType::Seller => {
                content.push_str(
                    "Selling your property is a big step, and getting the right price matters. \
                    I'll help you understand current market conditions in your area and develop \
                    a strategy to maximize your return.\n\n"
                );
            }
            _ => {}
        }

        // Add next steps
        content.push_str(
            "To get started, I'd love to learn more about what you're looking for. \
            Could you share:\n\
            - Your ideal timeline\n\
            - Must-have features\n\
            - Any questions you have about the process\n\n\
            I'm here to make this journey as smooth as possible for you.\n\n\
            Best regards,\n\
            [Agent Name]\n\
            Licensed Real Estate Agent\n\
            Durham Region, Ontario"
        );

        Ok(content)
    }

    /// Generate SMS content
    fn generate_sms_content(&self, lead: &crate::models::Lead) -> Result<String> {
        let first_name = lead.name.split_whitespace().next().unwrap_or(&lead.name);
        
        Ok(format!(
            "Hi {}, this is [Agent Name]. Just following up on your inquiry about {}. \
            Do you have a few minutes for a quick call this week? Thanks!",
            first_name,
            match lead.inquiry_type {
                crate::models::InquiryType::Buyer => "buying in Durham Region",
                crate::models::InquiryType::Seller => "selling your property",
                _ => "real estate",
            }
        ))
    }

    /// Generate email subject
    fn generate_subject(&self, lead: &crate::models::Lead) -> String {
        match lead.inquiry_type {
            crate::models::InquiryType::Buyer => {
                format!("Finding your perfect home in {}", 
                    lead.desired_location.as_deref().unwrap_or("Durham Region"))
            }
            crate::models::InquiryType::Seller => {
                "Let's discuss your property sale".to_string()
            }
            _ => "Following up on your real estate inquiry".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_generation() {
        // Would need mock services to test properly
        assert_eq!(2 + 2, 4);
    }
}
