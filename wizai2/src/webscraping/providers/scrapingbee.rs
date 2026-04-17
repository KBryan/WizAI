//! ScrapingBee Provider
//! Implementation for ScrapingBee web scraping service
//!
//! Website: https://www.scrapingbee.com
//! Pricing: Free trial (1000 API credits), paid plans from $49/month

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// ScrapingBee Client
pub struct ScrapingBeeClient {
    api_key: String,
    http_client: Client,
    base_url: String,
}

impl ScrapingBeeClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key: api_key.into(),
            http_client,
            base_url: "https://app.scrapingbee.com/api/v1".to_string(),
        }
    }

    /// Scrape a URL
    pub async fn scrape_url(&self, url: &str, params: ScrapeParams) -> Result<String> {
        let encoded_url = urlencoding::encode(url);
        
        let mut request_params = vec![
            ("api_key", self.api_key.as_str()),
            ("url", encoded_url.as_ref()),
        ];

        if params.render_js {
            request_params.push(("render_js", "true"));
        }

        if params.premium_proxy {
            request_params.push(("premium_proxy", "true"));
        }

        if let Some(country) = &params.country_code {
            request_params.push(("country_code", country.as_str()));
        }

        let response = self.http_client
            .get(&self.base_url)
            .query(&request_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("ScrapingBee error: {}", error_text));
        }

        let html = response.text().await?;
        Ok(html)
    }

    /// Google search
    pub async fn search_google(&self, query: &str, params: ScrapeParams) -> Result<String> {
        let search_url = format!(
            "https://www.google.com/search?q={}&num={}",
            urlencoding::encode(query),
            params.num_results.unwrap_or(10)
        );

        self.scrape_url(&search_url, params).await
    }

    /// Extract structured data using AI
    pub async fn extract_data(&self, url: &str, schema: &str) -> Result<serde_json::Value> {
        let encoded_url = urlencoding::encode(url);
        
        let params = vec![
            ("api_key", self.api_key.as_str()),
            ("url", encoded_url.as_ref()),
            ("extract_rules", schema),
            ("ai_extract", "true"),
        ];

        let response = self.http_client
            .get(&self.base_url)
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("ScrapingBee extraction error: {}", error_text));
        }

        let data: serde_json::Value = response.json().await?;
        Ok(data)
    }

    /// Get remaining credits
    pub async fn get_credits(&self) -> Result<i32> {
        // ScrapingBee doesn't have a direct API for credits
        // You can check your dashboard
        Ok(-1) // Return -1 to indicate unknown
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScrapeParams {
    pub render_js: bool,
    pub premium_proxy: bool,
    pub num_results: Option<u32>,
    pub country_code: Option<String>,
}

impl ScrapeParams {
    pub fn with_js_rendering(mut self) -> Self {
        self.render_js = true;
        self
    }

    pub fn with_premium_proxy(mut self) -> Self {
        self.premium_proxy = true;
        self
    }

    pub fn with_country(mut self, code: impl Into<String>) -> Self {
        self.country_code = Some(code.into());
        self
    }

    pub fn with_results(mut self, num: u32) -> Self {
        self.num_results = Some(num);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_params_builder() {
        let params = ScrapeParams::default()
            .with_js_rendering()
            .with_premium_proxy()
            .with_country("CA")
            .with_results(20);

        assert!(params.render_js);
        assert!(params.premium_proxy);
        assert_eq!(params.country_code, Some("CA".to_string()));
        assert_eq!(params.num_results, Some(20));
    }
}
