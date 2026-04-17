//! Web Scraping Client
//! Unified interface for multiple scraping providers

use super::*;
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Web Scraping Client
#[derive(Debug)]
pub struct WebScrapingClient {
    config: ScrapingConfig,
    http_client: Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    cache: Arc<Mutex<HashMap<String, (Vec<SearchResult>, Instant)>>>,
}

/// Rate limiter for API calls
#[derive(Debug)]
struct RateLimiter {
    last_request: Option<Instant>,
    min_interval: Duration,
}

impl WebScrapingClient {
    pub fn new(config: ScrapingConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        let min_interval = Duration::from_secs(60) / config.rate_limit_requests_per_minute;
        
        Self {
            config,
            http_client,
            rate_limiter: Arc::new(Mutex::new(RateLimiter {
                last_request: None,
                min_interval,
            })),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn from_env() -> Self {
        Self::new(ScrapingConfig::from_env())
    }
    
    /// Check if any provider is configured
    pub fn is_configured(&self) -> bool {
        self.config.is_configured()
    }
    
    /// Main search method - routes to appropriate provider
    pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        // Check cache first
        let cache_key = format!("{:?}:{}", request.search_type, request.query);
        {
            let cache = self.cache.lock().await;
            if let Some((results, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed().as_secs() < self.config.cache_duration_seconds {
                    debug!("Returning cached results for: {}", request.query);
                    return Ok(results.clone());
                }
            }
        }
        
        // Rate limit
        self.rate_limit().await;
        
        // Route to provider
        let results = match self.config.get_active_provider() {
            Some(Provider::SerpApi) => self.search_serpapi(&request).await,
            Some(Provider::ScrapingBee) => self.search_scrapingbee(&request).await,
            Some(Provider::Custom) => self.search_custom(&request).await,
            None => Err(anyhow!("No web scraping provider configured")),
        }?;
        
        // Cache results
        {
            let mut cache = self.cache.lock().await;
            cache.insert(cache_key, (results.clone(), Instant::now()));
        }
        
        Ok(results)
    }
    
    /// Search using SerpAPI
    async fn search_serpapi(&self, request: &SearchRequest) -> Result<Vec<SearchResult>> {
        let api_key = self.config.serpapi_key.as_ref()
            .ok_or_else(|| anyhow!("SerpAPI key not configured"))?;
        
        // Pre-allocate string storage to avoid temporary values
        let num_str = request.result_count.to_string();
        let tbs_str = request.time_range.as_ref().map(|t| format!("qdr:{}", t));
        
        let mut params: Vec<(&str, &str)> = vec![
            ("q", request.query.as_str()),
            ("api_key", api_key.as_str()),
            ("engine", "google"),
            ("num", &num_str),
        ];
        
        // Add location if specified
        if let Some(location) = &request.location {
            params.push(("location", location.as_str()));
        }
        
        // Add time range if specified
        if let Some(ref tbs) = tbs_str {
            params.push(("tbs", tbs.as_str()));
        }
        
        // Adjust for search type
        let endpoint = match request.search_type {
            SearchType::News => "https://serpapi.com/search?engine=google_news",
            SearchType::RealEstate => "https://serpapi.com/search?engine=google",
            _ => "https://serpapi.com/search?engine=google",
        };
        
        info!("Searching SerpAPI: {}", request.query);
        
        let response = self.http_client
            .get(endpoint)
            .query(&params)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("SerpAPI error: {}", error_text);
            return Err(anyhow!("SerpAPI request failed: {}", error_text));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        // Parse results based on search type
        let results = match request.search_type {
            SearchType::News => self.parse_serpapi_news(&data),
            _ => self.parse_serpapi_general(&data),
        }?;
        
        info!("SerpAPI returned {} results", results.len());
        Ok(results)
    }
    
    /// Parse SerpAPI general search results
    fn parse_serpapi_general(&self, data: &serde_json::Value) -> Result<Vec<SearchResult>> {
        let mut results = vec![];
        
        if let Some(organic) = data["organic_results"].as_array() {
            for (i, item) in organic.iter().enumerate() {
                results.push(SearchResult {
                    title: item["title"].as_str().unwrap_or("").to_string(),
                    url: item["link"].as_str().unwrap_or("").to_string(),
                    snippet: item["snippet"].as_str().unwrap_or("").to_string(),
                    source: item["source"].as_str().unwrap_or("Google").to_string(),
                    published_date: item["date"].as_str().map(|s| s.to_string()),
                    relevance_score: 1.0 - (i as f32 * 0.05), // Decrease relevance by position
                });
            }
        }
        
        Ok(results)
    }
    
    /// Parse SerpAPI news results
    fn parse_serpapi_news(&self, data: &serde_json::Value) -> Result<Vec<SearchResult>> {
        let mut results = vec![];
        
        if let Some(news_results) = data["news_results"].as_array() {
            for (i, item) in news_results.iter().enumerate() {
                results.push(SearchResult {
                    title: item["title"].as_str().unwrap_or("").to_string(),
                    url: item["link"].as_str().unwrap_or("").to_string(),
                    snippet: item["snippet"].as_str().unwrap_or("").to_string(),
                    source: item["source"].as_str().unwrap_or("News").to_string(),
                    published_date: item["date"].as_str().map(|s| s.to_string()),
                    relevance_score: 1.0 - (i as f32 * 0.05),
                });
            }
        }
        
        Ok(results)
    }
    
    /// Search using ScrapingBee
    async fn search_scrapingbee(&self, request: &SearchRequest) -> Result<Vec<SearchResult>> {
        let api_key = self.config.scrapingbee_key.as_ref()
            .ok_or_else(|| anyhow!("ScrapingBee key not configured"))?;
        
        // Construct Google search URL
        let search_url = format!(
            "https://www.google.com/search?q={}&num={}",
            urlencoding::encode(&request.query),
            request.result_count
        );
        
        let url = format!(
            "https://app.scrapingbee.com/api/v1/?api_key={}&url={}&render_js=false",
            api_key,
            urlencoding::encode(&search_url)
        );
        
        info!("Searching ScrapingBee: {}", request.query);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("ScrapingBee error: {}", error_text);
            return Err(anyhow!("ScrapingBee request failed: {}", error_text));
        }
        
        let html = response.text().await?;
        
        // Parse HTML results (simplified)
        let results = self.parse_scrapingbee_html(&html)?;
        
        info!("ScrapingBee returned {} results", results.len());
        Ok(results)
    }
    
    /// Parse ScrapingBee HTML response
    fn parse_scrapingbee_html(&self, html: &str) -> Result<Vec<SearchResult>> {
        // This would use HTML parsing (e.g., with select.rs or similar)
        // For now, return placeholder
        warn!("HTML parsing not fully implemented, using simplified extraction");
        
        // Simple extraction based on common Google HTML structure
        let mut results = vec![];
        
        // Look for search results (very simplified)
        if html.contains("g-bib") || html.contains("result") {
            results.push(SearchResult {
                title: "Search results extracted from HTML".to_string(),
                url: "https://www.google.com".to_string(),
                snippet: "HTML parsing would extract actual results here".to_string(),
                source: "Google".to_string(),
                published_date: None,
                relevance_score: 0.8,
            });
        }
        
        Ok(results)
    }
    
    /// Custom search (fallback)
    async fn search_custom(&self, _request: &SearchRequest) -> Result<Vec<SearchResult>> {
        warn!("Custom search not implemented, returning empty results");
        Ok(vec![])
    }
    
    /// Rate limit requests
    async fn rate_limit(&self) {
        let mut limiter = self.rate_limiter.lock().await;
        
        if let Some(last) = limiter.last_request {
            let elapsed = last.elapsed();
            if elapsed < limiter.min_interval {
                let sleep_duration = limiter.min_interval - elapsed;
                debug!("Rate limiting: sleeping for {:?}", sleep_duration);
                tokio::time::sleep(sleep_duration).await;
            }
        }
        
        limiter.last_request = Some(Instant::now());
    }
    
    /// Get configuration info
    pub fn get_config_info(&self) -> serde_json::Value {
        serde_json::json!({
            "configured": self.is_configured(),
            "provider": match self.config.get_active_provider() {
                Some(Provider::SerpApi) => "SerpAPI",
                Some(Provider::ScrapingBee) => "ScrapingBee",
                Some(Provider::Custom) => "Custom",
                None => "None",
            },
            "rate_limit_per_minute": self.config.rate_limit_requests_per_minute,
            "cache_duration_seconds": self.config.cache_duration_seconds,
        })
    }
}

impl RateLimiter {
    async fn wait_if_needed(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < self.min_interval {
                let sleep_duration = self.min_interval - elapsed;
                tokio::time::sleep(sleep_duration).await;
            }
        }
        self.last_request = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_search_request_builder() {
        let request = SearchRequest::new("Durham real estate")
            .with_location("Canada")
            .with_count(20)
            .with_type(SearchType::News);
        
        assert_eq!(request.query, "Durham real estate");
        assert_eq!(request.location, Some("Canada".to_string()));
        assert_eq!(request.result_count, 20);
        assert_eq!(request.search_type, SearchType::News);
    }
    
    #[test]
    fn test_scraping_config() {
        let config = ScrapingConfig {
            serpapi_key: Some("test_key".to_string()),
            ..Default::default()
        };
        
        assert!(config.is_configured());
        assert_eq!(config.get_active_provider(), Some(Provider::SerpApi));
    }
}
