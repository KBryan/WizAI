//! Services for AI Real Estate Team
//!
//! Business logic services that coordinate between agents, models, and storage.

pub mod communication_service;
pub mod lead_service;
pub mod market_research_service;
pub mod repliers_client;

pub use communication_service::CommunicationService;
pub use lead_service::LeadService;
pub use market_research_service::MarketResearchService;
pub use repliers_client::{
    ActiveListing, ActiveListingsFilter, ActiveListingsResult, AdvancedSearchCriteria,
    AreaStatistics, ComparableSubject, PriceHistoryEntry, PropertyDetails, RepliersClient,
    RepliersMarketStats, RepliersProperty,
};
