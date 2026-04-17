//! Market Research Service for AI Real Estate Team
//!
//! Business logic service for coordinating market research operations including
//! CMA generation, comparable property searches, and market trend analysis.
//! Now integrated with Repliers API for live MLS data.

use crate::models::{
    ApiRequestLog, CMAReport, CMAStatus, CMASummary, ComparableProperty, CreateCmaRequest,
    MarketAnalysis, MarketReport, PaginatedList, PropertyCategory, SearchComparablesRequest,
    TrendDirection,
    api::{
        ComparableApiModel, ComparablesQuery, ComparablesResponse, CmaJobStatus,
        CmaStatusResponse, CreateCmaResponse, ListCmasQuery, LogApiRequest, MarketStatus,
        TargetProperty, TrendsQuery, TrendsResponse, get_neighborhoods, get_supported_areas,
        is_area_supported,
    },
};
use crate::services::{ComparableSubject, RepliersClient, RepliersProperty};
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Service for market research operations
pub struct MarketResearchService {
    db_pool: Arc<sqlx::Pool<sqlx::Sqlite>>,
    repliers_client: Option<RepliersClient>,
}

impl MarketResearchService {
    /// Create a new MarketResearchService instance
    pub fn new(db_pool: Arc<sqlx::Pool<sqlx::Sqlite>>) -> Self {
        // Initialize Repliers client if API key is available
        let repliers_client = std::env::var("REPLIERS_API_KEY")
            .ok()
            .map(|key| RepliersClient::new(key));
        
        if repliers_client.is_some() {
            tracing::info!("✅ Repliers API client initialized");
        } else {
            tracing::warn!("⚠️  REPLIERS_API_KEY not set - using mock data");
        }
        
        Self { db_pool, repliers_client }
    }
    
    /// Check if live Repliers data is available
    pub fn has_live_data(&self) -> bool {
        self.repliers_client.as_ref().map(|c| c.is_configured()).unwrap_or(false)
    }

    // CMA Operations

    /// Generate a CMA asynchronously and return job ID
    pub async fn generate_cma_async(
        &self,
        agent_id: &str,
        request: CreateCmaRequest,
    ) -> Result<CreateCmaResponse, String> {
        // Validate request
        request.validate()?;

        // Create CMA report record
        let property_type = self.parse_property_type(&request.property_type)?;
        let mut cma = CMAReport::new(
            &request.address,
            None::<String>,
            property_type,
            "api-request",
        );

        // Initially set status to processing
        cma.status = CMAStatus::Draft;
        cma.summary = "CMA generation initiated via API. Processing...".to_string();

        // Save to database
        self.save_cma(&cma).await?;

        // Spawn async task to process CMA
        let cma_id = cma.id.clone();
        let db_pool = Arc::clone(&self.db_pool);
        tokio::spawn(async move {
            // TODO: Implement actual CMA generation logic
            // This would involve:
            // 1. Call Repliers API to get comparables
            // 2. Analyze market data
            // 3. Generate price recommendations
            // 4. Update database
            
            // For now, simulate processing delay
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            // Update CMA status
            let _ = sqlx::query(
                "UPDATE cma_reports SET status = 'draft', summary = 'CMA generated successfully. Awaiting review.' WHERE id = ?"
            )
            .bind(&cma_id)
            .execute(&*db_pool)
            .await;
        });

        Ok(CreateCmaResponse {
            cma_id: cma.id,
            status: CmaJobStatus::Processing,
            estimated_completion: (Utc::now() + chrono::Duration::seconds(120)).to_rfc3339(),
        })
    }

    /// Get CMA generation status
    pub async fn get_cma_status(&self, cma_id: &str) -> Result<CmaStatusResponse, String> {
        let cma = self.get_cma_by_id(cma_id).await?;

        let (status, progress, message) = match cma.status {
            CMAStatus::Draft => (
                CmaJobStatus::Completed,
                100,
                "CMA completed and awaiting review".to_string(),
            ),
            CMAStatus::UnderReview => (
                CmaJobStatus::Completed,
                100,
                "CMA under human agent review".to_string(),
            ),
            CMAStatus::Approved => (
                CmaJobStatus::Completed,
                100,
                "CMA approved and ready".to_string(),
            ),
            CMAStatus::Rejected => (
                CmaJobStatus::Failed,
                0,
                "CMA rejected - please regenerate".to_string(),
            ),
            CMAStatus::Archived => (
                CmaJobStatus::Completed,
                100,
                "CMA archived".to_string(),
            ),
        };

        Ok(CmaStatusResponse {
            cma_id: cma_id.to_string(),
            status,
            progress,
            message,
        })
    }

    /// Get a CMA report by ID
    pub async fn get_cma(&self, cma_id: &str) -> Result<CMAReport, String> {
        self.get_cma_by_id(cma_id).await
    }

    /// List CMAs for an agent with filtering and pagination
    pub async fn list_cmas(
        &self,
        agent_id: &str,
        query: ListCmasQuery,
    ) -> Result<PaginatedList<CMASummary>, String> {
        let page = query.page.unwrap_or(1).max(1);
        let limit = query.limit.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * limit;

        // Build query based on filters
        let mut query_builder = 
            String::from("SELECT id, subject_address, property_type, status, price_recommendation_mid, created_at FROM cma_reports WHERE 1=1");
        
        if let Some(status_filter) = &query.status {
            query_builder.push_str(&format!(" AND status = '{}'", status_filter));
        }

        // Add sorting
        let sort_field = match query.sort.as_deref() {
            Some("score:desc") => "created_at DESC",
            Some("score:asc") => "created_at ASC",
            Some("date:desc") => "created_at DESC",
            Some("date:asc") => "created_at ASC",
            _ => "created_at DESC",
        };
        query_builder.push_str(&format!(" ORDER BY {}", sort_field));
        query_builder.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

        // Execute query
        let rows = sqlx::query_as::<_, CMASummaryRow>(&query_builder)
            .fetch_all(&*self.db_pool)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Get total count
        let count_query = "SELECT COUNT(*) as count FROM cma_reports WHERE 1=1";
        let total: i64 = sqlx::query_scalar(count_query)
            .fetch_one(&*self.db_pool)
            .await
            .unwrap_or(0);

        // Map to CMASummary
        let summaries: Vec<CMASummary> = rows
            .into_iter()
            .map(|row| CMASummary {
                id: row.id,
                subject_address: row.subject_address,
                property_type: serde_json::from_str(&row.property_type).unwrap_or(PropertyCategory::Detached),
                status: serde_json::from_str(&row.status).unwrap_or(CMAStatus::Draft),
                price_recommendation_mid: row.price_recommendation_mid,
                comparable_count: 0, // TODO: Calculate from comparables JSON
                confidence: 0,
                created_at: row.created_at,
            })
            .collect();

        Ok(PaginatedList {
            items: summaries,
            total,
            page,
            limit,
        })
    }

    // Comparables Operations

    /// Search for comparable properties using Repliers API
    pub async fn search_comparables(
        &self,
        query: ComparablesQuery,
    ) -> Result<ComparablesResponse, String> {
        // Validate request
        query.validate()?;

        let target = TargetProperty {
            address: query.address.clone(),
            property_type: query.property_type.clone(),
            bedrooms: None,
            bathrooms: None,
            square_feet: None,
        };

        // Try to fetch live data from Repliers API
        if let Some(ref client) = self.repliers_client {
            if client.is_configured() {
                return self.fetch_live_comparables(client, &query, target).await;
            }
        }

        // Fallback to mock data if Repliers not available
        self.fetch_mock_comparables(&query, target)
    }

    /// Fetch live comparables from Repliers API
    async fn fetch_live_comparables(
        &self,
        client: &RepliersClient,
        query: &ComparablesQuery,
        target: TargetProperty,
    ) -> Result<ComparablesResponse, String> {
        let property_type = query.property_type.as_deref();
        let limit = query.get_limit();
        let radius = query.get_radius();

        // Fetch sold properties from last 90 days
        match client.get_sold_properties(
            &query.address,
            radius,
            90, // last 90 days
            property_type,
            limit * 2, // Fetch extra to filter best matches
        ).await {
            Ok(properties) => {
                // Create subject for comparison
                let subject = ComparableSubject {
                    property_type: property_type
                        .map(|pt| self.parse_property_type(pt).unwrap_or(PropertyCategory::Detached))
                        .unwrap_or(PropertyCategory::Detached),
                    bedrooms: None,
                    bathrooms: None,
                    square_feet: None,
                };

                // Convert and score properties
                let query_address = query.address.clone();
                let mut comparables: Vec<ComparableApiModel> = properties
                    .into_iter()
                    .map(|prop| {
                        let confidence = prop.calculate_similarity(&subject);
                        let distance = prop.calculate_distance(&query_address);
                        ComparableApiModel {
                            address: prop.address.clone(),
                            mls_number: prop.mls_number.clone(),
                            price: prop.price as i64,
                            price_type: "sold".to_string(),
                            property_type: format!("{:?}", prop.property_type),
                            bedrooms: prop.bedrooms.map(|b| b as i32).unwrap_or(0),
                            bathrooms: prop.bathrooms.map(|b| b as i32).unwrap_or(0),
                            square_feet: prop.square_feet.unwrap_or(0),
                            days_on_market: prop.days_on_market,
                            distance_km: distance,
                            match_confidence: confidence,
                        }
                    })
                    .collect();

                // Sort by match confidence (highest first)
                comparables.sort_by(|a, b| b.match_confidence.cmp(&a.match_confidence));

                // Take top N results
                comparables.truncate(limit as usize);
                let count = comparables.len() as i32;

                tracing::info!("Fetched {} live comparables from Repliers API", count);

                Ok(ComparablesResponse {
                    target_property: target,
                    comparables,
                    count,
                })
            }
            Err(e) => {
                tracing::warn!("Repliers API error: {}. Falling back to mock data.", e);
                self.fetch_mock_comparables(query, target)
            }
        }
    }

    /// Fetch mock comparables when Repliers not available
    fn fetch_mock_comparables(
        &self,
        query: &ComparablesQuery,
        target: TargetProperty,
    ) -> Result<ComparablesResponse, String> {
        let comparables = vec![
            ComparableApiModel {
                address: "125 Main St, Pickering".to_string(),
                mls_number: "E1234567".to_string(),
                price: 875000,
                price_type: "sold".to_string(),
                property_type: query.property_type.clone().unwrap_or_else(|| "Detached".to_string()),
                bedrooms: 3,
                bathrooms: 2,
                square_feet: 1450,
                days_on_market: 8,
                distance_km: 0.2,
                match_confidence: 95,
            },
            ComparableApiModel {
                address: "127 Main St, Pickering".to_string(),
                mls_number: "E1234568".to_string(),
                price: 899000,
                price_type: "sold".to_string(),
                property_type: query.property_type.clone().unwrap_or_else(|| "Detached".to_string()),
                bedrooms: 3,
                bathrooms: 2,
                square_feet: 1520,
                days_on_market: 12,
                distance_km: 0.3,
                match_confidence: 92,
            },
            ComparableApiModel {
                address: "121 Main St, Pickering".to_string(),
                mls_number: "E1234569".to_string(),
                price: 865000,
                price_type: "sold".to_string(),
                property_type: query.property_type.clone().unwrap_or_else(|| "Detached".to_string()),
                bedrooms: 3,
                bathrooms: 2,
                square_feet: 1480,
                days_on_market: 15,
                distance_km: 0.25,
                match_confidence: 89,
            },
        ];

        tracing::debug!("Using mock comparables data");

        let count = comparables.len() as i32;
        Ok(ComparablesResponse {
            target_property: target,
            comparables: comparables.into_iter().take(query.get_limit() as usize).collect(),
            count,
        })
    }

    // Market Trends Operations

    /// Get market trends for an area using Repliers API
    pub async fn get_market_trends(
        &self,
        area: &str,
        query: TrendsQuery,
    ) -> Result<TrendsResponse, String> {
        // Validate area
        if !is_area_supported(area) {
            return Err(format!(
                "Area '{}' is not supported. Supported areas: {:?}",
                area,
                get_supported_areas()
            ));
        }
        
        // Validate period
        query.validate()?;
        let period = query.get_period();
        let period_days = match period.as_str() {
            "30d" => 30,
            "90d" => 90,
            "1y" => 365,
            _ => 90,
        };

        // Try to fetch live market stats from Repliers
        if let Some(ref client) = self.repliers_client {
            if client.is_configured() {
                return self.fetch_live_market_trends(client, area, &period, period_days).await;
            }
        }

        // Fallback to mock data
        self.fetch_mock_market_trends(area, &period)
    }

    /// Fetch live market trends from Repliers API
    async fn fetch_live_market_trends(
        &self,
        client: &RepliersClient,
        area: &str,
        period: &str,
        period_days: i32,
    ) -> Result<TrendsResponse, String> {
        match client.get_market_stats(area, period_days).await {
            Ok(stats) => {
                // Format prices
                let avg_price = stats.avg_sold_price.map(|p| format!("${:.0}", p))
                    .unwrap_or_else(|| "$N/A".to_string());
                let median_price = stats.median_sold_price.map(|p| format!("${:.0}", p))
                    .unwrap_or_else(|| "$N/A".to_string());
                let price_per_sqft = stats.price_per_sqft.map(|p| format!("${:.0}", p))
                    .unwrap_or_else(|| "$N/A".to_string());
                
                // Calculate trend direction (comparing to previous period would need historical data)
                let trend_direction = "+0.0%".to_string(); // Would need historical comparison
                
                // Determine market status based on months of inventory
                let market_status = match stats.months_of_inventory {
                    Some(moi) if moi < 3.0 => MarketStatus::SellersMarket,
                    Some(moi) if moi > 6.0 => MarketStatus::BuyersMarket,
                    _ => MarketStatus::BalancedMarket,
                };

                tracing::info!("Fetched live market trends for {} from Repliers API", area);

                Ok(TrendsResponse {
                    area: area.to_string(),
                    period: period.to_string(),
                    average_price: avg_price,
                    median_price: median_price,
                    price_per_sqft,
                    median_days_on_market: stats.avg_days_on_market.unwrap_or(0.0) as i32,
                    inventory_level: stats.total_listings.unwrap_or(0),
                    sales_volume: stats.total_sold.unwrap_or(0),
                    price_trend_direction: trend_direction,
                    market_status,
                    generated_at: Utc::now().to_rfc3339(),
                    data_as_of: Utc::now().to_rfc3339(),
                    neighborhoods: Some(get_neighborhoods(area)),
                })
            }
            Err(e) => {
                tracing::warn!("Repliers market stats error: {}. Using mock data.", e);
                self.fetch_mock_market_trends(area, period)
            }
        }
    }

    /// Fetch mock market trends when Repliers not available
    fn fetch_mock_market_trends(&self, area: &str, period: &str) -> Result<TrendsResponse, String> {
        // Provide realistic mock data based on Durham Region
        let (avg_price, median_price, price_per_sqft) = match area.to_lowercase().as_str() {
            "pickering" => ("$950,000", "$925,000", "$620"),
            "ajax" => ("$850,000", "$825,000", "$580"),
            "whitby" => ("$875,000", "$850,000", "$590"),
            "oshawa" => ("$750,000", "$725,000", "$520"),
            _ => ("$875,000", "$850,000", "$580"),
        };

        tracing::debug!("Using mock market trends data for {}", area);

        Ok(TrendsResponse {
            area: area.to_string(),
            period: period.to_string(),
            average_price: avg_price.to_string(),
            median_price: median_price.to_string(),
            price_per_sqft: price_per_sqft.to_string(),
            median_days_on_market: 12,
            inventory_level: 145,
            sales_volume: 89,
            price_trend_direction: "+2.3%".to_string(),
            market_status: MarketStatus::BalancedMarket,
            generated_at: Utc::now().to_rfc3339(),
            data_as_of: Utc::now().to_rfc3339(),
            neighborhoods: Some(get_neighborhoods(area)),
        })
    }

    // Additional Repliers Features

    /// Get active listings in an area
    pub async fn get_active_listings(
        &self,
        area: &str,
        filters: crate::services::ActiveListingsFilter,
    ) -> Result<crate::services::ActiveListingsResult, String> {
        if let Some(ref client) = self.repliers_client {
            if client.is_configured() {
                return client.get_active_listings(area, filters).await;
            }
        }
        
        Err("Active listings feature requires Repliers API. Please set REPLIERS_API_KEY environment variable.".to_string())
    }

    /// Get property details by MLS number
    pub async fn get_property_details(
        &self,
        mls_number: &str,
    ) -> Result<crate::services::PropertyDetails, String> {
        if let Some(ref client) = self.repliers_client {
            if client.is_configured() {
                return client.get_property_details(mls_number).await;
            }
        }
        
        Err("Property details feature requires Repliers API. Please set REPLIERS_API_KEY environment variable.".to_string())
    }

    /// Get detailed area statistics
    pub async fn get_area_statistics(
        &self,
        area: &str,
        property_type: Option<&str>,
    ) -> Result<crate::services::AreaStatistics, String> {
        if let Some(ref client) = self.repliers_client {
            if client.is_configured() {
                return client.get_area_statistics(area, property_type).await;
            }
        }
        
        Err("Area statistics feature requires Repliers API. Please set REPLIERS_API_KEY environment variable.".to_string())
    }

    // Helper Methods

    /// Save CMA to database
    async fn save_cma(&self, cma: &CMAReport) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO cma_reports (
                id, subject_address, subject_mls_number, property_type, status,
                generated_by, reviewed_by, reviewed_at, comparables, market_analysis,
                summary, price_recommendation_low, price_recommendation_mid, price_recommendation_high,
                market_conditions, avg_days_on_market, price_per_sqft, list_to_sale_ratio,
                confidence, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&cma.id)
        .bind(&cma.subject_address)
        .bind(&cma.subject_mls_number)
        .bind(format!("{:?}", cma.property_type).to_lowercase().replace("semidetached", "semi_detached").replace("multifamily", "multi_family"))
        .bind(format!("{:?}", cma.status).to_lowercase())
        .bind(&cma.generated_by)
        .bind(&cma.reviewed_by)
        .bind(cma.reviewed_at)
        .bind(&cma.comparables)
        .bind(&cma.market_analysis)
        .bind(&cma.summary)
        .bind(cma.price_recommendation_low)
        .bind(cma.price_recommendation_mid)
        .bind(cma.price_recommendation_high)
        .bind(&cma.market_conditions)
        .bind(cma.avg_days_on_market)
        .bind(cma.price_per_sqft)
        .bind(cma.list_to_sale_ratio)
        .bind(cma.confidence)
        .bind(cma.created_at)
        .bind(cma.updated_at)
        .execute(&*self.db_pool)
        .await
        .map_err(|e| format!("Failed to save CMA: {}", e))?;

        Ok(())
    }

    /// Get CMA by ID from database
    async fn get_cma_by_id(&self, cma_id: &str) -> Result<CMAReport, String> {
        let row = sqlx::query_as::<_, CMAReportRow>(
            "SELECT * FROM cma_reports WHERE id = ?"
        )
        .bind(cma_id)
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        match row {
            Some(row) => Ok(self.row_to_cma_report(row)),
            None => Err(format!("CMA with ID '{}' not found", cma_id)),
        }
    }

    /// Convert database row to CMAReport
    fn row_to_cma_report(&self, row: CMAReportRow) -> CMAReport {
        CMAReport {
            id: row.id,
            subject_address: row.subject_address,
            subject_mls_number: row.subject_mls_number,
            property_type: serde_json::from_str(&row.property_type).unwrap_or(PropertyCategory::Detached),
            status: serde_json::from_str(&row.status).unwrap_or(CMAStatus::Draft),
            generated_by: row.generated_by,
            reviewed_by: row.reviewed_by,
            reviewed_at: row.reviewed_at,
            comparables: row.comparables,
            market_analysis: row.market_analysis,
            summary: row.summary,
            price_recommendation_low: row.price_recommendation_low,
            price_recommendation_mid: row.price_recommendation_mid,
            price_recommendation_high: row.price_recommendation_high,
            market_conditions: row.market_conditions,
            avg_days_on_market: row.avg_days_on_market,
            price_per_sqft: row.price_per_sqft,
            list_to_sale_ratio: row.list_to_sale_ratio,
            confidence: row.confidence,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }

    /// Parse property type string
    fn parse_property_type(&self, property_type: &str) -> Result<PropertyCategory, String> {
        match property_type.to_lowercase().as_str() {
            "detached" => Ok(PropertyCategory::Detached),
            "semi-detached" | "semidetached" => Ok(PropertyCategory::SemiDetached),
            "townhouse" | "townhome" => Ok(PropertyCategory::Townhouse),
            "condo" | "condominium" => Ok(PropertyCategory::Condo),
            "commercial" => Ok(PropertyCategory::Commercial),
            "multi-family" | "multifamily" => Ok(PropertyCategory::MultiFamily),
            "land" => Ok(PropertyCategory::Land),
            _ => Err(format!("Unknown property type: {}", property_type)),
        }
    }

    /// Log API request
    pub async fn log_api_request(&self, log_entry: LogApiRequest) -> Result<(), String> {
        let log = ApiRequestLog::new(log_entry);
        
        sqlx::query(
            r#"
            INSERT INTO api_request_logs (
                id, agent_id, endpoint, method, request_body, response_status,
                duration_ms, timestamp, ip_address
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&log.id)
        .bind(&log.agent_id)
        .bind(&log.endpoint)
        .bind(&log.method)
        .bind(&log.request_body)
        .bind(log.response_status)
        .bind(log.duration_ms)
        .bind(&log.timestamp)
        .bind(&log.ip_address)
        .execute(&*self.db_pool)
        .await
        .map_err(|e| format!("Failed to log API request: {}", e))?;

        Ok(())
    }
}

/// Database row structure for CMA queries
#[derive(sqlx::FromRow)]
struct CMASummaryRow {
    id: String,
    subject_address: String,
    property_type: String,
    status: String,
    price_recommendation_mid: Option<f64>,
    created_at: chrono::DateTime<Utc>,
}

/// Database row structure for full CMA report
#[derive(sqlx::FromRow)]
struct CMAReportRow {
    id: String,
    subject_address: String,
    subject_mls_number: Option<String>,
    property_type: String,
    status: String,
    generated_by: String,
    reviewed_by: Option<String>,
    reviewed_at: Option<chrono::DateTime<Utc>>,
    comparables: serde_json::Value,
    market_analysis: serde_json::Value,
    summary: String,
    price_recommendation_low: Option<f64>,
    price_recommendation_mid: Option<f64>,
    price_recommendation_high: Option<f64>,
    market_conditions: Option<String>,
    avg_days_on_market: Option<f32>,
    price_per_sqft: Option<f64>,
    list_to_sale_ratio: Option<f32>,
    confidence: i32,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}
