//! Approval engine for AI-generated content
//!
//! Risk classification and approval queue management.

use crate::models::{ApprovalStatus, CommunicationDraft, RiskLevel};
use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;

/// Engine for classifying risk and managing approvals
pub struct ApprovalEngine {
    /// Keywords that trigger high-risk classification
    high_risk_keywords: HashSet<String>,
    /// Keywords that trigger medium-risk classification
    medium_risk_keywords: HashSet<String>,
    /// Auto-approval enabled for low-risk content
    auto_approve_low_risk: bool,
}

impl ApprovalEngine {
    /// Create a new approval engine with default risk keywords
    pub fn new() -> Self {
        let mut high_risk = HashSet::new();
        high_risk.insert("price recommendation".to_string());
        high_risk.insert("market value".to_string());
        high_risk.insert("pricing advice".to_string());
        high_risk.insert("list at".to_string());
        high_risk.insert("should list".to_string());
        high_risk.insert("offer".to_string());
        high_risk.insert("negotiate".to_string());
        high_risk.insert("contract".to_string());
        high_risk.insert("agreement".to_string());
        high_risk.insert("legal".to_string());
        high_risk.insert("warranty".to_string());
        high_risk.insert("disclosure".to_string());
        high_risk.insert("guarantee".to_string());
        high_risk.insert("appreciation".to_string());
        high_risk.insert("investment return".to_string());
        high_risk.insert("roi".to_string());
        high_risk.insert("will increase".to_string());
        high_risk.insert("guaranteed".to_string());
        // Fair housing sensitive terms
        high_risk.insert("family".to_string());
        high_risk.insert("children".to_string());
        high_risk.insert("safe neighborhood".to_string());
        high_risk.insert("good schools".to_string());
        high_risk.insert("desirable".to_string());
        high_risk.insert("exclusive".to_string());

        let mut medium_risk = HashSet::new();
        medium_risk.insert("price".to_string());
        medium_risk.insert("pricing".to_string());
        medium_risk.insert("market".to_string());
        medium_risk.insert("value".to_string());
        medium_risk.insert("cost".to_string());
        medium_risk.insert("savings".to_string());
        medium_risk.insert("mortgage".to_string());
        medium_risk.insert("financing".to_string());
        medium_risk.insert("interest rate".to_string());
        medium_risk.insert("monthly payment".to_string());
        medium_risk.insert("down payment".to_string());
        medium_risk.insert("closing costs".to_string());
        medium_risk.insert("commission".to_string());
        medium_risk.insert("fee".to_string());

        Self {
            high_risk_keywords: high_risk,
            medium_risk_keywords: medium_risk,
            auto_approve_low_risk: true,
        }
    }

    /// Classify the risk level of content
    pub fn classify_risk(&self, content: &str, channel: &str) -> RiskClassification {
        let content_lower = content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();

        // Check for high-risk keywords
        let mut high_risk_matches = Vec::new();
        let mut medium_risk_matches = Vec::new();

        // Check phrases (2-3 word combinations)
        for window_size in 2..=4 {
            for window in words.windows(window_size) {
                let phrase = window.join(" ");
                if self.high_risk_keywords.contains(&phrase) {
                    high_risk_matches.push(phrase);
                } else if self.medium_risk_keywords.contains(&phrase) {
                    medium_risk_matches.push(phrase);
                }
            }
        }

        // Check individual words
        for word in &words {
            if self.high_risk_keywords.contains(*word) {
                high_risk_matches.push(word.to_string());
            } else if self.medium_risk_keywords.contains(*word) {
                medium_risk_matches.push(word.to_string());
            }
        }

        // Check for dollar amounts (pricing signals)
        let dollar_regex =
            Regex::new(r"\$[\d,]+(?:\.\d{2})?(?:\s*(?:k|thousand|million|m))?").unwrap();
        let dollar_matches: Vec<_> = dollar_regex.find_iter(&content_lower).collect();

        // Determine risk level
        let (risk_level, reason) = if !high_risk_matches.is_empty() {
            let unique_matches: HashSet<_> = high_risk_matches.iter().cloned().collect();
            (
                RiskLevel::High,
                format!(
                    "High-risk keywords detected: {}",
                    unique_matches.into_iter().collect::<Vec<_>>().join(", ")
                ),
            )
        } else if !medium_risk_matches.is_empty() || dollar_matches.len() > 1 {
            let unique_matches: HashSet<_> = medium_risk_matches.iter().cloned().collect();
            let mut reason_parts: Vec<String> = unique_matches.into_iter().collect();
            if dollar_matches.len() > 1 {
                reason_parts.push(format!("{} dollar amounts mentioned", dollar_matches.len()));
            }
            (
                RiskLevel::Medium,
                format!("Medium-risk content: {}", reason_parts.join(", ")),
            )
        } else {
            (RiskLevel::Low, "No risk keywords detected".to_string())
        };

        RiskClassification {
            level: risk_level,
            reason: Some(reason),
            confidence: self.calculate_confidence(&high_risk_matches, &medium_risk_matches),
        }
    }

    /// Calculate confidence score for the classification
    fn calculate_confidence(
        &self,
        high_risk_matches: &[String],
        medium_risk_matches: &[String],
    ) -> i32 {
        let high_weight = high_risk_matches.len() as i32 * 20;
        let medium_weight = medium_risk_matches.len() as i32 * 10;
        let confidence = 50 + high_weight + medium_weight;
        confidence.min(100)
    }

    /// Check if auto-approval is enabled for this risk level
    pub fn should_auto_approve(&self, risk_level: &RiskLevel) -> bool {
        matches!(risk_level, RiskLevel::Low) && self.auto_approve_low_risk
    }

    /// Set auto-approval policy
    pub fn set_auto_approve_low_risk(&mut self, enabled: bool) {
        self.auto_approve_low_risk = enabled;
    }
}

impl Default for ApprovalEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Risk classification result
#[derive(Debug, Clone)]
pub struct RiskClassification {
    /// Risk level
    pub level: RiskLevel,
    /// Reason for classification
    pub reason: Option<String>,
    /// Confidence score (0-100)
    pub confidence: i32,
}

/// Approval queue management
pub struct ApprovalQueue {
    /// Pending drafts awaiting approval
    pending: Vec<CommunicationDraft>,
}

impl ApprovalQueue {
    /// Create a new approval queue
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Add a draft to the queue
    pub fn add(&mut self, draft: CommunicationDraft) {
        self.pending.push(draft);
        // Re-sort by risk level and age
        self.pending.sort_by(|a, b| {
            let risk_order = match (&a.risk_level, &b.risk_level) {
                (RiskLevel::High, RiskLevel::High) => std::cmp::Ordering::Equal,
                (RiskLevel::High, _) => std::cmp::Ordering::Less,
                (_, RiskLevel::High) => std::cmp::Ordering::Greater,
                (RiskLevel::Medium, RiskLevel::Medium) => std::cmp::Ordering::Equal,
                (RiskLevel::Medium, _) => std::cmp::Ordering::Less,
                (_, RiskLevel::Medium) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            };
            risk_order.then_with(|| a.created_at.cmp(&b.created_at))
        });
    }

    /// Get all pending drafts
    pub fn get_pending(&self) -> &[CommunicationDraft] {
        &self.pending
    }

    /// Get pending drafts by risk level
    pub fn get_pending_by_risk(&self, risk_level: &RiskLevel) -> Vec<&CommunicationDraft> {
        self.pending
            .iter()
            .filter(|d| d.risk_level == *risk_level && d.status == ApprovalStatus::Pending)
            .collect()
    }

    /// Remove a draft from the queue by ID
    pub fn remove(&mut self, draft_id: &str) -> Option<CommunicationDraft> {
        if let Some(index) = self.pending.iter().position(|d| d.id == draft_id) {
            Some(self.pending.remove(index))
        } else {
            None
        }
    }

    /// Get count of pending drafts
    pub fn count(&self) -> usize {
        self.pending.len()
    }

    /// Get count by risk level
    pub fn count_by_risk(&self, risk_level: &RiskLevel) -> usize {
        self.pending
            .iter()
            .filter(|d| d.risk_level == *risk_level)
            .count()
    }
}

impl Default for ApprovalQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_risk_classification() {
        let engine = ApprovalEngine::new();
        let result =
            engine.classify_risk("Just following up on our appointment tomorrow.", "email");
        assert!(matches!(result.level, RiskLevel::Low));
    }

    #[test]
    fn test_medium_risk_classification() {
        let engine = ApprovalEngine::new();
        let result = engine.classify_risk(
            "I think you should consider the price range of $500k-$600k.",
            "email",
        );
        assert!(matches!(result.level, RiskLevel::Medium));
    }

    #[test]
    fn test_high_risk_classification() {
        let engine = ApprovalEngine::new();
        let result = engine.classify_risk(
            "I recommend listing at $850,000 based on market value.",
            "email",
        );
        assert!(matches!(result.level, RiskLevel::High));
    }

    #[test]
    fn test_fair_housing_risk() {
        let engine = ApprovalEngine::new();
        let result = engine.classify_risk(
            "This is a great area for families with good schools.",
            "email",
        );
        assert!(matches!(result.level, RiskLevel::High));
    }
}
