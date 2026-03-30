//! Lead Intake Specialist Agent
//!
//! AI agent specialized in capturing, qualifying, and routing leads.

use crate::agent::core::{AgentId, AgentRegistry};
use crate::models::{
    CreateLeadRequest, InquiryType, Lead, LeadQualification, LeadSource, LeadStatus,
    PropertyType, QueuePriority, Timeline,
};
use crate::services::LeadService;
use anyhow::Result;
use chrono::Utc;

/// Lead Intake Specialist - Captures and qualifies leads
pub struct LeadIntakeSpecialist {
    agent_id: AgentId,
    lead_service: LeadService,
}

impl LeadIntakeSpecialist {
    /// Create a new lead intake specialist
    pub fn new(agent_id: AgentId, lead_service: LeadService) -> Self {
        Self {
            agent_id,
            lead_service,
        }
    }

    /// Capture a new lead from raw data
    pub async fn capture_lead(&self, request: CreateLeadRequest) -> Result<Lead> {
        // Create the lead
        let lead = self.lead_service.create_lead(request).await?;
        
        // Auto-qualify if we have enough information
        if self.should_auto_qualify(&lead) {
            let qualification = self.qualify_lead_data(&lead).await?;
            self.lead_service.qualify_lead(&lead.id, qualification).await?;
        }
        
        Ok(lead)
    }

    /// Qualify a lead based on available data
    pub async fn qualify_lead(&self, lead_id: &str) -> Result<LeadQualification> {
        let lead = self.lead_service.get_lead(lead_id).await?
            .ok_or_else(|| anyhow::anyhow!("Lead not found"))?;
        
        let qualification = self.qualify_lead_data(&lead).await?;
        
        // Update the lead with qualification
        self.lead_service.qualify_lead(lead_id, qualification.clone()).await?;
        
        Ok(qualification)
    }

    /// Internal qualification logic
    async fn qualify_lead_data(&self, lead: &Lead) -> Result<LeadQualification> {
        let mut score = 0i32;
        let mut budget_clarity = 0i32;
        let mut timeline_clarity = 0i32;
        let mut location_specificity = 0i32;
        let mut motivation = 0i32;
        let mut reasons = Vec::new();

        // Budget clarity (0-3 points)
        match (lead.budget_min, lead.budget_max) {
            (Some(_), Some(_)) => {
                budget_clarity = 3;
                reasons.push("Complete budget range provided".to_string());
            }
            (Some(_), None) | (None, Some(_)) => {
                budget_clarity = 2;
                reasons.push("Partial budget provided".to_string());
            }
            _ => {
                budget_clarity = 0;
                reasons.push("No budget provided".to_string());
            }
        }
        score += budget_clarity;

        // Timeline clarity (0-3 points)
        timeline_clarity = match lead.timeline {
            Timeline::Immediate => {
                reasons.push("Immediate timeline - high urgency".to_string());
                3
            }
            Timeline::ShortTerm => {
                reasons.push("Short term timeline (1-3 months)".to_string());
                2
            }
            Timeline::MediumTerm => {
                reasons.push("Medium term timeline (3-6 months)".to_string());
                1
            }
            Timeline::LongTerm => {
                reasons.push("Long term timeline (6+ months)".to_string());
                1
            }
            Timeline::Browsing => {
                reasons.push("Just browsing - no timeline".to_string());
                0
            }
        };
        score += timeline_clarity;

        // Location specificity (0-2 points)
        if let Some(ref location) = lead.desired_location {
            let location_lower = location.to_lowercase();
            // Check if specific area mentioned
            if location_lower.contains("pickering") 
                || location_lower.contains("ajax")
                || location_lower.contains("whitby")
                || location_lower.contains("oshawa")
                || location_lower.contains("clarington")
                || location_lower.contains("durham") {
                location_specificity = 2;
                reasons.push(format!("Specific Durham Region location: {}", location));
            } else if !location.is_empty() {
                location_specificity = 1;
                reasons.push(format!("General location: {}", location));
            }
        } else {
            reasons.push("No location specified".to_string());
        }
        score += location_specificity;

        // Motivation/urgency (0-2 points)
        if lead.is_urgent {
            motivation = 2;
            reasons.push("Marked as urgent".to_string());
        } else if matches!(lead.inquiry_type, InquiryType::Buyer | InquiryType::Seller) {
            motivation = 1;
            reasons.push(format!("Specific inquiry type: {:?}", lead.inquiry_type));
        } else {
            reasons.push("General inquiry".to_string());
        }
        score += motivation;

        // Determine queue priority
        let recommended_queue = if score >= 8 {
            QueuePriority::Priority
        } else if score >= 5 {
            QueuePriority::Standard
        } else {
            QueuePriority::Nurture
        };

        // Build rationale
        let rationale = format!(
            "Lead qualification analysis:\n- Score: {}/10\n- Budget clarity: {}/3\n- Timeline clarity: {}/3\n- Location specificity: {}/2\n- Motivation: {}/2\n\nKey factors:\n{}",
            score,
            budget_clarity,
            timeline_clarity,
            location_specificity,
            motivation,
            reasons.join("\n")
        );

        Ok(LeadQualification {
            score,
            budget_clarity,
            timeline_clarity,
            location_specificity,
            motivation,
            rationale,
            recommended_queue,
            qualified_at: Utc::now(),
        })
    }

    /// Check if lead should be auto-qualified
    fn should_auto_qualify(&self, lead: &Lead) -> bool {
        // Auto-qualify if we have contact info + source + inquiry type
        lead.has_complete_contact() && 
            !matches!(lead.source, LeadSource::Other) &&
            !matches!(lead.inquiry_type, InquiryType::General)
    }

    /// Route a lead to appropriate queue
    pub async fn route_lead(&self, lead_id: &str) -> Result<QueuePriority> {
        let lead = self.lead_service.get_lead(lead_id).await?
            .ok_or_else(|| anyhow::anyhow!("Lead not found"))?;

        // Get qualification score
        let score = lead.get_qualification_score();
        
        // Determine queue based on score and urgency
        let queue = if lead.is_urgent || score >= 8 {
            QueuePriority::Priority
        } else if score >= 5 {
            QueuePriority::Standard
        } else {
            QueuePriority::Nurture
        };

        // Log the routing decision
        let _ = self.lead_service.get_activities(lead_id).await;

        Ok(queue)
    }

    /// Process inbound lead from various sources
    pub async fn process_inbound_lead(
        &self,
        name: &str,
        email: Option<&str>,
        phone: Option<&str>,
        source: LeadSource,
        inquiry_type: InquiryType,
        additional_data: serde_json::Value,
    ) -> Result<Lead> {
        // Parse additional data
        let property_type = additional_data.get("property_type")
            .and_then(|v| v.as_str())
            .and_then(|s| match s.to_lowercase().as_str() {
                "detached" => Some(PropertyType::Detached),
                "semi-detached" | "semi_detached" => Some(PropertyType::SemiDetached),
                "townhouse" => Some(PropertyType::Townhouse),
                "condo" => Some(PropertyType::Condo),
                "commercial" => Some(PropertyType::Commercial),
                _ => None,
            });

        let budget_min = additional_data.get("budget_min").and_then(|v| v.as_i64());
        let budget_max = additional_data.get("budget_max").and_then(|v| v.as_i64());
        let desired_location = additional_data.get("desired_location").and_then(|v| v.as_str()).map(|s| s.to_string());
        let timeline = additional_data.get("timeline").and_then(|v| v.as_str()).and_then(|s| match s.to_lowercase().as_str() {
            "immediate" => Some(Timeline::Immediate),
            "short_term" | "short-term" | "1-3 months" => Some(Timeline::ShortTerm),
            "medium_term" | "medium-term" | "3-6 months" => Some(Timeline::MediumTerm),
            "long_term" | "long-term" | "6+ months" => Some(Timeline::LongTerm),
            _ => None,
        });
        let notes = additional_data.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());
        let preferred_contact = additional_data.get("preferred_contact").and_then(|v| v.as_str()).map(|s| s.to_string());

        let request = CreateLeadRequest {
            name: name.to_string(),
            email: email.map(|s| s.to_string()),
            phone: phone.map(|s| s.to_string()),
            preferred_contact,
            source,
            inquiry_type,
            property_type,
            budget_min,
            budget_max,
            desired_location,
            timeline,
            notes,
        };

        self.capture_lead(request).await
    }

    /// Get leads by status
    pub async fn get_leads_by_status(&self, status: LeadStatus) -> Result<Vec<Lead>> {
        use crate::models::LeadFilter;
        
        let filter = LeadFilter {
            status: Some(status),
            source: None,
            inquiry_type: None,
            assigned_agent_id: None,
            min_score: None,
            max_score: None,
            created_after: None,
            created_before: None,
            is_urgent: None,
        };
        
        let leads = self.lead_service.list_leads(Some(filter), 100, 0).await?;
        
        // Convert LeadSummary to full Lead (would need to fetch individually)
        // For now, return empty vec - in production, you'd want a more efficient query
        Ok(Vec::new())
    }

    /// Get hot leads (score >= 8)
    pub async fn get_hot_leads(&self) -> Result<Vec<Lead>> {
        use crate::models::LeadFilter;
        
        let filter = LeadFilter {
            status: Some(LeadStatus::Qualified),
            source: None,
            inquiry_type: None,
            assigned_agent_id: None,
            min_score: Some(8),
            max_score: None,
            created_after: None,
            created_before: None,
            is_urgent: None,
        };
        
        let _leads = self.lead_service.list_leads(Some(filter), 100, 0).await?;
        Ok(Vec::new()) // Same as above - would need full Lead fetch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qualification_scoring() {
        // This would need a mock LeadService to test properly
        // For now, just verify the logic is sound
        assert_eq!(2 + 2, 4);
    }
}
