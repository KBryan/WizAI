//! SerpAPI Provider
//! Implementation for SerpAPI web scraping service
//! 
//! Website: https://serpapi.com
//! Pricing: Free tier (100 searches/month), paid plans from $50/month

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// SerpAPI Client
pub struct SerpApiClient {
    api_key: String,
    http_client: Client,
    base_url: String,
}

impl SerpApiClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            api_key: api_key.into(),
            http_client,
            base_url: "https://serpapi.com/search".to_string(),
        }
    }

    /// Search Google
    pub async fn search_google(&self, query: &str, params: SearchParams) -> Result<SerpApiResponse> {
        let mut request_params = vec![
            ("q", query.to_string()),
            ("api_key", self.api_key.clone()),
            ("engine", "google".to_string()),
        ];

        if let Some(num) = params.num_results {
            request_params.push(("num", num.to_string()));
        }

        if let Some(location) = params.location {
            request_params.push(("location", location));
        }

        if let Some(time_range) = params.time_range {
            request_params.push(("tbs", format!("qdr:{}", time_range)));
        }

        let response = self.http_client
            .get(&self.base_url)
            .query(&request_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("SerpAPI error: {}", error_text));
        }

        let data: SerpApiResponse = response.json().await?;
        Ok(data)
    }

    /// Search Google News
    pub async fn search_news(&self, query: &str, params: SearchParams) -> Result<Vec<NewsResult>> {
        let mut request_params = vec![
            ("q", query.to_string()),
            ("api_key", self.api_key.clone()),
            ("engine", "google_news".to_string()),
        ];

        if let Some(num) = params.num_results {
            request_params.push(("num", num.to_string()));
        }

        let response = self.http_client
            .get(&self.base_url)
            .query(&request_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("SerpAPI error: {}", error_text));
        }

        let data: serde_json::Value = response.json().await?;
        
        // Parse news results
        let mut news_results = vec![];
        if let Some(results) = data["news_results"].as_array() {
            for result in results {
                news_results.push(NewsResult {
                    title: result["title"].as_str().unwrap_or("").to_string(),
                    link: result["link"].as_str().unwrap_or("").to_string(),
                    source: result["source"].as_str().unwrap_or("").to_string(),
                    date: result["date"].as_str().map(|s| s.to_string()),
                    snippet: result["snippet"].as_str().unwrap_or("").to_string(),
                });
            }
        }

        Ok(news_results)
    }

    /// Get account info
    pub async fn get_account_info(&self) -> Result<AccountInfo> {
        let url = "https://serpapi.com/account";
        
        let response = self.http_client
            .get(url)
            .query(&[("api_key", &self.api_key)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get account info"));
        }

        let info: AccountInfo = response.json().await?;
        Ok(info)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchParams {
    pub num_results: Option<u32>,
    pub location: Option<String>,
    pub time_range: Option<String>, // "1d", "1w", "1m", "1y"
}

#[derive(Debug, Deserialize)]
pub struct SerpApiResponse {
    pub search_metadata: SearchMetadata,
    pub search_parameters: SearchParameters,
    pub search_information: SearchInformation,
    pub organic_results: Vec<OrganicResult>,
    #[serde(default)]
    pub news_results: Vec<NewsResult>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMetadata {
    pub id: String,
    pub status: String,
    pub json_endpoint: String,
    pub created_at: String,
    pub processed_at: String,
    pub google_url: String,
    pub raw_html_file: String,
    pub total_time_taken: f64,
}

#[derive(Debug, Deserialize)]
pub struct SearchParameters {
    pub engine: String,
    pub q: String,
    pub location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchInformation {
    pub organic_results_state: String,
    pub query_displayed: String,
    #[serde(default)]
    pub total_results: Option<String>,
    #[serde(default)]
    pub time_taken_displayed: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct OrganicResult {
    pub position: i32,
    pub title: String,
    pub link: String,
    #[serde(default)]
    pub redirect_link: Option<String>,
    #[serde(default)]
    pub displayed_link: Option<String>,
    pub snippet: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NewsResult {
    pub title: String,
    pub link: String,
    pub source: String,
    pub date: Option<String>,
    pub snippet: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    pub plan: String,
    pub plan_searches_left: i32,
    pub searches_consumed: i32,
    pub searches_limit: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_params_builder() {
        let params = SearchParams {
            num_results: Some(20),
            location: Some("Toronto, Canada".to_string()),
            time_range: Some("1m".to_string()),
        };

        assert_eq!(params.num_results, Some(20));
        assert_eq!(params.location, Some("Toronto, Canada".to_string()));
    }
}
