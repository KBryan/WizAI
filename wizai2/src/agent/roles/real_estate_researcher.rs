//! Real Estate Researcher Agent
//! Specializes in Toronto/Durham market analysis using Repliers MCP

use crate::agent::core::{AgentId, AgentRegistry, AgentRole};
use crate::agent::executor::{AgentExecutor, TaskRequest, TaskResult};
use crate::mcp::MCPClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Real Estate Researcher - Specializes in property market analysis
pub struct RealEstateResearcher {
    executor: AgentExecutor,
    mcp_client: Arc<MCPClient>,
}

/// Market sentiment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSentiment {
    pub location: String,
    pub overall_sentiment: String,
    pub key_indicators: Vec<SentimentIndicator>,
    pub news_sources: Vec<NewsSource>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentIndicator {
    pub name: String,
    pub value: String,
    pub trend: String,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSource {
    pub name: String,
    pub url: String,
    pub type_: String,
    pub relevance: String,
}

/// Research request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRequest {
    /// Location (city, neighborhood, or postal code)
    pub location: String,
    /// Property types to analyze
    pub property_types: Vec<PropertyType>,
    /// Price range
    pub price_range: Option<PriceRange>,
    /// Research focus areas
    pub focus_areas: Vec<FocusArea>,
    /// Output format
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyType {
    Detached,
    SemiDetached,
    Townhouse,
    Condo,
    Commercial,
    MultiFamily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub min: u64,
    pub max: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FocusArea {
    MarketTrends,
    PricingAnalysis,
    InventoryLevels,
    DaysOnMarket,
    ComparableSales,
    NeighborhoodAnalysis,
    InvestmentOpportunity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    MarketReport,
    InvestmentAnalysis,
    ComparableAnalysis,
    ExecutiveSummary,
}

/// Research report output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub title: String,
    pub location: String,
    pub generated_at: String,
    pub summary: String,
    pub findings: Vec<Finding>,
    pub recommendations: Vec<Recommendation>,
    pub data_sources: Vec<String>,
    pub total_listings_analyzed: u32,
    pub avg_days_on_market: f32,
    pub median_price: f64,
    pub price_per_sqft: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub description: String,
    pub data_points: Vec<DataPoint>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub label: String,
    pub value: String,
    pub trend: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl RealEstateResearcher {
    pub async fn new(
        agent_id: AgentId,
        registry: Arc<RwLock<AgentRegistry>>,
        mcp_client: Arc<MCPClient>,
    ) -> Result<Self> {
        let executor = AgentExecutor::new(agent_id, registry).await?;
        Ok(Self {
            executor,
            mcp_client,
        })
    }

    /// Conduct comprehensive market research
    pub async fn conduct_research(&self, request: ResearchRequest) -> Result<ResearchReport> {
        info!(
            "Starting real estate research for: {} - {:?}",
            request.location, request.focus_areas
        );

        // Step 1: Search listings
        let listings = self.search_listings(&request).await?;

        // Step 2: Get comparable sales
        let comparables = if request.focus_areas.contains(&FocusArea::ComparableSales) {
            self.get_comparable_sales(&request, &listings).await?
        } else {
            vec![]
        };

        // Step 3: Analyze market trends
        let trends = if request.focus_areas.contains(&FocusArea::MarketTrends) {
            self.analyze_market_trends(&request).await?
        } else {
            None
        };

        // Step 4: Generate report
        let report = self.generate_report(&request, listings, comparables, trends).await?;

        Ok(report)
    }

    /// Search listings via Repliers MCP
    async fn search_listings(
        &self,
        request: &ResearchRequest,
    ) -> Result<serde_json::Value> {
        // Build search parameters
        let mut params = serde_json::json!({
            "city": request.location.clone(),
            "status": "active",
        });

        if let Some(price_range) = &request.price_range {
            params["minListPrice"] = price_range.min.into();
            params["maxListPrice"] = price_range.max.into();
        }

        // Add property types
        let property_types: Vec<String> = request
            .property_types
            .iter()
            .map(|pt| match pt {
                PropertyType::Detached => "Detached".to_string(),
                PropertyType::SemiDetached => "Semi-Detached".to_string(),
                PropertyType::Townhouse => "Townhouse".to_string(),
                PropertyType::Condo => "Condo".to_string(),
                PropertyType::Commercial => "Commercial".to_string(),
                PropertyType::MultiFamily => "Multi-Family".to_string(),
            })
            .collect();

        if !property_types.is_empty() {
            params["propertyType"] = serde_json::json!(property_types);
        }

        // Call Repliers MCP tool
        let result = self
            .mcp_client
            .call_tool("repliers", "repliers_listings_search", params)
            .await?;

        // Extract text content
        let text = result
            .content
            .iter()
            .filter_map(|c| match c {
                crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        // Parse JSON response
        let listings: serde_json::Value = serde_json::from_str(&text)?;

        Ok(listings)
    }

    /// Get comparable sales
    async fn get_comparable_sales(
        &self,
        request: &ResearchRequest,
        listings: &serde_json::Value,
    ) -> Result<Vec<serde_json::Value>> {
        // Extract first listing MLS number
        let mls_number = listings["results"][0]["mlsNumber"]
            .as_str()
            .unwrap_or_default();

        if mls_number.is_empty() {
            return Ok(vec![]);
        }

        let params = serde_json::json!({
            "mlsNumber": mls_number,
            "radius": 1.0, // 1km radius
            "listPriceRange": 50000, // +/- $50k
        });

        let result = self
            .mcp_client
            .call_tool("repliers", "find_similar_listings", params)
            .await?;

        let text = result
            .content
            .iter()
            .filter_map(|c| match c {
                crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let comparables: serde_json::Value = serde_json::from_str(&text)?;

        Ok(vec![comparables])
    }

    /// Analyze market trends
    async fn analyze_market_trends(
        &self,
        request: &ResearchRequest,
    ) -> Result<Option<serde_json::Value>> {
        // Get sold listings for trend analysis
        let params = serde_json::json!({
            "city": request.location.clone(),
            "status": "sold",
            "minSoldDate": "2024-01-01",
        });

        let result = self
            .mcp_client
            .call_tool("repliers", "repliers_listings_search", params)
            .await?;

        let text = result
            .content
            .iter()
            .filter_map(|c| match c {
                crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        let trends: serde_json::Value = serde_json::from_str(&text)?;

        Ok(Some(trends))
    }

    /// Generate comprehensive report
    async fn generate_report(
        &self,
        request: &ResearchRequest,
        listings: serde_json::Value,
        _comparables: Vec<serde_json::Value>,
        trends: Option<serde_json::Value>,
    ) -> Result<ResearchReport> {
        // Calculate statistics
        let empty_array: Vec<serde_json::Value> = vec![];
        let results = listings["results"].as_array().unwrap_or(&empty_array);
        let total_listings = results.len() as u32;

        // Calculate median price
        let prices: Vec<f64> = results
            .iter()
            .filter_map(|r| r["listPrice"].as_f64())
            .collect();

        let median_price = if !prices.is_empty() {
            let mut sorted = prices.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted[sorted.len() / 2]
        } else {
            0.0
        };

        // Calculate average days on market
        let avg_dom: f32 = results
            .iter()
            .filter_map(|r| r["daysOnMarket"].as_u64())
            .map(|d| d as f32)
            .sum::<f32>()
            / total_listings.max(1) as f32;

        // Generate findings
        let findings = vec![
            Finding {
                category: "Market Overview".to_string(),
                description: format!(
                    "Found {} active listings in {}",
                    total_listings, request.location
                ),
                data_points: vec![
                    DataPoint {
                        label: "Total Listings".to_string(),
                        value: total_listings.to_string(),
                        trend: None,
                    },
                    DataPoint {
                        label: "Median Price".to_string(),
                        value: format!("${:.0}", median_price),
                        trend: trends.as_ref().map(|_| "stable".to_string()),
                    },
                    DataPoint {
                        label: "Avg Days on Market".to_string(),
                        value: format!("{:.1}", avg_dom),
                        trend: None,
                    },
                ],
                confidence: 0.85,
            },
        ];

        // Generate recommendations
        let recommendations = vec![
            Recommendation {
                priority: Priority::High,
                title: "Monitor Inventory Levels".to_string(),
                description: format!(
                    "Current inventory of {} listings indicates a {} market",
                    total_listings,
                    if total_listings > 100 { "balanced" } else { "limited inventory" }
                ),
                expected_outcome: "Better pricing strategy".to_string(),
            },
        ];

        Ok(ResearchReport {
            title: format!("Real Estate Market Analysis: {}", request.location),
            location: request.location.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            summary: format!(
                "Analysis of {} properties in {}. Median price: ${:.0}",
                total_listings, request.location, median_price
            ),
            findings,
            recommendations,
            data_sources: vec!["Repliers API".to_string()],
            total_listings_analyzed: total_listings,
            avg_days_on_market: avg_dom,
            median_price,
            price_per_sqft: None,
        })
    }

    /// Get current market sentiment and news
    pub async fn get_market_sentiment(&self, location: &str) -> Result<MarketSentiment> {
        info!("Getting market sentiment for: {}", location);

        // Get sentiment indicators
        let sentiment_result = self
            .mcp_client
            .call_tool("browser", "browser.get_market_sentiment", serde_json::json!({
                "location": location
            }))
            .await;

        // Get news sources
        let sources_result = self
            .mcp_client
            .call_tool("browser", "browser.fetch_news_sources", serde_json::json!({
                "location": location
            }))
            .await;

        // Search for recent news
        let news_result = self
            .mcp_client
            .call_tool("browser", "browser.search_news", serde_json::json!({
                "query": format!("{} real estate market 2026", location),
                "limit": 10
            }))
            .await;

        // Build sentiment report
        let mut indicators = vec![];
        let mut sources = vec![];
        let mut recommendations = vec![];

        // Parse sentiment data
        if let Ok(result) = sentiment_result {
            let text = result
                .content
                .iter()
                .filter_map(|c| match c {
                    crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(indicator_list) = data["sentiment_indicators"].as_array() {
                    for indicator in indicator_list {
                        indicators.push(SentimentIndicator {
                            name: indicator["indicator"].as_str().unwrap_or("Unknown").to_string(),
                            value: indicator["description"].as_str().unwrap_or("N/A").to_string(),
                            trend: "Check local real estate board".to_string(),
                            impact: indicator["where_to_find"].as_str().unwrap_or("N/A").to_string(),
                        });
                    }
                }
            }
        }

        // Parse sources
        if let Ok(result) = sources_result {
            let text = result
                .content
                .iter()
                .filter_map(|c| match c {
                    crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");
            
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                // Local sources
                if let Some(local) = data["local_sources"].as_array() {
                    for source in local {
                        sources.push(NewsSource {
                            name: source["name"].as_str().unwrap_or("Unknown").to_string(),
                            url: source["url"].as_str().unwrap_or("#").to_string(),
                            type_: source["type"].as_str().unwrap_or("News").to_string(),
                            relevance: "High - Local market data".to_string(),
                        });
                    }
                }
                // Regional sources
                if let Some(regional) = data["regional_sources"].as_array() {
                    for source in regional {
                        sources.push(NewsSource {
                            name: source["name"].as_str().unwrap_or("Unknown").to_string(),
                            url: source["url"].as_str().unwrap_or("#").to_string(),
                            type_: source["type"].as_str().unwrap_or("News").to_string(),
                            relevance: "Medium - Regional trends".to_string(),
                        });
                    }
                }
                // Social sources
                if let Some(social) = data["social_sources"].as_array() {
                    for source in social {
                        sources.push(NewsSource {
                            name: source["name"].as_str().unwrap_or("Unknown").to_string(),
                            url: source["url"].as_str().unwrap_or("#").to_string(),
                            type_: source["type"].as_str().unwrap_or("Discussion").to_string(),
                            relevance: "Medium - Community sentiment".to_string(),
                        });
                    }
                }
            }
        }

        // Build recommendations
        let source_name = sources
            .iter()
            .find(|s| s.name.contains("Association"))
            .map(|s| s.name.as_str())
            .unwrap_or("local real estate board");
        recommendations.push(format!(
            "Check {} for current inventory levels and months of supply",
            source_name
        ));
        recommendations.push("Compare current month stats to previous month and year".to_string());
        recommendations.push("Monitor Bank of Canada interest rate announcements".to_string());
        recommendations.push("Watch for policy changes (foreign buyer rules, capital gains)".to_string());

        // Determine overall sentiment based on data availability
        let overall_sentiment = if sources.len() > 5 {
            "Data-rich environment - multiple sources available for analysis"
        } else {
            "Limited data - recommend direct research from primary sources"
        };

        Ok(MarketSentiment {
            location: location.to_string(),
            overall_sentiment: overall_sentiment.to_string(),
            key_indicators: indicators,
            news_sources: sources,
            recommendations,
        })
    }

    /// Search Reddit for community discussions
    pub async fn search_reddit_sentiment(&self, query: &str, subreddit: Option<&str>) -> Result<String> {
        info!("Searching Reddit for: {} in {:?}", query, subreddit);

        let result = self
            .mcp_client
            .call_tool("browser", "browser.search_reddit", serde_json::json!({
                "query": query,
                "subreddit": subreddit
            }))
            .await?;

        let text = result
            .content
            .iter()
            .filter_map(|c| match c {
                crate::mcp::protocol::MCPContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(text)
    }

    /// Comprehensive market research including sentiment
    pub async fn conduct_full_research(&self, location: &str, price_range: Option<(u64, u64)>) -> Result<String> {
        let mut report = format!("# Comprehensive Real Estate Research: {}\n\n", location);

        // 1. Get market sentiment
        report.push_str("## 1. Market Sentiment Analysis\n\n");
        match self.get_market_sentiment(location).await {
            Ok(sentiment) => {
                report.push_str(&format!("**Overall Assessment:** {}\n\n", sentiment.overall_sentiment));
                
                report.push_str("### Key Indicators to Monitor:\n");
                for indicator in &sentiment.key_indicators {
                    report.push_str(&format!("- **{}**: {}\n", indicator.name, indicator.value));
                    report.push_str(&format!("  - Where to find: {}\n", indicator.impact));
                }
                report.push_str("\n");

                report.push_str("### Recommended News Sources:\n");
                for source in &sentiment.news_sources {
                    report.push_str(&format!("- [{}]({}) - {} (Relevance: {})\n", 
                        source.name, source.url, source.type_, source.relevance));
                }
                report.push_str("\n");

                report.push_str("### Action Items:\n");
                for (i, rec) in sentiment.recommendations.iter().enumerate() {
                    report.push_str(&format!("{}. {}\n", i + 1, rec));
                }
                report.push_str("\n");
            }
            Err(e) => {
                report.push_str(&format!("*Error getting sentiment: {}*\n\n", e));
            }
        }

        // 2. Get Reddit sentiment
        report.push_str("## 2. Community Sentiment (Reddit)\n\n");
        match self.search_reddit_sentiment(&format!("{} real estate", location), Some("realestatecanada")).await {
            Ok(reddit_data) => {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&reddit_data) {
                    if let Some(url) = data["search_url"].as_str() {
                        report.push_str(&format!("**Reddit Search:** [View Discussions]({})\n\n", url));
                    }
                    if let Some(subs) = data["relevant_subreddits"].as_array() {
                        report.push_str("**Key Communities:**\n");
                        for sub in subs {
                            report.push_str(&format!("- r/{}\n", sub.as_str().unwrap_or("")));
                        }
                        report.push_str("\n");
                    }
                }
            }
            Err(e) => {
                report.push_str(&format!("*Error searching Reddit: {}*\n\n", e));
            }
        }

        // 3. Property search if price range provided
        if let Some((min, max)) = price_range {
            report.push_str("## 3. Current Market Listings\n\n");
            let research_req = ResearchRequest {
                location: location.to_string(),
                property_types: vec![PropertyType::Detached, PropertyType::Townhouse],
                price_range: Some(PriceRange { min, max }),
                focus_areas: vec![FocusArea::MarketTrends, FocusArea::PricingAnalysis, FocusArea::InventoryLevels],
                output_format: OutputFormat::MarketReport,
            };

            match self.conduct_research(research_req).await {
                Ok(market_report) => {
                    report.push_str(&format!("**Active Listings:** {}\n", market_report.total_listings_analyzed));
                    report.push_str(&format!("**Median Price:** ${:.0}\n", market_report.median_price));
                    report.push_str(&format!("**Avg Days on Market:** {:.1}\n\n", market_report.avg_days_on_market));
                    
                    report.push_str("### Key Findings:\n");
                    for finding in &market_report.findings {
                        report.push_str(&format!("- **{}**: {}\n", finding.category, finding.description));
                        for point in &finding.data_points {
                            report.push_str(&format!("  - {}: {}\n", point.label, point.value));
                        }
                    }
                }
                Err(e) => {
                    report.push_str(&format!("*Error getting listings: {}*\n\n", e));
                }
            }
        }

        // 4. Summary and recommendations
        report.push_str("\n## Summary & Recommendations\n\n");
        report.push_str("### What to Do Next:\n");
        report.push_str("1. **Monitor these indicators monthly:**\n");
        report.push_str("   - Months of supply (6+ = buyer's market)\n");
        report.push_str("   - Sale-to-list price ratio\n");
        report.push_str("   - Days on market trends\n\n");
        report.push_str("2. **Stay informed via:**\n");
        report.push_str("   - Local real estate board monthly stats\n");
        report.push_str("   - Bank of Canada rate announcements\n");
        report.push_str("   - Government policy changes\n\n");
        report.push_str("3. **For this specific search:**\n");
        if price_range.is_some() {
            report.push_str("   - Review current listings above\n");
        }
        report.push_str("   - Contact local agents for ground-level insights\n");
        report.push_str("   - Visit target neighborhoods at different times/days\n\n");

        report.push_str("---\n\n");
        report.push_str("*Note: This report combines real-time listings data (where available) with guidance on how to research current market sentiment. For the most up-to-date sentiment analysis, check the sources listed above directly.*");

        Ok(report)
    }
}
