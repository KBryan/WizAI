//! Web Scraping Module
//! Provides real-time web search and scraping capabilities for Real Estate Research
//!
//! Supported Providers:
//! - SerpAPI (Google Search, News, etc.)
//! - ScrapingBee (Alternative)
//! - Custom HTTP scraping (fallback)

pub mod client;
pub mod providers;
pub mod real_estate_search;

pub use client::*;
pub use providers::*;
pub use real_estate_search::*;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Search result from web scraping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: String,
    pub published_date: Option<String>,
    pub relevance_score: f32,
}

/// News article from search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub url: String,
    pub source: String,
    pub published_date: String,
    pub snippet: String,
    pub image_url: Option<String>,
}

/// Market data from search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataPoint {
    pub metric: String,
    pub value: String,
    pub source: String,
    pub date: String,
    pub confidence: f32,
}

/// Web scraping configuration
#[derive(Debug, Clone)]
pub struct ScrapingConfig {
    pub serpapi_key: Option<String>,
    pub scrapingbee_key: Option<String>,
    pub default_provider: Provider,
    pub rate_limit_requests_per_minute: u32,
    pub cache_duration_seconds: u64,
}

impl Default for ScrapingConfig {
    fn default() -> Self {
        Self {
            serpapi_key: None,
            scrapingbee_key: None,
            default_provider: Provider::SerpApi,
            rate_limit_requests_per_minute: 60,
            cache_duration_seconds: 3600, // 1 hour cache
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provider {
    SerpApi,
    ScrapingBee,
    Custom,
}

impl ScrapingConfig {
    pub fn from_env() -> Self {
        use std::env;
        
        Self {
            serpapi_key: env::var("SERPAPI_KEY").ok(),
            scrapingbee_key: env::var("SCRAPINGBEE_KEY").ok(),
            default_provider: Provider::SerpApi,
            rate_limit_requests_per_minute: 60,
            cache_duration_seconds: 3600,
        }
    }
    
    pub fn is_configured(&self) -> bool {
        self.serpapi_key.is_some() || self.scrapingbee_key.is_some()
    }
    
    pub fn get_active_provider(&self) -> Option<Provider> {
        if self.serpapi_key.is_some() {
            Some(Provider::SerpApi)
        } else if self.scrapingbee_key.is_some() {
            Some(Provider::ScrapingBee)
        } else {
            None
        }
    }
}

/// Unified search request
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub location: Option<String>,
    pub time_range: Option<String>, // e.g., "1d", "1w", "1m", "1y"
    pub result_count: u32,
    pub search_type: SearchType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchType {
    General,
    News,
    Images,
    RealEstate,
}

impl SearchRequest {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            location: None,
            time_range: None,
            result_count: 10,
            search_type: SearchType::General,
        }
    }
    
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
    
    pub fn with_time_range(mut self, range: impl Into<String>) -> Self {
        self.time_range = Some(range.into());
        self
    }
    
    pub fn with_count(mut self, count: u32) -> Self {
        self.result_count = count;
        self
    }
    
    pub fn with_type(mut self, search_type: SearchType) -> Self {
        self.search_type = search_type;
        self
    }
}
