//! Data models for AI Real Estate Team
//!
//! This module contains all data structures for leads, communications, market research,
//! and compliance auditing.

pub mod api;
pub mod communication;
pub mod lead;
pub mod market;

// Re-export commonly used types
pub use api::{
    get_neighborhoods, get_supported_areas, is_area_supported, ActiveListingsApiResponse,
    ActiveListingsQuery, ApiError, ApiRequestLog, ApiResponse, AreaStatisticsApiResponse,
    AreaStatisticsQuery, CmaJobStatus, CmaStatusResponse, ComparableApiModel, ComparablesQuery,
    ComparablesResponse, CreateCmaRequest, CreateCmaResponse, ListCmasQuery, LogApiRequest,
    MarketStatus, PaginatedList, PropertyDetailsApiResponse, SupportedAreasResponse,
    TargetProperty, TrendsQuery, TrendsResponse,
};
pub use communication::{
    ApprovalStatus, CommunicationChannel, CommunicationDraft, CommunicationFilter,
    CommunicationTemplate, DraftEditHistory, GenerateDraftRequest, PendingApprovalSummary,
    ReviewAction, ReviewDraftRequest, RiskLevel, SentCommunication,
};
pub use lead::{
    CreateLeadRequest, InquiryType, Lead, LeadActivity, LeadFilter, LeadQualification, LeadSource,
    LeadStatus, LeadSummary, PropertyType, QueuePriority, Timeline, UpdateLeadRequest,
};
pub use market::{
    BuyerProfile, CMAReport, CMAStatus, CMASummary, ComparableProperty, GenerateCMARequest,
    ListingStatus, MarketAnalysis, MarketReport, MarketSentiment, PropertyCategory,
    PropertyComparison, PropertyDetails, PropertySearchCriteria, SearchComparablesRequest,
    TrendDirection,
};
