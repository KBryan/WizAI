use clap::{Parser, Subcommand};
use anyhow::Result;
use tracing::{info, warn};

mod database;
mod scraper;
mod analyzer;
mod models;

use database::Database;
use scraper::RealEstateScraper;
use analyzer::MarketAnalyzer;

#[derive(Parser)]
#[command(name = "durham-realestate")]
#[command(about = "Durham Region Real Estate Market Analysis Tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Database file path
    #[arg(short, long, default_value = "durham_realestate.db")]
    database: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch data from sources
    Fetch {
        /// Source to fetch from (zolo, realtor, all)
        #[arg(short, long, default_value = "all")]
        source: String,
        
        /// Maximum number of listings to fetch
        #[arg(short, long, default_value = "100")]
        limit: usize,
    },
    
    /// Update existing data
    Update {
        /// Days back to check for updates
        #[arg(short, long, default_value = "7")]
        days: i64,
    },
    
    /// Show collection status
    Status,
    
    /// Show market statistics
    Stats {
        /// Municipality to filter by
        #[arg(short, long)]
        municipality: Option<String>,
        
        /// Property type to filter by
        #[arg(short, long)]
        property_type: Option<String>,
        
        /// Days back to analyze
        #[arg(short, long, default_value = "30")]
        days: i64,
    },
    
    /// Show price trends
    Trends {
        /// Municipality to analyze
        #[arg(short, long)]
        municipality: Option<String>,
        
        /// Property type
        #[arg(short, long)]
        property_type: Option<String>,
        
        /// Months back to analyze
        #[arg(short, long, default_value = "6")]
        months: i64,
    },
    
    /// Compare municipalities
    Compare {
        /// Property type to compare
        #[arg(short, long)]
        property_type: Option<String>,
    },
    
    /// Generate report
    Report {
        /// Output file path
        #[arg(short, long, default_value = "durham_realestate_report.md")]
        output: String,
        
        /// Include charts
        #[arg(long)]
        charts: bool,
    },
    
    /// Export data
    Export {
        /// Output file path
        #[arg(short, long, default_value = "durham_realestate.csv")]
        output: String,
        
        /// Format (csv, json)
        #[arg(short, long, default_value = "csv")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    // Initialize database
    let db = Database::new(&cli.database).await?;
    info!("Connected to database: {}", cli.database);
    
    match cli.command {
        Commands::Fetch { source, limit } => {
            info!("Fetching data from {} (limit: {})", source, limit);
            let scraper = RealEstateScraper::new(db);
            scraper.fetch(&source, limit).await?;
            info!("Data fetch complete");
        }
        
        Commands::Update { days } => {
            info!("Updating data from last {} days", days);
            let scraper = RealEstateScraper::new(db);
            scraper.update(days).await?;
            info!("Data update complete");
        }
        
        Commands::Status => {
            let stats = db.get_stats().await?;
            println!("\n╔══════════════════════════════════════════╗");
            println!("║       Durham Real Estate Database        ║");
            println!("╚══════════════════════════════════════════╝\n");
            println!("Total Listings:     {}", stats.total_listings);
            println!("Active Listings:    {}", stats.active_listings);
            println!("Sold Listings:      {}", stats.sold_listings);
            println!("Data Sources:       {}", stats.sources.join(", "));
            println!("Last Updated:       {}", stats.last_updated);
            println!("Date Range:         {} to {}", 
                stats.earliest_date.format("%Y-%m-%d"),
                stats.latest_date.format("%Y-%m-%d")
            );
        }
        
        Commands::Stats { municipality, property_type, days } => {
            let mun_str = municipality.clone();
            let prop_str = property_type.clone();
            
            let analyzer = MarketAnalyzer::new(db);
            let stats = analyzer.calculate_stats(municipality, property_type, days).await?;
            
            println!("\n╔══════════════════════════════════════════╗");
            println!("║           Market Statistics              ║");
            println!("╚══════════════════════════════════════════╝\n");
            
            if let Some(mun) = &mun_str {
                println!("Municipality: {}", mun);
            }
            if let Some(pt) = &prop_str {
                println!("Property Type: {}", pt);
            }
            println!("Analysis Period: Last {} days\n", days);
            
            println!("Listings Analyzed:  {}", stats.count);
            println!("Average Price:      ${:.0}", stats.avg_price);
            println!("Median Price:       ${:.0}", stats.median_price);
            println!("Min Price:          ${:.0}", stats.min_price);
            println!("Max Price:          ${:.0}", stats.max_price);
            println!("Avg Price/SqFt:     ${:.2}", stats.avg_price_per_sqft);
            println!("Avg Days on Market: {:.1}", stats.avg_days_on_market);
        }
        
        Commands::Trends { municipality, property_type, months } => {
            let analyzer = MarketAnalyzer::new(db);
            let trends = analyzer.calculate_trends(municipality, property_type, months).await?;
            
            println!("\n╔══════════════════════════════════════════╗");
            println!("║            Price Trends                  ║");
            println!("╚══════════════════════════════════════════╝\n");
            
            for trend in trends {
                println!("{} - {}: ${:.0} ({} listings)", 
                    trend.month,
                    trend.property_type,
                    trend.avg_price,
                    trend.count
                );
            }
        }
        
        Commands::Compare { property_type } => {
            let analyzer = MarketAnalyzer::new(db);
            let comparisons = analyzer.compare_municipalities(property_type).await?;
            
            println!("\n╔══════════════════════════════════════════╗");
            println!("║       Municipality Comparison            ║");
            println!("╚══════════════════════════════════════════╝\n");
            
            for comp in comparisons {
                println!("{:<15} Avg: ${:>10.0}  Median: ${:>10.0}  Count: {}",
                    comp.municipality,
                    comp.avg_price,
                    comp.median_price,
                    comp.count
                );
            }
        }
        
        Commands::Report { output, charts } => {
            info!("Generating report to {}", output);
            let analyzer = MarketAnalyzer::new(db);
            analyzer.generate_report(&output, charts).await?;
            info!("Report saved to {}", output);
        }
        
        Commands::Export { output, format } => {
            info!("Exporting data to {} (format: {})", output, format);
            db.export(&output, &format).await?;
            info!("Export complete: {}", output);
        }
    }
    
    Ok(())
}
