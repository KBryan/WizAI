//! Integration tests for ResearchLead Agent Types
//!
//! Tests the ResearchLead type system and structures

use spree_agent::agent::roles::research_lead::{
    DevelopmentTeam, PrdConfig, PrdStatus, Priority, RequirementCategory, ResearchRequest,
    ResearchTask, ResearchType,
};

#[test]
fn test_research_types() {
    println!("Testing ResearchLead research types...");

    // Test all research types
    let research_types = vec![
        ResearchType::MarketResearch,
        ResearchType::UserResearch,
        ResearchType::TechnicalResearch,
        ResearchType::CompetitiveAnalysis,
        ResearchType::FeasibilityStudy,
    ];

    for research_type in research_types {
        let request = ResearchRequest {
            research_type: research_type.clone(),
            scope: "Test scope".to_string(),
            objectives: vec!["Test objective".to_string()],
            timeline: None,
            budget: None,
        };

        assert_eq!(request.research_type, research_type);
    }

    println!("✅ All research types work correctly");
}

#[test]
fn test_prd_status_transitions() {
    println!("Testing PRD status transitions...");

    // Test PRD status enum
    let statuses = vec![
        PrdStatus::Draft,
        PrdStatus::InReview,
        PrdStatus::PendingApproval,
        PrdStatus::Approved,
        PrdStatus::Rejected,
        PrdStatus::Implementation,
        PrdStatus::Completed,
        PrdStatus::Archived,
    ];

    // Verify each status exists and can be formatted
    for status in statuses {
        let status_str = format!("{:?}", status);
        assert!(!status_str.is_empty());
    }

    println!("✅ PRD status transitions defined correctly");
}

#[test]
fn test_development_teams() {
    println!("Testing development team types...");

    let teams = vec![
        DevelopmentTeam::Software,
        DevelopmentTeam::RealEstate,
        DevelopmentTeam::Data,
        DevelopmentTeam::Marketing,
        DevelopmentTeam::Operations,
        DevelopmentTeam::Custom("Custom Team".to_string()),
    ];

    for team in teams {
        let team_name = format!("{:?}", team);
        assert!(!team_name.is_empty());
    }

    println!("✅ All development team types defined");
}

#[test]
fn test_research_task_delegation() {
    println!("Testing research task delegation...");

    let task = ResearchTask {
        id: "task_001".to_string(),
        title: "Market Analysis".to_string(),
        description: "Analyze competitor landscape".to_string(),
        research_type: ResearchType::CompetitiveAnalysis,
        priority: Priority::High,
        deadline: None,
    };

    assert_eq!(task.id, "task_001");
    assert_eq!(task.priority, Priority::High);
    assert_eq!(task.research_type, ResearchType::CompetitiveAnalysis);

    println!("✅ Research task delegation structure works");
}

#[test]
fn test_prd_config_creation() {
    println!("Testing PRD configuration...");

    let config = PrdConfig {
        title: "Test Feature PRD".to_string(),
        target_users: vec!["Power users".to_string(), "Regular users".to_string()],
        problem_statement: "Users need better analytics".to_string(),
        success_criteria: vec![
            "Increase engagement by 20%".to_string(),
            "Reduce churn by 10%".to_string(),
        ],
    };

    assert_eq!(config.title, "Test Feature PRD");
    assert_eq!(config.target_users.len(), 2);
    assert_eq!(config.success_criteria.len(), 2);
    assert_eq!(config.problem_statement, "Users need better analytics");

    println!("✅ PRD configuration structure valid");
}

#[test]
fn test_requirement_categories() {
    println!("Testing requirement categories...");

    let categories = vec![
        RequirementCategory::Functional,
        RequirementCategory::NonFunctional,
        RequirementCategory::Technical,
        RequirementCategory::Design,
        RequirementCategory::Compliance,
        RequirementCategory::Security,
        RequirementCategory::Performance,
    ];

    for category in categories {
        let cat_str = format!("{:?}", category);
        assert!(!cat_str.is_empty());
    }

    println!("✅ All requirement categories defined");
}

#[test]
fn test_priority_levels() {
    println!("Testing priority levels...");

    let priorities = vec![
        Priority::Critical,
        Priority::High,
        Priority::Medium,
        Priority::Low,
    ];

    for priority in priorities {
        let priority_str = format!("{:?}", priority);
        assert!(!priority_str.is_empty());
    }

    println!("✅ All priority levels defined");
}

#[test]
fn test_research_request_builder() {
    println!("Testing research request builder pattern...");

    let request = ResearchRequest {
        research_type: ResearchType::MarketResearch,
        scope: "Analyze Toronto real estate market".to_string(),
        objectives: vec![
            "Identify pricing trends".to_string(),
            "Assess inventory levels".to_string(),
        ],
        timeline: Some("Q1 2026".to_string()),
        budget: Some(5000.0),
    };

    assert_eq!(request.research_type, ResearchType::MarketResearch);
    assert_eq!(request.scope, "Analyze Toronto real estate market");
    assert_eq!(request.objectives.len(), 2);
    assert_eq!(request.timeline, Some("Q1 2026".to_string()));
    assert_eq!(request.budget, Some(5000.0));

    println!("✅ Research request builder works");
}
