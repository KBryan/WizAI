//! Repliers API Client for live MLS data
//!
//! Direct integration with Repliers API for real-time property data,
//! comparables, and market trends in Durham Region.

use crate::models::{
    ComparableProperty, CMAReport, ListingStatus, MarketAnalysis, PropertyCategory,
    TrendDirection,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Repliers API Client
#[derive(Debug, Clone)]
pub struct RepliersClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl RepliersClient {
    /// Create a new Repliers API client
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.repliers.io".to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Check if client is configured
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    /// Search for properties (active listings and recent sales)
    pub async fn search_properties(
        &self,
        address: &str,
        radius: f64,
        property_type: Option<&str>,
        limit: i32,
    ) -> Result<Vec<RepliersProperty>, String> {
        // Parse address to extract city/area
        let area = self.extract_area_from_address(address);
        
        // Build search URL
        let mut url = format!(
            "{}/listings?area={}&radius={}",
            self.base_url,
            urlencoding::encode(&area),
            radius
        );

        // Add property type filter if specified
        if let Some(pt) = property_type {
            url.push_str(&format!("&type={}", urlencoding::encode(pt)));
        }

        // Add limit
        url.push_str(&format!("&limit={}", limit.min(50)));

        // Make request
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Repliers API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Repliers API error: {}", response.status()));
        }

        let data: RepliersListingsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Repliers response: {}", e))?;

        // Convert to our property model
        let properties: Vec<RepliersProperty> = data
            .listings
            .unwrap_or_default()
            .into_iter()
            .map(|listing| RepliersProperty {
                mls_number: listing.mls_number,
                address: format!("{}, {}, {}", 
                    listing.address.street_address,
                    listing.address.city,
                    listing.address.province
                ),
                property_type: self.parse_property_type(&listing.property_type),
                status: self.parse_listing_status(&listing.status),
                price: listing.price,
                original_list_price: listing.original_list_price,
                sold_date: listing.sold_date,
                list_date: listing.list_date,
                days_on_market: listing.days_on_market.unwrap_or(0),
                bedrooms: listing.bedrooms,
                bathrooms: listing.bathrooms,
                square_feet: listing.square_feet,
                lot_size: listing.lot_size,
                features: listing.features.unwrap_or_default(),
            })
            .collect();

        Ok(properties)
    }

    /// Get sold properties for comparables
    pub async fn get_sold_properties(
        &self,
        address: &str,
        radius: f64,
        days_back: i32,
        property_type: Option<&str>,
        limit: i32,
    ) -> Result<Vec<RepliersProperty>, String> {
        let area = self.extract_area_from_address(address);
        
        // Calculate date range
        let end_date = Utc::now().naive_utc();
        let start_date = end_date - chrono::Duration::days(days_back as i64);

        let mut url = format!(
            "{}/listings/sold?area={}&radius={}&startDate={}&endDate={}",
            self.base_url,
            urlencoding::encode(&area),
            radius,
            start_date,
            end_date
        );

        if let Some(pt) = property_type {
            url.push_str(&format!("&type={}", urlencoding::encode(pt)));
        }

        url.push_str(&format!("&limit={}", limit.min(50)));

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Repliers API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Repliers API error: {}", response.status()));
        }

        let data: RepliersListingsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Repliers response: {}", e))?;

        let properties: Vec<RepliersProperty> = data
            .listings
            .unwrap_or_default()
            .into_iter()
            .map(|listing| RepliersProperty {
                mls_number: listing.mls_number,
                address: format!("{}, {}, {}", 
                    listing.address.street_address,
                    listing.address.city,
                    listing.address.province
                ),
                property_type: self.parse_property_type(&listing.property_type),
                status: ListingStatus::Sold,
                price: listing.price,
                original_list_price: listing.original_list_price,
                sold_date: listing.sold_date,
                list_date: listing.list_date,
                days_on_market: listing.days_on_market.unwrap_or(0),
                bedrooms: listing.bedrooms,
                bathrooms: listing.bathrooms,
                square_feet: listing.square_feet,
                lot_size: listing.lot_size,
                features: listing.features.unwrap_or_default(),
            })
            .collect();

        Ok(properties)
    }

    /// Get market statistics for an area
    pub async fn get_market_stats(
        &self,
        area: &str,
        period_days: i32,
    ) -> Result<RepliersMarketStats, String> {
        let end_date = Utc::now().naive_utc();
        let start_date = end_date - chrono::Duration::days(period_days as i64);

        let url = format!(
            "{}/market-stats?area={}&startDate={}&endDate={}",
            self.base_url,
            urlencoding::encode(area),
            start_date,
            end_date
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Repliers API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Repliers API error: {}", response.status()));
        }

        let data: RepliersMarketStatsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Repliers response: {}", e))?;

        Ok(data.stats.unwrap_or_default())
    }

    /// Extract area from address (city name)
    fn extract_area_from_address(&self, address: &str) -> String {
        // Simple extraction - get last word or look for known Durham cities
        let lower = address.to_lowercase();
        
        let durham_cities = [
            "pickering", "ajax", "whitby", "oshawa", "courtice", 
            "bowmanville", "newcastle", "uxbridge", "scugog", "brock"
        ];
        
        for city in &durham_cities {
            if lower.contains(city) {
                return city.to_string();
            }
        }
        
        // Default to extracting last word
        address.split(',').last()
            .unwrap_or(address)
            .trim()
            .to_lowercase()
            .replace(" ", "-")
    }

    /// Parse property type string
    fn parse_property_type(&self, pt: &str) -> PropertyCategory {
        match pt.to_lowercase().as_str() {
            "detached" => PropertyCategory::Detached,
            "semi-detached" | "semi" => PropertyCategory::SemiDetached,
            "townhouse" | "townhome" => PropertyCategory::Townhouse,
            "condo" | "condominium" | "apartment" => PropertyCategory::Condo,
            "commercial" => PropertyCategory::Commercial,
            "multi-family" | "multifamily" => PropertyCategory::MultiFamily,
            "land" | "lot" => PropertyCategory::Land,
            _ => PropertyCategory::Detached,
        }
    }

    /// Parse listing status
    fn parse_listing_status(&self, status: &str) -> ListingStatus {
        match status.to_lowercase().as_str() {
            "active" | "for sale" => ListingStatus::Active,
            "sold" => ListingStatus::Sold,
            "pending" | "conditional" => ListingStatus::Pending,
            "expired" => ListingStatus::Expired,
            "withdrawn" => ListingStatus::Withdrawn,
            "leased" | "for lease" => ListingStatus::Leased,
            _ => ListingStatus::Active,
        }
    }

    // Additional Repliers Features

    /// Get active listings in an area
    pub async fn get_active_listings(
        &self,
        area: &str,
        filters: ActiveListingsFilter,
    ) -> Result<ActiveListingsResult, String> {
        let mut url = format!(
            "{}/listings?area={}",
            self.base_url,
            urlencoding::encode(area)
        );

        // Add optional filters
        if let Some(property_type) = &filters.property_type {
            url.push_str(&format!("&type={}", urlencoding::encode(property_type)));
        }

        if let Some(price_min) = filters.price_min {
            url.push_str(&format!("&minPrice={}", price_min));
        }

        if let Some(price_max) = filters.price_max {
            url.push_str(&format!("&maxPrice={}", price_max));
        }

        if let Some(bedrooms_min) = filters.bedrooms_min {
            url.push_str(&format!("&minBeds={}", bedrooms_min));
        }

        if let Some(bathrooms_min) = filters.bathrooms_min {
            url.push_str(&format!("&minBaths={}", bathrooms_min));
        }

        url.push_str(&format!("&limit={}", filters.limit.min(50)));

        // Add pagination if provided
        if let Some(offset) = filters.offset {
            url.push_str(&format!("&offset={}", offset));
        }

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Repliers API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Repliers API error: {}", response.status()));
        }

        let data: RepliersActiveListingsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Repliers response: {}", e))?;

        // Convert to active listing model
        let listings: Vec<ActiveListing> = data
            .listings
            .unwrap_or_default()
            .into_iter()
            .map(|listing| ActiveListing {
                mls_number: listing.mls_number.clone(),
                address: format!("{}, {}, {}", 
                    listing.address.street_address,
                    listing.address.city,
                    listing.address.province
                ),
                city: listing.address.city,
                province: listing.address.province,
                postal_code: listing.address.postal_code,
                property_type: self.parse_property_type(&listing.property_type),
                status: ListingStatus::Active,
                list_price: listing.price,
                bedrooms: listing.bedrooms,
                bathrooms: listing.bathrooms,
                square_feet: listing.square_feet,
                lot_size: listing.lot_size,
                list_date: listing.list_date,
                days_on_market: listing.days_on_market.unwrap_or(0),
                features: listing.features.unwrap_or_default(),
                description: listing.description,
                photos: listing.photos.unwrap_or_default(),
                agent_name: listing.agent_name,
                brokerage: listing.brokerage,
            })
            .collect();

        let count = listings.len();
        Ok(ActiveListingsResult {
            listings,
            total: data.total.unwrap_or(0),
            has_more: count >= filters.limit as usize,
        })
    }

    /// Get property details by MLS number
    pub async fn get_property_details(&self, mls_number: &str) -> Result<PropertyDetails, String> {
        let url = format!(
            "{}/listings/{}",
            self.base_url,
            urlencoding::encode(mls_number)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Repliers API request failed: {}", e))?;

        if response.status().as_u16() == 404 {
            return Err(format!("Property with MLS number '{}' not found", mls_number));
        }

        if !response.status().is_success() {
            return Err(format!("Repliers API error: {}", response.status()));
        }

        let data: RepliersPropertyDetailResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Repliers response: {}", e))?;

        let listing = data.listing
            .ok_or_else(|| format!("No property data found for MLS {}", mls_number))?;

        Ok(PropertyDetails {
            mls_number: listing.mls_number,
            address: format!("{}, {}, {}", 
                listing.address.street_address,
                listing.address.city,
                listing.address.province
            ),
            city: listing.address.city,
            province: listing.address.province,
            postal_code: listing.address.postal_code,
            property_type: self.parse_property_type(&listing.property_type),
            status: self.parse_listing_status(&listing.status),
            price: listing.price,
            original_list_price: listing.original_list_price,
            bedrooms: listing.bedrooms,
            bathrooms: listing.bathrooms,
            square_feet: listing.square_feet,
            lot_size: listing.lot_size,
            year_built: listing.year_built,
            description: listing.description,
            features: listing.features.unwrap_or_default(),
            photos: listing.photos.unwrap_or_default(),
            virtual_tour_url: listing.virtual_tour_url,
            floor_plan_url: listing.floor_plan_url,
            agent_name: listing.agent_name,
            agent_phone: listing.agent_phone,
            agent_email: listing.agent_email,
            brokerage: listing.brokerage,
            list_date: listing.list_date,
            sold_date: listing.sold_date,
            days_on_market: listing.days_on_market.unwrap_or(0),
            price_history: listing.price_history.unwrap_or_default().into_iter().map(|e| PriceHistoryEntry {
                date: e.date,
                price: e.price,
                event: e.event,
            }).collect(),
            property_taxes: listing.property_taxes,
            condo_fees: listing.condo_fees,
            parking_spaces: listing.parking_spaces,
            garage_spaces: listing.garage_spaces,
        })
    }

    /// Search listings with advanced filters
    pub async fn advanced_search(
        &self,
        criteria: AdvancedSearchCriteria,
    ) -> Result<ActiveListingsResult, String> {
        let mut url = format!("{}/listings/search", self.base_url);

        // Build query parameters
        let mut params: Vec<(String, String)> = Vec::new();

        if let Some(area) = &criteria.area {
            params.push(("area".to_string(), area.clone()));
        }

        if let Some(keywords) = &criteria.keywords {
            params.push(("keywords".to_string(), keywords.clone()));
        }

        if let Some(status) = &criteria.status {
            params.push(("status".to_string(), status.clone()));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&"));
        }

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Repliers API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Repliers API error: {}", response.status()));
        }

        let data: RepliersActiveListingsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Repliers response: {}", e))?;

        let listings: Vec<ActiveListing> = data
            .listings
            .unwrap_or_default()
            .into_iter()
            .map(|listing| ActiveListing {
                mls_number: listing.mls_number.clone(),
                address: format!("{}, {}, {}", 
                    listing.address.street_address,
                    listing.address.city,
                    listing.address.province
                ),
                city: listing.address.city,
                province: listing.address.province,
                postal_code: listing.address.postal_code,
                property_type: self.parse_property_type(&listing.property_type),
                status: self.parse_listing_status(&listing.status),
                list_price: listing.price,
                bedrooms: listing.bedrooms,
                bathrooms: listing.bathrooms,
                square_feet: listing.square_feet,
                lot_size: listing.lot_size,
                list_date: listing.list_date,
                days_on_market: listing.days_on_market.unwrap_or(0),
                features: listing.features.unwrap_or_default(),
                description: listing.description,
                photos: listing.photos.unwrap_or_default(),
                agent_name: listing.agent_name,
                brokerage: listing.brokerage,
            })
            .collect();

        let count = listings.len();
        Ok(ActiveListingsResult {
            listings,
            total: data.total.unwrap_or(0),
            has_more: count >= 20, // Default limit
        })
    }

    /// Get area statistics (extended)
    pub async fn get_area_statistics(
        &self,
        area: &str,
        property_type: Option<&str>,
    ) -> Result<AreaStatistics, String> {
        let mut url = format!(
            "{}/areas/{}/statistics",
            self.base_url,
            urlencoding::encode(area)
        );

        if let Some(pt) = property_type {
            url.push_str(&format!("?type={}", urlencoding::encode(pt)));
        }

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| format!("Repliers API request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Repliers API error: {}", response.status()));
        }

        let data: RepliersAreaStatsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Repliers response: {}", e))?;

        let stats = data.statistics
            .ok_or_else(|| format!("No statistics found for area '{}'", area))?;

        Ok(AreaStatistics {
            area: area.to_string(),
            property_type: property_type.map(|s| s.to_string()),
            avg_list_price: stats.avg_list_price,
            avg_sold_price: stats.avg_sold_price,
            median_sold_price: stats.median_sold_price,
            price_per_sqft: stats.price_per_sqft,
            total_active: stats.total_active,
            total_sold_30d: stats.total_sold_30d,
            total_sold_90d: stats.total_sold_90d,
            total_sold_1y: stats.total_sold_1y,
            avg_days_on_market: stats.avg_days_on_market,
            months_of_inventory: stats.months_of_inventory,
            new_listings_30d: stats.new_listings_30d,
            price_change_percent: stats.price_change_percent,
            updated_at: Utc::now().to_rfc3339(),
        })
    }
}

/// Repliers API property listing
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersListing {
    pub mls_number: String,
    pub address: RepliersAddress,
    #[serde(rename = "type")]
    pub property_type: String,
    pub status: String,
    pub price: f64,
    #[serde(rename = "originalListPrice")]
    pub original_list_price: Option<f64>,
    #[serde(rename = "soldDate")]
    pub sold_date: Option<NaiveDate>,
    #[serde(rename = "listDate")]
    pub list_date: NaiveDate,
    #[serde(rename = "daysOnMarket")]
    pub days_on_market: Option<i32>,
    pub bedrooms: Option<f32>,
    pub bathrooms: Option<f32>,
    #[serde(rename = "squareFeet")]
    pub square_feet: Option<i32>,
    #[serde(rename = "lotSize")]
    pub lot_size: Option<String>,
    pub features: Option<Vec<String>>,
}

/// Repliers address structure
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersAddress {
    #[serde(rename = "streetAddress")]
    pub street_address: String,
    pub city: String,
    pub province: String,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
}

/// Repliers listings response
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersListingsResponse {
    pub listings: Option<Vec<RepliersListing>>,
    pub total: Option<i32>,
}

/// Repliers market stats response
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersMarketStatsResponse {
    pub stats: Option<RepliersMarketStats>,
}

/// Repliers market statistics
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RepliersMarketStats {
    #[serde(rename = "avgListPrice")]
    pub avg_list_price: Option<f64>,
    #[serde(rename = "avgSoldPrice")]
    pub avg_sold_price: Option<f64>,
    #[serde(rename = "medianSoldPrice")]
    pub median_sold_price: Option<f64>,
    #[serde(rename = "avgDaysOnMarket")]
    pub avg_days_on_market: Option<f32>,
    #[serde(rename = "totalListings")]
    pub total_listings: Option<i32>,
    #[serde(rename = "totalSold")]
    pub total_sold: Option<i32>,
    #[serde(rename = "monthsOfInventory")]
    pub months_of_inventory: Option<f32>,
    #[serde(rename = "pricePerSqFt")]
    pub price_per_sqft: Option<f64>,
    #[serde(rename = "salesVolume")]
    pub sales_volume: Option<i32>,
}

/// Property data from Repliers
#[derive(Debug, Clone)]
pub struct RepliersProperty {
    pub mls_number: String,
    pub address: String,
    pub property_type: PropertyCategory,
    pub status: ListingStatus,
    pub price: f64,
    pub original_list_price: Option<f64>,
    pub sold_date: Option<NaiveDate>,
    pub list_date: NaiveDate,
    pub days_on_market: i32,
    pub bedrooms: Option<f32>,
    pub bathrooms: Option<f32>,
    pub square_feet: Option<i32>,
    pub lot_size: Option<String>,
    pub features: Vec<String>,
}

impl RepliersProperty {
    /// Calculate distance from subject property (simplified)
    pub fn calculate_distance(&self, _subject_address: &str) -> f64 {
        // In real implementation, use geocoding
        // For now, return random distance based on address similarity
        0.5 + (self.address.len() % 10) as f64 * 0.1
    }

    /// Calculate similarity score (0-100)
    pub fn calculate_similarity(&self, subject: &ComparableSubject) -> i32 {
        let mut score = 50; // Base score
        
        // Property type match
        if self.property_type == subject.property_type {
            score += 20;
        }
        
        // Size similarity
        if let (Some(sqft), Some(subject_sqft)) = (self.square_feet, subject.square_feet) {
            let diff = (sqft as f64 - subject_sqft as f64).abs() / subject_sqft as f64;
            if diff < 0.15 { score += 15; }
            else if diff < 0.25 { score += 10; }
            else if diff < 0.35 { score += 5; }
        }
        
        // Bedroom/bathroom match
        if let (Some(beds), Some(subject_beds)) = (self.bedrooms, subject.bedrooms) {
            if (beds - subject_beds).abs() < 1.0 { score += 10; }
        }
        
        // Recent sales score higher
        if self.status == ListingStatus::Sold {
            if let Some(sold_date) = self.sold_date {
                let today = Utc::now().naive_utc().date();
                let days_ago = (today - sold_date).num_days();
                if days_ago < 30 { score += 5; }
            }
        }
        
        score.min(100)
    }
}

/// Subject property for comparison
#[derive(Debug, Clone)]
pub struct ComparableSubject {
    pub property_type: PropertyCategory,
    pub bedrooms: Option<f32>,
    pub bathrooms: Option<f32>,
    pub square_feet: Option<i32>,
}

/// Convert RepliersProperty to our ComparableProperty model
impl From<RepliersProperty> for ComparableProperty {
    fn from(prop: RepliersProperty) -> Self {
        Self {
            mls_number: prop.mls_number,
            address: prop.address.clone(),
            property_type: prop.property_type,
            status: prop.status,
            price: prop.price,
            original_list_price: prop.original_list_price,
            sold_date: prop.sold_date,
            list_date: prop.list_date,
            days_on_market: prop.days_on_market,
            bedrooms: prop.bedrooms,
            bathrooms: prop.bathrooms,
            square_feet: prop.square_feet,
            lot_size: prop.lot_size,
            features: prop.features,
            distance_km: 0.0, // Will be calculated
            similarity_score: 0, // Will be calculated
            source: "Repliers API".to_string(),
        }
    }
}

// Additional Repliers Data Structures

/// Filter for active listings search
#[derive(Debug, Clone, Default)]
pub struct ActiveListingsFilter {
    pub property_type: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub bedrooms_min: Option<i32>,
    pub bathrooms_min: Option<i32>,
    pub limit: i32,
    pub offset: Option<i32>,
}

impl ActiveListingsFilter {
    pub fn new() -> Self {
        Self {
            limit: 20,
            ..Default::default()
        }
    }

    pub fn with_property_type(mut self, property_type: impl Into<String>) -> Self {
        self.property_type = Some(property_type.into());
        self
    }

    pub fn with_price_range(mut self, min: f64, max: f64) -> Self {
        self.price_min = Some(min);
        self.price_max = Some(max);
        self
    }

    pub fn with_bedrooms(mut self, min: i32) -> Self {
        self.bedrooms_min = Some(min);
        self
    }

    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = limit.min(50);
        self
    }
}

/// Result of active listings search
#[derive(Debug, Clone)]
pub struct ActiveListingsResult {
    pub listings: Vec<ActiveListing>,
    pub total: i32,
    pub has_more: bool,
}

/// Active listing data
#[derive(Debug, Clone, Serialize)]
pub struct ActiveListing {
    pub mls_number: String,
    pub address: String,
    pub city: String,
    pub province: String,
    pub postal_code: Option<String>,
    pub property_type: PropertyCategory,
    pub status: ListingStatus,
    pub list_price: f64,
    pub bedrooms: Option<f32>,
    pub bathrooms: Option<f32>,
    pub square_feet: Option<i32>,
    pub lot_size: Option<String>,
    pub list_date: NaiveDate,
    pub days_on_market: i32,
    pub features: Vec<String>,
    pub description: Option<String>,
    pub photos: Vec<String>,
    pub agent_name: Option<String>,
    pub brokerage: Option<String>,
}

/// Detailed property information
#[derive(Debug, Clone, Serialize)]
pub struct PropertyDetails {
    pub mls_number: String,
    pub address: String,
    pub city: String,
    pub province: String,
    pub postal_code: Option<String>,
    pub property_type: PropertyCategory,
    pub status: ListingStatus,
    pub price: f64,
    pub original_list_price: Option<f64>,
    pub bedrooms: Option<f32>,
    pub bathrooms: Option<f32>,
    pub square_feet: Option<i32>,
    pub lot_size: Option<String>,
    pub year_built: Option<i32>,
    pub description: Option<String>,
    pub features: Vec<String>,
    pub photos: Vec<String>,
    pub virtual_tour_url: Option<String>,
    pub floor_plan_url: Option<String>,
    pub agent_name: Option<String>,
    pub agent_phone: Option<String>,
    pub agent_email: Option<String>,
    pub brokerage: Option<String>,
    pub list_date: NaiveDate,
    pub sold_date: Option<NaiveDate>,
    pub days_on_market: i32,
    pub price_history: Vec<PriceHistoryEntry>,
    pub property_taxes: Option<f64>,
    pub condo_fees: Option<f64>,
    pub parking_spaces: Option<i32>,
    pub garage_spaces: Option<i32>,
}

/// Price history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryEntry {
    pub date: NaiveDate,
    pub price: f64,
    pub event: String, // "listed", "price_change", "sold", etc.
}

/// Advanced search criteria
#[derive(Debug, Clone, Default)]
pub struct AdvancedSearchCriteria {
    pub area: Option<String>,
    pub keywords: Option<String>,
    pub status: Option<String>,
}

/// Area statistics
#[derive(Debug, Clone, Serialize)]
pub struct AreaStatistics {
    pub area: String,
    pub property_type: Option<String>,
    pub avg_list_price: Option<f64>,
    pub avg_sold_price: Option<f64>,
    pub median_sold_price: Option<f64>,
    pub price_per_sqft: Option<f64>,
    pub total_active: Option<i32>,
    pub total_sold_30d: Option<i32>,
    pub total_sold_90d: Option<i32>,
    pub total_sold_1y: Option<i32>,
    pub avg_days_on_market: Option<f32>,
    pub months_of_inventory: Option<f32>,
    pub new_listings_30d: Option<i32>,
    pub price_change_percent: Option<f32>,
    pub updated_at: String,
}

// Repliers API Response Structures

/// Repliers active listings response
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersActiveListingsResponse {
    pub listings: Option<Vec<RepliersActiveListing>>,
    pub total: Option<i32>,
}

/// Repliers active listing
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersActiveListing {
    pub mls_number: String,
    pub address: RepliersAddress,
    #[serde(rename = "type")]
    pub property_type: String,
    pub status: String,
    pub price: f64,
    #[serde(rename = "listDate")]
    pub list_date: NaiveDate,
    #[serde(rename = "daysOnMarket")]
    pub days_on_market: Option<i32>,
    pub bedrooms: Option<f32>,
    pub bathrooms: Option<f32>,
    #[serde(rename = "squareFeet")]
    pub square_feet: Option<i32>,
    #[serde(rename = "lotSize")]
    pub lot_size: Option<String>,
    pub description: Option<String>,
    pub features: Option<Vec<String>>,
    pub photos: Option<Vec<String>>,
    #[serde(rename = "agentName")]
    pub agent_name: Option<String>,
    pub brokerage: Option<String>,
}

/// Repliers property detail response
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersPropertyDetailResponse {
    pub listing: Option<RepliersPropertyDetail>,
}

/// Repliers property detail
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersPropertyDetail {
    pub mls_number: String,
    pub address: RepliersAddress,
    #[serde(rename = "type")]
    pub property_type: String,
    pub status: String,
    pub price: f64,
    #[serde(rename = "originalListPrice")]
    pub original_list_price: Option<f64>,
    #[serde(rename = "listDate")]
    pub list_date: NaiveDate,
    #[serde(rename = "soldDate")]
    pub sold_date: Option<NaiveDate>,
    #[serde(rename = "daysOnMarket")]
    pub days_on_market: Option<i32>,
    pub bedrooms: Option<f32>,
    pub bathrooms: Option<f32>,
    #[serde(rename = "squareFeet")]
    pub square_feet: Option<i32>,
    #[serde(rename = "lotSize")]
    pub lot_size: Option<String>,
    #[serde(rename = "yearBuilt")]
    pub year_built: Option<i32>,
    pub description: Option<String>,
    pub features: Option<Vec<String>>,
    pub photos: Option<Vec<String>>,
    #[serde(rename = "virtualTourUrl")]
    pub virtual_tour_url: Option<String>,
    #[serde(rename = "floorPlanUrl")]
    pub floor_plan_url: Option<String>,
    #[serde(rename = "agentName")]
    pub agent_name: Option<String>,
    #[serde(rename = "agentPhone")]
    pub agent_phone: Option<String>,
    #[serde(rename = "agentEmail")]
    pub agent_email: Option<String>,
    pub brokerage: Option<String>,
    #[serde(rename = "priceHistory")]
    pub price_history: Option<Vec<RepliersPriceHistoryEntry>>,
    #[serde(rename = "propertyTaxes")]
    pub property_taxes: Option<f64>,
    #[serde(rename = "condoFees")]
    pub condo_fees: Option<f64>,
    #[serde(rename = "parkingSpaces")]
    pub parking_spaces: Option<i32>,
    #[serde(rename = "garageSpaces")]
    pub garage_spaces: Option<i32>,
}

/// Repliers price history entry
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersPriceHistoryEntry {
    pub date: NaiveDate,
    pub price: f64,
    pub event: String,
}

/// Repliers area stats response
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersAreaStatsResponse {
    pub statistics: Option<RepliersAreaStatistics>,
}

/// Repliers area statistics
#[derive(Debug, Clone, Deserialize)]
pub struct RepliersAreaStatistics {
    #[serde(rename = "avgListPrice")]
    pub avg_list_price: Option<f64>,
    #[serde(rename = "avgSoldPrice")]
    pub avg_sold_price: Option<f64>,
    #[serde(rename = "medianSoldPrice")]
    pub median_sold_price: Option<f64>,
    #[serde(rename = "pricePerSqFt")]
    pub price_per_sqft: Option<f64>,
    #[serde(rename = "totalActive")]
    pub total_active: Option<i32>,
    #[serde(rename = "totalSold30d")]
    pub total_sold_30d: Option<i32>,
    #[serde(rename = "totalSold90d")]
    pub total_sold_90d: Option<i32>,
    #[serde(rename = "totalSold1y")]
    pub total_sold_1y: Option<i32>,
    #[serde(rename = "avgDaysOnMarket")]
    pub avg_days_on_market: Option<f32>,
    #[serde(rename = "monthsOfInventory")]
    pub months_of_inventory: Option<f32>,
    #[serde(rename = "newListings30d")]
    pub new_listings_30d: Option<i32>,
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: Option<f32>,
}
