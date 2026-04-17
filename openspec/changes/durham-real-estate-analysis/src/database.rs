use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;
use tracing::info;

use crate::models::{Listing, Stats};

pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let db = Self { pool };
        db.init().await?;

        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS listings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL,
                source_id TEXT NOT NULL UNIQUE,
                address TEXT NOT NULL,
                municipality TEXT NOT NULL,
                postal_code TEXT,
                price REAL,
                sold_price REAL,
                property_type TEXT NOT NULL,
                bedrooms REAL,
                bathrooms REAL,
                square_feet REAL,
                listing_date TEXT,
                sold_date TEXT,
                days_on_market INTEGER,
                description TEXT,
                url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_listings_municipality ON listings(municipality);
            CREATE INDEX IF NOT EXISTS idx_listings_property_type ON listings(property_type);
            CREATE INDEX IF NOT EXISTS idx_listings_listing_date ON listings(listing_date);
            CREATE INDEX IF NOT EXISTS idx_listings_source ON listings(source);
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Database initialized");
        Ok(())
    }

    pub async fn insert_listing(&self, listing: &Listing) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT OR REPLACE INTO listings (
                source, source_id, address, municipality, postal_code,
                price, sold_price, property_type, bedrooms, bathrooms,
                square_feet, listing_date, sold_date, days_on_market,
                description, url, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&listing.source)
        .bind(&listing.source_id)
        .bind(&listing.address)
        .bind(&listing.municipality)
        .bind(&listing.postal_code)
        .bind(listing.price)
        .bind(listing.sold_price)
        .bind(&listing.property_type)
        .bind(listing.bedrooms)
        .bind(listing.bathrooms)
        .bind(listing.square_feet)
        .bind(listing.listing_date.map(|d| d.to_rfc3339()))
        .bind(listing.sold_date.map(|d| d.to_rfc3339()))
        .bind(listing.days_on_market)
        .bind(&listing.description)
        .bind(&listing.url)
        .bind(listing.created_at.to_rfc3339())
        .bind(listing.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_stats(&self) -> Result<Stats> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM listings")
            .fetch_one(&self.pool)
            .await?;

        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM listings WHERE sold_date IS NULL"
        )
        .fetch_one(&self.pool)
        .await?;

        let sold: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM listings WHERE sold_date IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?;

        let sources: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT source FROM listings"
        )
        .fetch_all(&self.pool)
        .await?;

        let date_range: (Option<String>, Option<String>) = sqlx::query_as(
            "SELECT MIN(listing_date), MAX(listing_date) FROM listings"
        )
        .fetch_one(&self.pool)
        .await?;

        let earliest = date_range
            .0
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let latest = date_range
            .1
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        Ok(Stats {
            total_listings: total,
            active_listings: active,
            sold_listings: sold,
            sources,
            last_updated: Utc::now(),
            earliest_date: earliest,
            latest_date: latest,
        })
    }

    pub async fn export(&self, output_path: &str, format: &str) -> Result<()> {
        match format {
            "csv" => self.export_csv(output_path).await,
            "json" => self.export_json(output_path).await,
            _ => Err(anyhow!("Unsupported export format: {}", format)),
        }
    }

    async fn export_csv(&self, output_path: &str) -> Result<()> {
        // TODO: Implement CSV export
        info!("Exporting to CSV: {}", output_path);
        Ok(())
    }

    async fn export_json(&self, output_path: &str) -> Result<()> {
        // TODO: Implement JSON export
        info!("Exporting to JSON: {}", output_path);
        Ok(())
    }
}
