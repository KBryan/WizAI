//! Real Estate Search Module
//! Specialized search queries for real estate market research

use super::*;
use anyhow::Result;

/// Real Estate Search Builder
pub struct RealEstateSearch {
    client: WebScrapingClient,
}

impl RealEstateSearch {
    pub fn new(client: WebScrapingClient) -> Self {
        Self { client }
    }
    
    /// Search for news about a specific market
    pub async fn search_market_news(&self, location: &str, time_range: &str) -> Result<Vec<NewsArticle>> {
        let queries = vec![
            format!("{} real estate market news", location),
            format!("{} housing market trends", location),
            format!("{} property prices 2026", location),
        ];
        
        let mut all_results = vec![];
        
        for query in queries {
            let request = SearchRequest::new(query)
                .with_type(SearchType::News)
                .with_time_range(time_range)
                .with_count(10);
            
            match self.client.search(request).await {
                Ok(results) => {
                    for result in results {
                        all_results.push(NewsArticle {
                            title: result.title,
                            url: result.url,
                            source: result.source,
                            published_date: result.published_date.unwrap_or_else(|| "Unknown".to_string()),
                            snippet: result.snippet,
                            image_url: None,
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!("Search failed for query: {}", e);
                }
            }
        }
        
        // Remove duplicates based on URL
        all_results.sort_by(|a, b| a.url.cmp(&b.url));
        all_results.dedup_by(|a, b| a.url == b.url);
        
        // Sort by date (newest first)
        all_results.sort_by(|a, b| b.published_date.cmp(&a.published_date));
        
        Ok(all_results)
    }
    
    /// Search for market sentiment indicators
    pub async fn search_market_sentiment(&self, location: &str) -> Result<MarketSentimentData> {
        let queries = vec![
            format!("{} real estate inventory levels", location),
            format!("{} housing market supply", location),
            format!("{} days on market real estate", location),
            format!("{} real estate buyer demand", location),
        ];
        
        let mut market_data = vec![];
        let mut articles = vec![];
        
        for query in queries {
            let request = SearchRequest::new(query)
                .with_type(SearchType::News)
                .with_time_range("1m") // Last month
                .with_count(5);
            
            match self.client.search(request).await {
                Ok(results) => {
                    for result in results {
                        articles.push(NewsArticle {
                            title: result.title.clone(),
                            url: result.url.clone(),
                            source: result.source.clone(),
                            published_date: result.published_date.clone().unwrap_or_else(|| "Unknown".to_string()),
                            snippet: result.snippet.clone(),
                            image_url: None,
                        });
                        
                        // Try to extract numeric data from snippets
                        if let Some(metric) = self.extract_metric_from_text(&result.snippet, location) {
                            market_data.push(metric);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Sentiment search failed: {}", e);
                }
            }
        }
        
        // Analyze sentiment based on keywords in articles
        let sentiment_score = self.calculate_sentiment_score(&articles);
        let trend = self.determine_market_trend(&market_data);
        
        Ok(MarketSentimentData {
            location: location.to_string(),
            overall_sentiment: sentiment_score,
            trend,
            key_metrics: market_data,
            recent_articles: articles.into_iter().take(10).collect(),
            data_sources: vec![
                "Google News".to_string(),
                "Real Estate Reports".to_string(),
                "Local Market Data".to_string(),
            ],
        })
    }
    
    /// Search for specific property information
    pub async fn search_property_info(&self, location: &str, property_type: &str, price_range: &str) -> Result<Vec<SearchResult>> {
        let query = format!(
            "{} {} for sale {} price trends",
            location, property_type, price_range
        );
        
        let request = SearchRequest::new(query)
            .with_type(SearchType::RealEstate)
            .with_count(20);
        
        self.client.search(request).await
    }
    
    /// Search for Reddit discussions
    pub async fn search_reddit_sentiment(&self, location: &str) -> Result<Vec<SearchResult>> {
        let query = format!("site:reddit.com {} real estate", location);
        
        let request = SearchRequest::new(query)
            .with_time_range("1m")
            .with_count(15);
        
        self.client.search(request).await
    }
    
    /// Search for government/regulatory news
    pub async fn search_policy_changes(&self, location: &str) -> Result<Vec<NewsArticle>> {
        let queries = vec![
            format!("{} foreign buyer ban real estate", location),
            format!("{} capital gains tax real estate", location),
            format!("{} mortgage rules changes 2026", location),
            format!("{} real estate policy government", location),
        ];
        
        let mut articles = vec![];
        
        for query in queries {
            let request = SearchRequest::new(query)
                .with_type(SearchType::News)
                .with_time_range("3m")
                .with_count(5);
            
            match self.client.search(request).await {
                Ok(results) => {
                    for result in results {
                        articles.push(NewsArticle {
                            title: result.title,
                            url: result.url,
                            source: result.source,
                            published_date: result.published_date.unwrap_or_else(|| "Unknown".to_string()),
                            snippet: result.snippet,
                            image_url: None,
                        });
                    }
                }
                Err(e) => tracing::warn!("Policy search failed: {}", e),
            }
        }
        
        // Remove duplicates
        articles.sort_by(|a, b| a.url.cmp(&b.url));
        articles.dedup_by(|a, b| a.url == b.url);
        
        Ok(articles)
    }
    
    /// Extract numeric metrics from text using simple pattern matching
    fn extract_metric_from_text(&self, text: &str, location: &str) -> Option<MarketDataPoint> {
        // Look for patterns like "X days on market", "Y% increase", "Z months supply"
        
        if text.contains("days on market") || text.contains("DOM") {
            // Try to find a number followed by days
            if let Some(days) = self.extract_number_before_keyword(text, &["days", "DOM"]) {
                return Some(MarketDataPoint {
                    metric: "Days on Market".to_string(),
                    value: format!("{} days", days),
                    source: "News Analysis".to_string(),
                    date: chrono::Utc::now().to_rfc3339(),
                    confidence: 0.6,
                });
            }
        }
        
        if text.contains("months of supply") || text.contains("inventory") {
            if let Some(months) = self.extract_number_before_keyword(text, &["months", "supply"]) {
                return Some(MarketDataPoint {
                    metric: "Months of Supply".to_string(),
                    value: format!("{} months", months),
                    source: "News Analysis".to_string(),
                    date: chrono::Utc::now().to_rfc3339(),
                    confidence: 0.6,
                });
            }
        }
        
        if text.contains("price") && (text.contains("%") || text.contains("percent")) {
            if let Some(percent) = self.extract_percentage(text) {
                return Some(MarketDataPoint {
                    metric: "Price Change".to_string(),
                    value: format!("{}%", percent),
                    source: "News Analysis".to_string(),
                    date: chrono::Utc::now().to_rfc3339(),
                    confidence: 0.5,
                });
            }
        }
        
        None
    }
    
    /// Extract number before keywords
    fn extract_number_before_keyword(&self, text: &str, keywords: &[&str]) -> Option<i32> {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            for keyword in keywords {
                if word.to_lowercase().contains(keyword) && i > 0 {
                    // Check previous word for number
                    if let Ok(num) = words[i-1].trim_matches(&['.', ','][..]).parse::<i32>() {
                        return Some(num);
                    }
                }
            }
        }
        
        None
    }
    
    /// Extract percentage from text
    fn extract_percentage(&self, text: &str) -> Option<f32> {
        // Look for patterns like "X%" or "X percent"
        let re = regex::Regex::new(r"(\d+\.?\d*)\s*%").ok()?;
        
        if let Some(cap) = re.captures(text) {
            if let Ok(percent) = cap[1].parse::<f32>() {
                return Some(percent);
            }
        }
        
        None
    }
    
    /// Calculate sentiment score based on keywords
    fn calculate_sentiment_score(&self, articles: &[NewsArticle]) -> String {
        let positive_words = ["boom", "surge", "growth", "rising", "hot", "seller's market", "bidding war"];
        let negative_words = ["crash", "decline", "falling", "cooling", "correction", "buyer's market", "slowdown", "recession"];
        
        let mut positive_count = 0;
        let mut negative_count = 0;
        
        for article in articles {
            let text = format!("{} {}", article.title, article.snippet).to_lowercase();
            
            for word in &positive_words {
                if text.contains(word) { positive_count += 1; }
            }
            
            for word in &negative_words {
                if text.contains(word) { negative_count += 1; }
            }
        }
        
        if articles.is_empty() {
            return "No data available".to_string();
        }
        
        let ratio = positive_count as f32 / articles.len() as f32;
        
        if ratio > 0.6 {
            "Positive (Bullish)".to_string()
        } else if ratio > 0.4 {
            "Neutral/Mixed".to_string()
        } else {
            "Negative (Bearish)".to_string()
        }
    }
    
    /// Determine market trend based on metrics
    fn determine_market_trend(&self, metrics: &[MarketDataPoint]) -> String {
        // This is a simplified analysis
        let dom_metric = metrics.iter().find(|m| m.metric == "Days on Market");
        let supply_metric = metrics.iter().find(|m| m.metric == "Months of Supply");
        
        let mut factors = vec![];
        
        if let Some(dom) = dom_metric {
            // Parse days value
            if dom.value.contains("15") || dom.value.contains("20") {
                factors.push("Fast-moving market (low DOM)");
            } else if dom.value.contains("40") || dom.value.contains("50") || dom.value.contains("60") {
                factors.push("Slower market (higher DOM)");
            }
        }
        
        if let Some(supply) = supply_metric {
            if supply.value.contains("2") || supply.value.contains("3") {
                factors.push("Low inventory (seller's market)");
            } else if supply.value.contains("6") || supply.value.contains("8") {
                factors.push("High inventory (buyer's market)");
            }
        }
        
        if factors.is_empty() {
            "Trend unclear - insufficient data".to_string()
        } else {
            factors.join("; ")
        }
    }
}

/// Market sentiment data structure
#[derive(Debug, Clone)]
pub struct MarketSentimentData {
    pub location: String,
    pub overall_sentiment: String,
    pub trend: String,
    pub key_metrics: Vec<MarketDataPoint>,
    pub recent_articles: Vec<NewsArticle>,
    pub data_sources: Vec<String>,
}

impl MarketSentimentData {
    /// Generate a report from sentiment data
    pub fn generate_report(&self) -> String {
        let mut report = format!(
            "# Market Sentiment Report: {}\n\n",
            self.location
        );
        
        report.push_str(&format!("**Overall Sentiment:** {}\n", self.overall_sentiment));
        report.push_str(&format!("**Market Trend:** {}\n\n", self.trend));
        
        if !self.key_metrics.is_empty() {
            report.push_str("## Key Metrics\n\n");
            for metric in &self.key_metrics {
                report.push_str(&format!("- **{}**: {} (confidence: {:.0}%)\n", 
                    metric.metric, 
                    metric.value,
                    metric.confidence * 100.0
                ));
            }
            report.push_str("\n");
        }
        
        if !self.recent_articles.is_empty() {
            report.push_str("## Recent Articles\n\n");
            for (i, article) in self.recent_articles.iter().take(5).enumerate() {
                report.push_str(&format!("{}. [{}]({})\n", i + 1, article.title, article.url));
                report.push_str(&format!("   - Source: {} | Date: {}\n", article.source, article.published_date));
                report.push_str(&format!("   - {}\n\n", article.snippet));
            }
        }
        
        report.push_str("---\n\n");
        report.push_str("*Data sources: ");
        report.push_str(&self.data_sources.join(", "));
        report.push_str("*\n");
        
        report
    }
}
